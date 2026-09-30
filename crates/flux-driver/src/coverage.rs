//! An append-only evidence journal. A crash must leave unattempted bodies visible.
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
};

use flux_config as config;
use flux_middle::global_env::GlobalEnv;
use rustc_hir::{
    def::DefKind,
    def_id::{LOCAL_CRATE, LocalDefId},
};
use serde_json::{Value, json};

#[derive(Default)]
pub(crate) struct Coverage {
    file: Option<fs::File>,
    statuses: HashMap<LocalDefId, &'static str>,
}

impl Coverage {
    pub fn new(genv: GlobalEnv) -> Self {
        if !config::coverage() {
            return Self::default();
        }
        let tcx = genv.tcx();
        let crate_name = tcx.crate_name(LOCAL_CRATE).to_string();
        let open = || -> io::Result<fs::File> {
            fs::create_dir_all(config::log_dir())?;
            // Distinguish multiple invocations of the same crate in one Cargo run.
            let path = config::log_dir()
                .join(format!("{crate_name}-{}-coverage.jsonl", std::process::id()));
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
        };
        let file = match open() {
            Ok(file) => file,
            Err(err) => {
                eprintln!("warning: cannot create Flux coverage journal: {err}");
                return Self::default();
            }
        };
        let mut this = Self { file: Some(file), statuses: HashMap::new() };
        this.emit(json!({"event": "start", "schema_version": 1, "crate": crate_name,
            "scope": "active compiler configuration; local functions, methods and closures",
            "trust_dependencies": "not_collected"}));
        for local_id in tcx.iter_local_def_id() {
            let id = genv.maybe_extern_id(local_id);
            if genv.is_dummy(local_id) || id.as_local().is_none() {
                continue;
            }
            let kind = tcx.def_kind(local_id);
            if !matches!(kind, DefKind::Fn | DefKind::AssocFn | DefKind::Closure) {
                continue;
            }
            let span = tcx.def_span(local_id);
            let has_body = tcx.is_mir_available(local_id);
            let trusted_derive = span.in_derive_expansion()
                && genv
                    .derive_self_ty(local_id)
                    .is_some_and(|id| genv.trusted_derive(id));
            let status = if genv.ignored(local_id) {
                "ignored"
            } else if !genv.included(id) {
                "not_selected"
            } else if !has_body {
                "no_body"
            } else if genv.trusted(local_id) || trusted_derive {
                "trusted"
            } else if matches!(kind, DefKind::Closure) {
                "enclosing_body_obligation"
            } else {
                "not_attempted"
            };
            this.statuses.insert(local_id, status);
            this.emit(json!({"event": "function", "id": format!("{local_id:?}"),
                "def_path": tcx.def_path_str(local_id.to_def_id()),
                "parent": tcx.opt_local_parent(local_id).map(|id| tcx.def_path_str(id.to_def_id())),
                "kind": format!("{kind:?}"), "has_body": has_body,
                "source": tcx.sess.source_map().span_to_diagnostic_string(span),
                "generated": span.from_expansion(),
                "explicit_contract": genv.spec_attr_span(local_id.to_def_id()).is_some(),
                "contract": genv.spec_attr_string(local_id.to_def_id()),
                "status": status}));
        }
        this.emit(json!({"event": "inventory_complete"}));
        this
    }

    pub fn begin(&mut self, local_id: LocalDefId) {
        if self.statuses.get(&local_id) == Some(&"not_attempted") {
            self.emit(
                json!({"event": "result", "id": format!("{local_id:?}"), "status": "in_progress"}),
            );
        }
    }

    pub fn end(&mut self, genv: GlobalEnv, local_id: LocalDefId, outcome: &str) {
        let Some(initial) = self.statuses.get(&local_id) else { return };
        let status = if outcome != "accepted" {
            outcome
        } else if *initial != "not_attempted" {
            return;
        } else if config::lean().is_emit()
            || (config::lean().is_check() && genv.proven_externally(local_id).is_some())
        {
            // The later batch Lean check is not attributed to individual bodies here.
            "external_proof_unattributed"
        } else {
            "accepted_with_observed_models"
        };
        self.emit(json!({"event": "result", "id": format!("{local_id:?}"), "status": status}));
    }

    pub fn finish(&mut self, success: bool) {
        self.emit(json!({"event": "finish", "success": success}));
    }

    fn emit(&mut self, value: Value) {
        let Some(file) = &mut self.file else { return };
        let result = (|| -> io::Result<()> {
            serde_json::to_writer(&mut *file, &value)?;
            file.write_all(b"\n")?;
            file.flush()
        })();
        if let Err(err) = result {
            eprintln!("warning: cannot write Flux coverage journal: {err}");
            self.file = None;
        }
    }
}

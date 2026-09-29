// Reproducer attempt for the one Flux warning in codex-mermaid that looked like a real panic:
// `put_text` computes `row[column + 1..column + width]`, which panics for zero-width chars
// (width 0 => start > end). Result: every path rejects them first via `check_label`, so it is
// safe (a false positive). Keep as the template for triaging Flux warnings in Codex.
//
//   cp handoff/codex/mermaid_zero_width.rs <codex>/codex-rs/mermaid/examples/zw.rs
//   cargo run -q -p codex-mermaid --example zw
fn main() {
    let z = "e\u{301}"; // combining accent: `UnicodeWidthChar::width` is Some(0)
    let cases = vec![
        format!("graph TD\n  A[{z}] --> B"),
        format!("graph TD\n  A -->|{z}| B"),
        format!("graph TD\n  A -- {z} --> B"),
        format!("graph LR\n  A{{{z}}} --> B"),
        format!("graph TD\n  A([{z}]) --> B"),
        format!("graph TD\n  A[\"{z}\"] --> B"),
        format!("sequenceDiagram\n  participant {z}\n  {z}->>B: hi"),
        format!("sequenceDiagram\n  participant A as {z}\n  A->>B: hi"),
        format!("sequenceDiagram\n  A->>{z}: hi"),
        format!("sequenceDiagram\n  A->>B: hi\n  Note over A: {z}"),
        format!("sequenceDiagram\n  loop {z}\n  A->>B: hi\n  end"),
        format!("stateDiagram-v2\n  [*] --> {z}"),
        format!("stateDiagram-v2\n  s1: {z}\n  [*] --> s1"),
        format!("stateDiagram-v2\n  s1 --> s2: {z}"),
        format!("classDiagram\n  class {z}"),
        format!("classDiagram\n  A --> B : {z}"),
        format!("erDiagram\n  A ||--o{{ B : {z}"),
    ];
    std::panic::set_hook(Box::new(|_| {}));
    for src in &cases {
        let first = src.lines().nth(1).unwrap_or("").trim();
        match std::panic::catch_unwind(|| codex_mermaid::render(src, 80)) {
            Ok(Ok(_)) => println!("OK      | {first}"),
            Ok(Err(e)) => println!("{e:?} | {first}"),
            Err(_) => println!("PANIC   | {first}"),
        }
    }
}

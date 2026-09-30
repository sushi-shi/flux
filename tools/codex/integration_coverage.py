#!/usr/bin/env python3
"""Run under nix develop ./handoff after building Flux's sysroot."""
import json
from pathlib import Path
import subprocess
import tempfile

from corpus import function_map

root = Path(__file__).resolve().parents[2]
fixture = Path(__file__).parent / 'fixtures/coverage.rs'
for selected in (False, True):
    with tempfile.TemporaryDirectory(prefix='flux-coverage-test-') as directory:
        args = ['cargo', 'x', 'run', str(fixture), '--', '-Fcoverage=on',
                '-Flog-dir=' + directory]
        if selected:
            args.append('-Finclude=def:good')
        result = subprocess.run(args, cwd=root, capture_output=True, text=True)
        mapping = function_map(Path(directory))
        assert mapping['status'] == 'complete_observations', (mapping, result.stderr)
        invocation, = mapping['invocations']
        functions = {f['def_path']: f for f in invocation['functions']}
        assert functions['good']['status'] == 'accepted_with_observed_models'
        assert functions['good']['explicit_contract']
        assert 'fn(x: i32)' in functions['good']['contract']
        assert functions['ignored']['status'] == 'ignored'
        if selected:
            assert result.returncode == 0, result.stderr
            assert functions['bad']['status'] == 'not_selected'
            assert functions['trusted']['status'] == 'not_selected'
        else:
            assert result.returncode != 0, result.stderr
            assert functions['bad']['status'] == 'check_error'
            assert functions['trusted']['status'] == 'trusted'
            assert functions['Declaration::no_body']['status'] == 'no_body'
            assert functions['enclosing::{closure#0}']['status'] == 'enclosing_body_obligation'
        print('selected' if selected else 'all', json.dumps(invocation['counts']))

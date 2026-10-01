#!/usr/bin/env python3
"""Check the real Codex split contract and falsify it with implementation mutations."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from corpus import check, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--flux', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--native-target', type=Path, required=True)
    args = parser.parse_args()
    source, flux, output = args.source.resolve(), args.flux.resolve(), args.output.resolve()
    if output.exists():
        raise ValueError('Use a new output directory')
    path = source / 'codex-rs/utils/string/src/truncate.rs'
    original = path.read_bytes()
    mutations = {
        'prefix_over_budget': (b'if char_end <= beginning_bytes {', b'if idx <= beginning_bytes {'),
        'suffix_over_budget': (b'len.saturating_sub(end_bytes)', b'len.saturating_sub(end_bytes.saturating_add(1))'),
        'wrong_removed_count': (b'removed_chars.saturating_add(1)', b'removed_chars.saturating_add(2)'),
    }
    for before, _ in mutations.values():
        if original.count(before) != 1:
            raise ValueError('Expected the pinned split_string implementation')
    output.mkdir(parents=True)
    results = []
    env = dict(os.environ, CARGO_TARGET_DIR=str(args.native_target.resolve()))

    def run(label, expected_flux, native_fails):
        report = check(source, flux, ['codex-utils-string'], output / label, True,
                       ['def:split_string'], timeout=90,
                       proof_cache=output / 'proof-cache')
        result, = report['packages']
        if result['status'] != expected_flux:
            raise AssertionError((label, result['status'], expected_flux))
        mapping = json.loads((output / label / result['function_map']).read_text())
        selected = [f for invocation in mapping['invocations'] for f in invocation['functions']
                    if f['def_path'] == 'truncate::split_string']
        if len(selected) != 1 or selected[0]['status'] not in ('check_error', 'accepted_with_observed_models'):
            raise AssertionError((label, selected))
        native = subprocess.run(
            ['just', 'test', '-p', 'codex-utils-string', 'split_string_matches_boundary_oracle'],
            cwd=source / 'codex-rs', env=env, text=True,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        (output / label / 'native.log').write_text(native.stdout)
        if native_fails:
            if native.returncode == 0 or 'FAIL' not in native.stdout or 'split_string_matches_boundary_oracle' not in native.stdout:
                raise AssertionError((label, 'expected an actual native test failure', native.returncode))
        elif native.returncode != 0:
            raise AssertionError((label, 'native test failed', native.returncode))
        results.append({'stage': label, 'flux_status': result['status'],
                        'function_status': selected[0]['status'],
                        'native_returncode': native.returncode,
                        'elapsed_seconds': result['elapsed_seconds'],
                        'performance': result['performance']})
        write_json(output / 'experiment.json', {
            'scope': 'split_string budgets, non-overlap and UTF-8 safety; exact removed count remains unproved',
            'source': report['source'], 'driver_sha256': report['driver_sha256'],
            'model_sha256': report['model_sha256'], 'results': results,
        })

    try:
        run('cold', 'checked_with_observed_models', False)
        run('unchanged', 'checked_with_observed_models', False)
        for label, (before, after) in mutations.items():
            path.write_bytes(original.replace(before, after))
            # Deliberately document the present specification gap: count is an
            # unconstrained usize in the first contract. Native tests reject it.
            expected = 'checked_with_observed_models' if label == 'wrong_removed_count' else 'proof_failure'
            run(label, expected, True)
            path.write_bytes(original)
            run(label + '-restored', 'checked_with_observed_models', False)
        warm, = results[1]['performance']
        if warm['cached_bodies'] != 1 or warm['executed_solver_queries'] != 0:
            raise AssertionError(('unchanged proof was not reused', warm))
    finally:
        path.write_bytes(original)


if __name__ == '__main__':
    main()

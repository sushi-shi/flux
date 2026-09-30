#!/usr/bin/env python3
"""Measure actual Codex budget checks and falsify stale proof-cache reuse."""
import argparse
import json
from pathlib import Path

from corpus import check, write_json


def validate_reuse(results):
    by_stage = {r['stage']: r for r in results}
    for repeat in range(3):
        evidence, = by_stage[f'unchanged-{repeat}']['performance']
        assert evidence['cached_bodies'] == 3, evidence
        assert evidence['executed_solver_queries'] == 0, evidence
    for stage, function in (
        ('code_mutation', 'truncate::approx_bytes_for_tokens'),
        ('contract_mutation', 'truncate::approx_bytes_for_tokens'),
        ('proof_mutation', 'truncate::budget_proofs::token_budget_is_monotone'),
    ):
        result = by_stage[stage]
        assert next(f['status'] for f in result['functions'] if f['def_path'] == function) == 'check_error'
        evidence, = result['performance']
        assert 1 <= evidence['cached_bodies'] < 3, evidence
        assert evidence['executed_solver_queries'] >= 1, evidence


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--flux', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source, flux, output = args.source.resolve(), args.flux.resolve(), args.output.resolve()
    if output.exists():
        raise ValueError('Use a new output directory')
    path = source / 'codex-rs/utils/string/src/truncate.rs'
    original = path.read_bytes()
    replacements = {
        'code_mutation': (b'tokens.saturating_mul(APPROX_BYTES_PER_TOKEN)', b'tokens.saturating_mul(3)'),
        'contract_mutation': (b'&& tokens <= bytes', b'&& tokens < bytes'),
        'proof_mutation': (b'assert!(smaller_bytes <= larger_bytes)', b'assert!(smaller_bytes < larger_bytes)'),
    }
    for before, _ in replacements.values():
        if original.count(before) != 1:
            raise ValueError('Benchmark needs the pinned Codex budget contract and proof')
    selections = ['def:split_budget', 'def:approx_bytes_for_tokens', 'def:token_budget_is_monotone']
    output.mkdir(parents=True)
    results = []

    def run(label, expected):
        report = check(source, flux, ['codex-utils-string'], output / label, True,
                       selections, proof_cache=output / 'proof-cache')
        result, = report['packages']
        if result['status'] != expected:
            raise AssertionError((label, result))
        mapping = json.loads((output / label / result['function_map']).read_text())
        invocation, = mapping['invocations']
        active = [f for f in invocation['functions'] if f['status'] in (
            'accepted_with_observed_models', 'check_error')]
        if len(active) != 3 or not invocation['complete']:
            raise AssertionError((label, invocation['counts']))
        results.append({
            'stage': label, 'status': result['status'],
            'elapsed_seconds': result['elapsed_seconds'], 'performance': result['performance'],
            'build_lock_wait_observed': result['build_lock_wait_observed'],
            'functions': [{'def_path': f['def_path'], 'status': f['status']} for f in active],
        })
        write_json(output / 'benchmark.json', {
            'scope': 'Three selected Codex budget bodies, not whole-project speed',
            'cache_scope': 'Constraint reuse; Rust and Flux still process the selected bodies',
            'source': report['source'], 'driver_sha256': report['driver_sha256'],
            'model_sha256': report['model_sha256'], 'results': results,
        })

    try:
        run('cold', 'checked_with_observed_models')
        for repeat in range(3):
            run(f'unchanged-{repeat}', 'checked_with_observed_models')
        for label, (before, after) in replacements.items():
            path.write_bytes(original.replace(before, after))
            run(label, 'proof_failure')
            path.write_bytes(original)
            run(label + '-restored', 'checked_with_observed_models')
        validate_reuse(results)
    finally:
        path.write_bytes(original)


if __name__ == '__main__':
    main()

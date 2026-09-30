#!/usr/bin/env python3
"""Export portable review evidence from bounded Codex corpus runs."""
import argparse
import collections
import gzip
import hashlib
import json
from pathlib import Path
import re


def read(path):
    return json.loads(path.read_text())


def packed(path, value):
    data = json.dumps(value, separators=(',', ':')).encode()
    path.write_bytes(gzip.compress(data, mtime=0))


def export(baseline, retries, output):
    output.mkdir(parents=True, exist_ok=False)
    roots = [baseline, *retries]
    reports = [read(root / 'report.json') for root in roots]
    context = reports[0]
    for report in reports[1:]:
        for key in ('source', 'driver_sha256', 'model_sha256', 'fixpoint_sha256',
                    'rustc', 'solver', 'flags', 'configuration', 'only_check'):
            if report[key] != context[key]:
                raise ValueError(f'Retry changes verification context: {key}')
    selected = {}
    superseded = []
    diagnostics = []
    for root, report in zip(roots, reports):
        for result in report['packages']:
            name = result['package']
            if name in selected:
                superseded.append({'package': name, 'previous_run': selected[name][0].name,
                                   'replacement_run': root.name})
            selected[name] = (root, result)
            diagnostics.append({'run': root.name, 'package': name,
                                'text': (root / result['diagnostics']).read_text()})
    inventory = read(baseline / 'inventory.json')
    if set(selected) != {p['name'] for p in inventory['packages']}:
        raise ValueError('Expected an observation for every inventoried package')
    packages, summaries = [], []
    counts = collections.Counter()
    crashes = collections.defaultdict(list)
    for name, (root, result) in sorted(selected.items()):
        mapping = read(root / result['function_map'])
        log = (root / result['diagnostics']).read_text()
        status = result['status']
        if status == 'checker_or_compiler_crash' and 'failed to run custom build command for `v8' in log:
            status = 'dependency_build_script_crash'
        counts[status] += 1
        match = re.search(r'(?:panicked at |internal compiler error: )([^\n]+)', log)
        if match:
            location = re.sub(r'^/home/[^/]+/\.cargo/registry/src/[^/]+/',
                              'cargo-registry/', match[1])
            crashes[location].append(name)
        row = {
            'package': name, 'outcome': status, 'original_classifier': result['status'],
            'run': root.name, 'elapsed_seconds': result['elapsed_seconds'],
            'mapping_status': mapping['status'], 'invocations': len(mapping['invocations']),
            'item_observations': sum(len(i['functions']) for i in mapping['invocations']),
            'complete_function_inventories': sum(i['inventory_complete'] for i in mapping['invocations']),
            'specification_status': 'needs_review',
        }
        summaries.append(row)
        packages.append(dict(row, function_map=mapping))
    scope = 'Host platform, default features, all available Cargo targets; observations are not proof coverage'
    snapshot = {
        'schema_version': 1, 'scope': scope, 'inventory': inventory,
        'runs': [dict(directory=root.name, report=report,
                      flux_patch=(root / 'flux.patch').read_text(),
                      codex_patch=(root / 'codex.patch').read_text())
                 for root, report in zip(roots, reports)],
        'superseded': superseded, 'packages': packages,
    }
    packed(output / 'codex-function-baseline.json.gz', snapshot)
    packed(output / 'baseline-diagnostics.json.gz', diagnostics)
    overview = {
        'schema_version': 1, 'source': context['source'],
        'configuration': context['configuration'], 'scope': scope,
        'package_count': len(summaries), 'outcomes': dict(counts),
        'item_observations': sum(p['item_observations'] for p in summaries),
        'warning': 'Observations include repeats, tests, and generated code; they are not unique functions or completed contracts',
        'superseded': superseded,
        'crash_groups': [{'location': k, 'packages': v}
                         for k, v in sorted(crashes.items(), key=lambda x: -len(x[1]))],
        'packages': summaries,
    }
    (output / 'baseline-summary.json').write_text(json.dumps(overview, indent=2) + '\n')
    manifest = {p.name: {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                         'bytes': p.stat().st_size}
                for p in sorted(output.iterdir()) if p.is_file()}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({'packages': len(packages), 'outcomes': counts, 'artifacts': manifest}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--retry', type=Path, action='append', default=[])
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    export(args.baseline, args.retry, args.output)

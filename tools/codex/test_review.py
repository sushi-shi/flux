import contextlib
import gzip
import io
import json
from pathlib import Path
import tempfile
import unittest

from export_review import export


class ReviewEvidenceTests(unittest.TestCase):
    def run_fixture(self, root, status='timeout', driver='same', diagnostics=''):
        root.mkdir()
        report = {key: 'same' for key in (
            'source', 'model_sha256', 'fixpoint_sha256', 'rustc', 'solver',
            'flags', 'configuration', 'only_check')}
        report['driver_sha256'] = driver
        report['packages'] = [{'package': 'sample', 'status': status,
                               'diagnostics': 'output.txt', 'function_map': 'map.json',
                               'elapsed_seconds': 1}]
        (root / 'report.json').write_text(json.dumps(report))
        (root / 'inventory.json').write_text(json.dumps({'packages': [{'name': 'sample'}]}))
        (root / 'map.json').write_text(json.dumps({'status': 'incomplete', 'invocations': []}))
        (root / 'output.txt').write_text(diagnostics)
        for name in ('flux.patch', 'codex.patch'):
            (root / name).write_text('')

    def test_retry_preserves_old_evidence_and_replaces_the_package_outcome(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.run_fixture(root / 'base')
            self.run_fixture(root / 'retry', status='proof_failure')
            with contextlib.redirect_stdout(io.StringIO()):
                export(root / 'base', [root / 'retry'], root / 'export')
            summary = json.loads((root / 'export/baseline-summary.json').read_text())
            self.assertEqual(summary['outcomes'], {'proof_failure': 1})
            self.assertEqual(len(summary['superseded']), 1)
            with gzip.open(root / 'export/codex-function-baseline.json.gz') as stream:
                snapshot = json.load(stream)
            self.assertEqual(len(snapshot['runs']), 2)
            self.assertEqual(snapshot['packages'][0]['specification_status'], 'needs_review')

    def test_changed_driver_cannot_supersede_old_observations(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.run_fixture(root / 'base')
            self.run_fixture(root / 'retry', driver='different')
            with self.assertRaisesRegex(ValueError, 'driver_sha256'):
                export(root / 'base', [root / 'retry'], root / 'export')
            self.assertFalse((root / 'export/baseline-summary.json').exists())

    def test_dependency_panic_is_not_labeled_a_checker_crash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.run_fixture(root / 'base', status='checker_or_compiler_crash',
                             diagnostics='failed to run custom build command for `v8 v1`')
            with contextlib.redirect_stdout(io.StringIO()):
                export(root / 'base', [], root / 'export')
            summary = json.loads((root / 'export/baseline-summary.json').read_text())
            self.assertEqual(summary['outcomes'], {'dependency_build_script_crash': 1})


if __name__ == '__main__':
    unittest.main()

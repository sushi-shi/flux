import json
from pathlib import Path
import tempfile
import unittest

from corpus import read_coverage, function_map


class CoverageTests(unittest.TestCase):
    def read(self, events, suffix=''):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'test-1-coverage.jsonl'
            path.write_text(''.join(json.dumps(e) + '\n' for e in events) + suffix)
            return read_coverage(path)

    def events(self):
        return [
            {'event': 'start', 'schema_version': 1, 'crate': 'example'},
            {'event': 'function', 'id': 'a', 'status': 'not_attempted'},
            {'event': 'function', 'id': 'b', 'status': 'trusted'},
            {'event': 'function', 'id': 'c', 'status': 'not_selected'},
            {'event': 'inventory_complete'},
        ]

    def test_crash_keeps_unattempted_and_skipped_functions(self):
        result = self.read(self.events() + [
            {'event': 'result', 'id': 'a', 'status': 'in_progress'},
        ])
        self.assertFalse(result['complete'])
        self.assertEqual(result['counts'], {'interrupted': 1, 'trusted': 1, 'not_selected': 1})

    def test_complete_failed_crate_preserves_individual_outcomes(self):
        result = self.read(self.events() + [
            {'event': 'result', 'id': 'a', 'status': 'check_error'},
            {'event': 'finish', 'success': False},
        ])
        self.assertTrue(result['complete'])
        self.assertFalse(result['crate_check_succeeded'])
        self.assertEqual(result['counts']['check_error'], 1)

    def test_acceptance_is_not_full_specification(self):
        result = self.read(self.events() + [
            {'event': 'result', 'id': 'a', 'status': 'accepted_with_observed_models'},
            {'event': 'finish', 'success': True},
        ])
        self.assertEqual(result['specification_status'], 'needs_review')
        self.assertEqual(result['trust_dependencies'], 'not_collected')
        self.assertEqual(result['counts']['trusted'], 1)

    def test_truncated_or_unknown_results_fail_closed(self):
        for suffix in ('{"event":', '{"event":"result","id":"unknown","status":"accepted"}\n'):
            with self.subTest(suffix=suffix):
                result = self.read(self.events() + [{'event': 'finish', 'success': True}], suffix)
                self.assertFalse(result['complete'])
                self.assertTrue(result['errors'])

    def test_missing_journal_is_not_coverage(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(function_map(Path(directory))['status'], 'incomplete_or_unavailable')

    def test_multiple_invocations_are_not_summed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for pid in (1, 2):
                (root / f'example-{pid}-coverage.jsonl').write_text(''.join(
                    json.dumps(e) + '\n' for e in self.events() + [{'event': 'finish', 'success': True}]
                ))
            result = function_map(root)
            self.assertEqual(len(result['invocations']), 2)


if __name__ == '__main__':
    unittest.main()

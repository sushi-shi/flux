import copy
import json
from pathlib import Path
import tempfile
import unittest

from corpus import performance_evidence, proof_cache_namespace


class IncrementalEvidenceTests(unittest.TestCase):
    def context(self):
        return {
            'driver_sha256': 'driver', 'model_sha256': {'core': 'model'},
            'fixpoint_sha256': 'fixpoint', 'rustc': 'pinned', 'solver': 'z3 pinned',
            'flags': ['strict', 'no-panic'], 'configuration': {'targets': 'lib'},
            'source': {'revision': 'before'}, 'only_check': ['def:a'],
        }

    def test_every_trust_context_change_invalidates_the_namespace(self):
        context = self.context()
        baseline = proof_cache_namespace(context)
        for key in ('driver_sha256', 'model_sha256', 'fixpoint_sha256', 'rustc',
                    'solver', 'flags', 'configuration'):
            with self.subTest(key=key):
                changed = copy.deepcopy(context)
                changed[key] = {'changed': changed[key]}
                self.assertNotEqual(baseline, proof_cache_namespace(changed))

    def test_source_edits_and_expanded_selection_keep_query_level_reuse(self):
        context = self.context()
        before = proof_cache_namespace(context)
        context['source']['revision'] = 'after'
        context['only_check'].append('def:b')
        self.assertEqual(before, proof_cache_namespace(context))

    def test_missing_timings_are_not_zero_cost(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(performance_evidence(Path(directory)), [])

    def test_solver_time_and_cache_hits_are_distinct(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'sample-timings.json'
            path.write_text(json.dumps({
                'total': 20, 'cached_bodies': 2,
                'functions': [{'def_path': 'a', 'time_ms': 18}],
                'queries': [{'task_key': 'a', 'time_ms': 4}],
            }))
            result, = performance_evidence(Path(directory))
            self.assertEqual((result['checker_ms'], result['body_ms'], result['solver_ms']), (20, 18, 4))
            self.assertEqual(result['cached_bodies'], 2)
            self.assertEqual(result['executed_solver_queries'], 1)

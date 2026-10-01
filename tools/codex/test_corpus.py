import unittest

from corpus import classify


class EvidenceTests(unittest.TestCase):
    def test_cargo_success_is_not_verification(self):
        self.assertEqual(classify(0, "Finished dev profile")['status'],
                         "no_verification_evidence")

    def test_ignored_functions_are_not_checked(self):
        result = classify(0, "summary. 7 functions processed: 0 checked; 3 trusted; 4 ignored.")
        self.assertEqual(result['status'], "no_verification_evidence")

    def test_colored_summary_preserves_trust_counts(self):
        result = classify(0, "\x1b[1msummary.\x1b[0m 7 functions processed: 2 checked; 3 trusted; 2 ignored.")
        self.assertEqual(result['status'], "checked_with_observed_models")
        self.assertEqual(result['summaries'], [
            {'processed': 7, 'checked': 2, 'trusted': 3, 'ignored': 2},
        ])

    def test_crash_takes_precedence_over_partial_checks(self):
        log = "summary. 1 functions processed: 1 checked; 0 trusted; 0 ignored.\ninternal compiler error"
        self.assertEqual(classify(101, log)['status'], "checker_or_compiler_crash")

    def test_proof_failure_is_not_a_confirmed_bug(self):
        self.assertEqual(classify(1, "error: refinement type error")['status'], "proof_failure")

    def test_missing_dependency_is_build_failure(self):
        self.assertEqual(classify(101, "failed to download dependency")['status'],
                         "build_or_checker_failure")

    def test_panic_and_overflow_obligations_are_proof_failures(self):
        for message in ("call to index may panic: MightPanic(Transitive)",
                        "arithmetic operation may overflow",
                        "type invariant may not hold (when place is folded)"):
            with self.subTest(message=message):
                self.assertEqual(classify(101, message)['status'], 'proof_failure')

    def test_multiple_compiler_summaries_are_not_deduplicated_or_added(self):
        log = "summary. 1 functions processed: 1 checked; 0 trusted; 0 ignored.\n" * 2
        self.assertEqual(len(classify(0, log)['summaries']), 2)


if __name__ == '__main__':
    unittest.main()

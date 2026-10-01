# Specifications and documentation

- Describe what the code does, the result it produces, and why that behavior matters.
- Specify unchanged state or absent behavior only when there is a concrete reason, such as a caller's proof obligation, a regression, or an externally observable guarantee. Explain that reason alongside the condition.
- For Flux contracts, start with the operation's useful result. For example, appending text should specify `new_text = old_text + appended_text`.
- Keep necessary frame conditions with the main contract. Do not create separate lemmas merely to enumerate fields that remain unchanged.
- Add lemmas that establish useful behavior or discharge an identified proof obligation. Put generic verifier composition and syntax regressions in Flux's tests.
- Measure progress by meaningful behavior specified and verified. Counts of annotations, lemmas, and accepted bodies are supporting diagnostics, not the objective.

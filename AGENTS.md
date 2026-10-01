# Specifications and documentation

- Describe what the code does, the result it produces, and why that behavior matters.
- Specify unchanged state or absent behavior only when there is a concrete reason, such as a caller's proof obligation, a regression, or an externally observable guarantee. Explain that reason alongside the condition.
- For Flux contracts, start with the operation's useful result. For example, appending text should specify `new_text = old_text + appended_text`.
- Keep necessary frame conditions with the main contract. Do not create separate lemmas merely to enumerate fields that remain unchanged.
- Add lemmas that establish useful behavior or discharge an identified proof obligation. Put generic verifier composition and syntax regressions in Flux's tests.
- Start with the implementation and the property a caller needs. Try inferred summaries first; when inference stalls, generate candidate invariants, lemmas, or inline proof steps and check them against the implementation and accepted models.
- Use explicit requires/ensures for stable intent, reusable interfaces, or an identified abstraction gap. Prefer verified inference and generated proofs over asking developers to duplicate function bodies in handwritten annotations.
- Keep inferred current behavior distinct from intended behavior. Preserve accepted requirements across implementation changes; regeneration must not silently change a requirement to match a regression. Generated proofs must establish their conclusions, never assume them.
- Measure progress by meaningful behavior specified and verified. Counts of annotations, lemmas, and accepted bodies are supporting diagnostics, not the objective.

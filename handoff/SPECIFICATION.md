# Draft specification for agent assisted Rust verification

This proposal explores whether AI agents can make refinement types practical for
ordinary Rust development. Agents write code, contracts, lemmas, and proof hints;
humans review the intended behavior and explicit assumptions; a verifier checks
that the implementation satisfies the contracts. The experiment succeeds if this
reduces review effort while catching meaningful mistakes. The application target
is the whole Codex tool: a maintained map of its behavior using our contracts,
invariants, and reusable lemmas, with verification coverage growing across the
codebase. Small examples and selected modules are initial milestones.

Readability, incremental adoption and checking, and reusable lemmas are primary
requirements. A proof system that is effective only after annotating an entire
project, or whose contracts are hard to review, does not meet the goal.

The starting point is our Flux fork. This document records what we want to try,
not features that already exist. It continues the September 29 discussion and the
[Flux handoff](HANDOFF.md). The specification and implementation belong in the
Flux fork. Flowistry integration is a separate possibility.

## Implementation driven development

The working method is to try the next real piece of Codex, observe the failure,
reduce it to a regression, and generalize only what that failure needs. Solve the
missing contract, lemma, library model, or checker capability, then rerun the
original code and retain both positive and negative tests. Repeat across the
whole project. The feature tables below are acceptance ideas, not a waterfall
schedule that postpones applying the tool until its design is complete.

Each iteration should produce executable evidence and a focused stacked PR.
Update the map with what is specified, what is proved, and what still fails.
The stopping target is meaningful coverage of the whole pinned Codex tool;
package inventory and annotation counts alone do not satisfy it.

## Hypothesis and intended workflow

Proofs are often more work to write than to check. Our hypothesis is that agents
can absorb that writing cost, making stronger guarantees affordable. Humans
should be able to understand a contract without understanding the proof machinery.
Whether the checker is reliable and the contracts capture the intended behavior
remain separate questions.

The intended development loop is:

1. A human describes the behavior and reviews the proposed public contracts.
2. An agent implements the behavior and runs the verifier.
3. For an unproved obligation, the agent fixes the code, supplies a checked lemma
   or invariant, or proposes a contract change for review.
4. The verifier checks every proof obligation in the selected scope and reports
   unsupported code and assumptions explicitly.
5. The human reviews changes to behavior, contracts, and assumptions. Proof
   details are available but ordinarily need no line by line review.

Verification covers stated properties. Security, performance, resource use, and
behavior outside those properties can still require implementation review.
An agent must not obtain a passing result by silently weakening a postcondition,
strengthening a precondition, narrowing the checked scope, or adding assumptions.

## Readability as a requirement

A Rust developer should be able to read a contract and explain the accepted
inputs, promised outputs, and possible errors without learning the solver's
representation. Use domain names and small named predicates when an inline
formula becomes hard to follow. Avoid duplicated implementations, numbered enum
tags, and compiler details in the normal review view.

Contracts and lemmas should appear beside the code or have direct source links.
Diagnostics should speak in the same terms as the contract. The review view
should show changed guarantees, assumptions, and coverage, with proof details
available on demand.

Evaluate readability directly in the pilot: can a reviewer correctly describe
each contract and identify an intentionally weakened guarantee? Record confusion
and review time, not just the number of annotation lines.

## Contracts people can read

Contracts should use Rust expressions, names, patterns, and types wherever
possible. The proposed surface is `#[requires(...)]` and `#[ensures(...)]`;
`result` names the return value. These examples describe desired syntax and
semantics, not currently executable examples.

```rust
#[requires(!xs.is_empty() && xs[0] == 5)]
fn consume_first_five(xs: &[i32]) {
    // Implementation checked under the precondition.
}

fn caller(xs: &[i32]) {
    if xs.len() > 5 && xs.iter().all(|x| *x == 5) {
        consume_first_five(xs);
    }
}
```

The branch must establish the callee's precondition: the slice is nonempty and
every element equals five. The fact must flow across function boundaries through
contracts, without requiring the verifier to inline the whole program.

Contract expressions have their own definedness obligations. Short circuiting
must make the index above safe; an unconditional `xs[0]` cannot silently assume
the slice is nonempty. Facts describe a particular state and must be invalidated
or updated when a relevant value changes.

The original odd length example is an essential negative test: `len > 5` does
not imply `len % 2 == 1`. A six element slice is a counterexample. Adding an odd
length check, or knowing the length is exactly five, supplies the missing fact.

## Pattern matching and precise errors

Contracts should express enum variants and payload constraints directly, without
manually assigning numeric tags to variants.

```rust
enum Error {
    NotFound(u32),
    Denied,
    Timeout(u64),
}

#[requires(matches!(e, Error::Timeout(seconds) if seconds > 30))]
fn escalate(e: Error) {
    // Only a timeout greater than thirty seconds reaches this function.
}

#[ensures(matches!(result, Ok(_) | Err(Error::NotFound(_) | Error::Denied)))]
fn lookup(id: u32) -> Result<u32, Error> {
    // Returning Timeout must fail verification.
    todo!()
}
```

The initial implementation should handle variant bindings, payload guards, nested
patterns, and alternative patterns. Contracts must preserve error information
through `match`, `?`, and supported `map_err` calls, accounting for conversions.
Ordinary Rust exhaustiveness rules still apply: an otherwise required fallback
arm can contain `unreachable!()` only when its unreachability is proved.

## Reusing Rust functions in specifications

The long term goal is to use any suitable `const fn` in a specification without
rewriting its logic in a second language. The first experiment supports pure,
terminating functions over integers, booleans, enums, and immutable slices.

```rust
const fn all_fives(xs: &[i32]) -> bool {
    let mut i = 0;
    while i < xs.len() {
        if xs[i] != 5 {
            return false;
        }
        i += 1;
    }
    true
}
```

We want to use `all_fives(xs)` in both executable checks and contracts. The
verifier must relate its body to its logical meaning. Being a `const fn` alone
does not establish termination or freedom from panic. Loops and recursion need
checked invariants and termination arguments before their definitions can be
used as total logical functions. Unsupported cases stay explicit.

Machine integers must retain the semantics of the selected Rust build profile.
Mathematical integers, if provided for proofs, must be a distinct concept with
checked conversions. Bounds, overflow, division by zero, and panic paths cannot
disappear when translating a Rust expression into logic.

Iterator operations also need models. Supporting `const fn` does not by itself
give `.all()` a useful contract. The initial `.all()` experiment uses a pure
predicate over an immutable slice; predicates with side effects need separate
semantics.

## Proofs that agents can supply

Automatic checking is the first attempt. When it cannot discharge an obligation,
an agent can add a local assertion, call a reusable lemma, or provide a loop
invariant. The checker verifies these additions; they are not assumptions.

We want proof annotations at the call site, next to the obligation they discharge.
The exact Rust compatible spelling is open. A first version can use helper
functions and attributes before introducing dedicated proof syntax.

Proof only code must have no observable runtime effects and must not change the
executable result when erased. It cannot mutate executable state, perform I/O,
or justify a logical theorem through nontermination. Facts involving mutation
need explicit before and after states and frame conditions describing what can
change. Rich mutation proofs are deferred beyond the first examples.

Diagnostics should identify the source location, required proposition, available
facts, and missing or unsupported model. Structured output should let an agent
distinguish a failed proof from a timeout, checker crash, or unsupported feature.
A counterexample is useful when available, but a failed proof is not by itself
evidence of a runtime bug.

### Writing and reusing lemmas

A lemma has a name, parameters, preconditions, a proposition it establishes, and
a checked proof body. Calling it requires proving its preconditions and makes
its conclusion available locally. For example, a lemma can establish that
`all_fives(xs) && i < xs.len()` implies `xs[i] == 5`, or that an index produced
by a modeled string operation lies on a UTF-8 boundary.

Agents should be able to discover existing lemmas, write a missing one, prove it,
and reuse it across functions and crates. Lemmas belong in ordinary versioned
source with readable statements and explicit dependencies. A checked lemma is
distinct from an external axiom; replacing its body with `trusted` changes the
trust report. Recursive lemmas require a well founded argument.

The lemma experiment must reuse one lemma at two call sites, reject a false
version of its statement, and invalidate dependent results when its statement or
proof changes. Measure whether the second use needs less agent effort.

## Applying verification incrementally

Start with one function or module in an existing crate. Keep normal Cargo builds
and tests usable while verification coverage grows. Users must be able to choose
the scope, see the remaining gaps, and add another function without first
specifying every dependency in the project.

At a boundary with unverified code, use only facts established by executable
validation, ordinary Rust guarantees within the stated model, or an explicitly
approved external contract. An unknown call does not acquire the needed
postcondition automatically. If outside callers are unchecked, report a verified
body as conditional on its precondition; do not claim that every call is safe.

Adoption proceeds from a baseline report to selected contracts, proofs, and
library models. Each change should show which obligations became verified,
which regressed, and whether scope or trust changed. A project can ratchet its
chosen baseline in CI while still showing unresolved obligations elsewhere.
Deleting a checked function from the scope must remain visible as lost coverage.

Incremental checking is a separate requirement: after an edit, recheck affected
functions, proofs, and dependents instead of solving the entire project again.
Track dependencies through contracts, lemma bodies, models, generic instances,
features, and configuration. Cold and incremental runs must agree on outcomes.

Test adoption on two small modules: begin with one selected function, add a
caller and a shared lemma, then expand to the second module. Change a callee
contract, a lemma, and a trusted model in separate edits. Verify that affected
results are recomputed, unrelated results can be reused, and a clean run produces
the same report. Include a newly unchecked caller to test the coverage boundary.

## Mapping the whole Codex tool

The target is a maintained specification of all first party Rust code in Codex,
including its library and executable crates. Tests, generated code, feature gated
code, and platform specific implementations must be accounted for in the map.
External dependencies, operating system services, network peers, and other
language components form explicit modeled boundaries. Mapping the whole tool
means every area is accounted for; it does not mean an unsupported area is proved.

Build the inventory before narrowing execution to the first tractable functions.
Pin the Codex revision and record the supported feature and platform matrix. Map
crates, modules, entry points, functions, trait implementations, important data
types, and calls across component boundaries. Record unresolved dynamic dispatch
and unavailable configurations as gaps. Stable identifiers and source locations
connect contracts, proof obligations, lemmas, and runtime tests to this inventory.

For each function or family of functions, describe the intended inputs and
outputs, possible errors, relevant state invariants, effects, and conditions on
panics. Use our readable requires and ensures syntax, named predicates, checked
lemmas, and eventually explicit before and after state specifications. Simple
functions can rely on inferred facts or shared contracts; the goal is meaningful
behavioral coverage, not a handwritten annotation on every line.

A function marked verified is verified against identified properties. Maintain
separate status for specification coverage and proof coverage. Distinguish a
missing contract, a reviewed contract, a proved obligation, a conditional result,
a trusted model, unsupported code, a timeout, and a checker defect. An annotation
count or a vacuous postcondition is not evidence that behavior has been specified.

The initial component plan is:

| Area | Contracts and reusable lemmas to develop |
| --- | --- |
| Utility crates | Index bounds, UTF-8 boundaries, sequence contents, arithmetic, and relationships between lengths |
| Parsing and protocol | Validated input shapes, parser progress, precise error variants, and serialization relationships |
| Patch application | Search bounds, hunk applicability, and specified transformations of file contents |
| Core orchestration | Allowed state transitions, request and event relationships, and invariants maintained across calls |
| Execution and I/O boundaries | Validation of requests and responses, explicit modeled effects, failure behavior, and resource lifecycle obligations |
| Async components | State preserved across suspension, cancellation behavior, and synchronization assumptions; extend proof support as required |
| TUI and app server | Layout bounds, protocol state, event handling, and propagation of component contracts to entry points |

Grow two reusable libraries together: general Rust models and lemmas in the Flux
fork, and Codex domain predicates, contracts, and lemmas alongside Codex code in a
dedicated integration checkout. Behavioral requirements must come from intended
semantics, documentation, tests, and human review, not merely from restating what
the current implementation does. An agent must not turn arbitrary external input
into a precondition that unchecked callers are assumed to satisfy.

Work from both ends: specify entry point behavior and domain invariants early,
then discharge the obligations through utility functions and component contracts.
Track those dependencies explicitly so a local proof shows which larger claims it
supports. Pure utilities are the first completed proofs; they are not the limit
of the project. Unsupported features required by Codex become Flux implementation
work in the stack.

### Testing the growing specification

Keep a pinned full project inventory and baseline, then run targeted checks after
each edit and broader checks at stack integration points. Expand the executable
corpus as models and checker support arrive. Preserve unsupported and untested
areas in the report rather than removing them from the denominator. Add tests
for cross crate contract propagation and entry point behavior as coverage grows.

Each component needs representative valid cases, false claims that are rejected,
checked lemmas, and regression tests for known failures. Maintain a mutation
corpus that breaks bounds, error guarantees, state transitions, and input
validation. First show that each mutation changes the claimed behavior, then
check that verification detects the relevant violation. A compiler error or
checker crash is not a detected proof violation. Actual Codex bug claims require
an executable reproducer through the relevant public API.

Review specification quality as well as proofs: exercise weakened guarantees,
inconsistent preconditions, and implementation changes that preserve safety but
violate intended behavior. Test incremental invalidation across crate and lemma
boundaries by comparing cached outcomes with a clean run. Report coverage per
component and property, assumptions, outstanding obligations, checking time, and
human review effort. Define complete milestones against a pinned revision and
configuration matrix, and update the map as Codex changes.

## Trust and verification coverage

The earlier handoff reports two bugs addressed in
[Flux fork PR 1](https://github.com/sushi-shi/flux/pull/1): a missing closure
constraint made `.map()` loop bodies verify vacuously, and missing state models
for string iterators made the generic iterator contract contradictory. The PR
is open as checked on September 30, 2026; its changes are on the handoff branch.
The broader bug for iterator implementations without suitable models remained
open in the handoff. These were bugs in Flux; the prior run reported no confirmed
bugs in Codex.

These failures allowed false claims to be accepted because a loop body became
logically unreachable. Reproducing them is the first gate. We cannot evaluate
the benefits of the workflow using a checker
that still accepts the known false claims in its selected scope.

Unknown iterator implementations must not inherit unproved size or termination
facts. Unsupported operations must produce an explicit coverage gap. An infinite
iterator must not be modeled as a finite sequence merely because it implements
`Iterator`.

Every run should report:

- Which functions and obligations were checked, with verified, unproved,
  unsupported, timeout, and internal error outcomes kept distinct.
- Every trusted contract, unchecked dependency model, assumption, and excluded
  body on which a result depends, including transitive dependencies.
- The code revision, toolchain, verifier, models, configuration, and arithmetic
  semantics used for the run.

The prototype's acceptance cases require all selected obligations to verify, no
coverage gaps, and no unreviewed assumptions. A report may describe a conditional
proof under an approved external contract, but must make that dependency visible.
An intentional panic or unverified external input remains outside a panic freedom
claim unless the contract explicitly accounts for it; marking it trusted does
not make it safe.

Agents can propose library models. Those models must be checked against the
implementation where supported, or remain explicit trusted assumptions. Generated
runtime tests help falsify bad models but cannot prove them sound. Include
negative verification tests that attempt to derive false claims, as well as
tests comparing model predictions with actual execution.

## Experiments and acceptance criteria

Choose these experiments from observed Codex failures. Each keeps its code,
commands, tool versions,
expected outcomes, and measured results so another session can reproduce it.

| Experiment | What we try | Acceptance criterion |
| --- | --- | --- |
| 1. Reproduce the baseline | Run the handoff probes with library models enabled and disabled; reproduce the planted indexing bug and iterator contradictions | False claims are never reported as verified within the selected supported scope; missing models are visible; known failures become regression cases |
| 2. Prove the slice example | Propagate a runtime length check and `.all()` fact into a callee contract | The valid call verifies; an empty slice, a missing element fact, and an unjustified odd length requirement fail; mutation cannot preserve a stale fact |
| 3. Check error contracts | Use variants and payload guards across `match`, `?`, and a modeled error conversion | Allowed paths verify; an unexpected variant or insufficient payload bound fails; no manual variant numbers appear in user code |
| 4. Reflect a function | Use `all_fives` and a small arithmetic helper in contracts, including their loops or recursion where needed | The executable definition supplies the logical meaning; false postconditions fail; nontermination and undefined arithmetic cannot create a theorem |
| 5. Repair a proof with an agent | Hold the intended contract fixed and have an agent supply a missing lemma or invariant at a call site | A previously unproved valid case verifies without new trust or executable changes; a nearby false claim remains unproved |
| 6. Try useful library models | Model UTF-8 slice boundaries and a small set of iterator operations | Safe slicing verifies; an in-bounds byte index inside a multibyte character is rejected; unknown iterators provide no invented facts |
| 7. Adopt and check incrementally | Expand from one function to two small modules and edit shared contracts, lemmas, and models | Coverage and trust changes stay visible; affected results are invalidated; incremental and clean runs agree |
| 8. Try a real module | Select one small parser, truncation, or layout module and triage all selected obligations | Publish coverage, assumptions, proof effort, and reproducible findings; every claimed bug has a triggering input through the relevant public API |

For the string and real module experiments, a UTF-8 slice contract must require
`a <= b && b <= s.len()` as well as boundaries at both `a` and `b`. External
data needs executable validation that establishes useful facts; a specification
cannot manufacture guarantees about input we do not control.

## Measuring whether the idea is worthwhile

Record agent attempts, elapsed time, proof size, contract changes, new assumptions,
and human review time. Compare the same small tasks with ordinary Rust tests and
review. This is initially a pilot, so report individual outcomes and failure
cases without claiming a general productivity improvement.

Measure cold compilation, warm checking after an edit, and solver time separately.
Compare inferred invariants with supplied invariants. Pin the hardware and
toolchain; treat the handoff's earlier timings as historical observations to
reproduce. A proposed initial usability target is a warm feedback loop under one
second for the small examples. Measure larger modules before setting their budget.

Changes to a function, contract, dependency model, or verification configuration
must invalidate affected cached results. Faster checking is useful only when the
result still describes the current program.

Continue if agents can complete the examples with readable contracts, checked
proofs, explicit trust, and manageable review effort. Reconsider the approach if
the success depends on growing trusted assumptions, repeatedly rewriting normal
Rust, or spending more review effort on specifications than the baseline task.
Finding no real bug in the pilot is a valid result.

## Stacked PR workflow

Use the same workflow as the Flowistry fork: small, independently reviewable PRs,
with each dependent branch based on the preceding branch and its PR targeting
that branch. Keep each PR focused on one behavior and its validation. Fix a
regression in the PR that introduced it; put new work and pre-existing gaps in a
new PR instead of growing the earlier one.

The existing root of the stack is PR 1, branch
`fix/iter-adapter-soundness`, targeting `main`. The `handoff` branch preserves
experiments and context; it is not itself an implementation PR. Its useful files
should enter the review stack through a focused setup and specification PR.

The layers below are candidate work, not a fixed order or a list of implemented
or opened PRs. Actual Codex failures determine the next layer. Split a layer
further when needed to keep its diff reviewable.

| Layer | Change | Review and validation focus |
| --- | --- | --- |
| Existing PR 1 | Fix closure constraints and string iterator models | The two known false proofs are rejected; valid iterator cases still work |
| Setup and specification | Bring in this specification, a reproducible environment, and the relevant baseline probes | Reproduce the baseline and identify remaining failures explicitly |
| General iterator soundness | Remove unjustified facts for iterator implementations without models | Unknown and infinite iterators cannot make false propositions verify |
| Scope and coverage | Inventory all Codex components, expose specification and proof status, and support selecting functions for checking | The whole target stays visible while individual components become verified; show conditional guarantees honestly |
| Readable contracts | Introduce Rust expression contracts and enum patterns in small steps | Readable source and diagnostics; positive and negative contract cases |
| Library models | Add the slice, iterator, and UTF-8 facts needed by the examples, in separate PRs where practical | Validate each model and expose its trust requirements |
| Lemmas and local proofs | Support writing, checking, finding, and reusing lemmas at call sites | Reuse a checked lemma; reject false lemmas and unmet preconditions |
| Function reflection | Support a bounded subset of Rust functions in specifications, then extend it | Definedness, termination, and agreement with executable semantics |
| Incremental checking | Reuse results using explicit verification dependencies | Edits invalidate affected results; clean and incremental reports agree |
| Codex integration series | Add domain contracts, invariants, and lemmas component by component, expanding from utilities to the whole tool | Track meaningful specification coverage, proof coverage, cross component guarantees, readability, and review effort |

Flux implementation PRs and Codex contract PRs form linked stacks in their own
repositories. Each Codex integration layer pins the required Flux revision and
links to the checker or model changes it needs. New Codex requirements feed back
into focused Flux PRs. The existing Codex working tree is not the experiment
sandbox.

Every PR description states the behavior it changes, its predecessor, relevant
validation, and remaining limitations. Experimental results belong to the PR
that produces them. Do not describe planned validation as already completed.

The user reviews each PR before it is merged. Squash-merge from the bottom of
the stack into the fork's `main`. After a squash merge, rebase the next branch
onto the updated `main`, excluding its old base commits, and retarget its PR.
Ask before a force-push needed to publish the restacked branch. Keep downstream
branches aligned as the stack advances. Upstream submissions are separate from
review and merging in the fork.

## Implementation scope and open decisions

The working choice is to extend the Flux fork, starting with its existing proof
and contract machinery. Add readable syntax once the baseline and first proof
work reliably. The [Flux Book](https://flux-rs.github.io/flux/) describes the
underlying approach of compile time checking with refinement types.

Before a large front end rewrite, run the slice and call site proof examples
through Verus as a comparison. This is an experiment, not a claim that either
tool already meets this specification.

The first milestone verifies a small, explicitly selected part of Codex while
accounting for the rest in the project map. Later milestones expand contracts
and proofs across the whole tool. Unsafe code, concurrency, async behavior, and
external effects require explicit models or additional proof machinery; they
remain visible obligations until supported. Complete verification of the whole
tool is an ambition to evaluate, not a capability claimed by the initial Flux
prototype.

Decisions to make after the first experiments are the proof syntax, which Rust
functions can be reflected safely, how models are packaged and versioned, and
how much library support a useful real module needs.

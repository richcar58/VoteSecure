# Kernel Work Queue: Issue Drafts K1–K3

**Date:** 2026-07-11
**Status:** Drafts ready to post (BMVS bootstrap plan, Phase 1 item 6)

The three kernel work items below are drafted as complete GitHub issues. Per plan decision D9,
posting them is Rich's action: create one issue per section on `richcar58/VoteSecure`, using the
**Title** line as the issue title and the body as-is (the line references are valid at commit
`3b16d14`, the `v1_3` / `kernel-v1.3-fork.1` lineage). The fork convention that every PR
references an issue (`Closes #N`) then applies to the implementing PRs. This file remains in the
repo as the durable record of the queue's origin; the issues become the live tracking.

---

## K1 — Complete and review all Fiat-Shamir challenge derivations

**Title:** `Complete Fiat-Shamir challenge inputs across all ZK proof derivations`
**Suggested labels:** `security`, `cryptography`
**Priority:** highest-value kernel work; gates all external security claims about systems built
on this library. Requires independent cryptographic review. Upstream contribution intended.

### Problem

The code marks its own Fiat-Shamir challenge derivations as incomplete:

- Four sites in
  `implementations/rust/workspace/protocol/src/trustee_protocols/trustee_application/top_level_actor.rs`
  (lines 1383, 1453, 1553, 1590) carry
  `#[crate::warning("Challenge inputs are incomplete.")]` around the calls that produce and
  verify mix-round shuffles via `shuffle_ciphertexts(ciphertexts, election_pk, &election_context)`.
- `implementations/rust/workspace/cryptography/src/zkp/shuffle.rs` line 596 carries
  `#[crate::warning("Verify that this double hashing set up is ok")]` in the challenge
  derivation, which currently hashes `[pk, w_n, w_prime_n, context]` under domain-separation
  tags; line 296 carries `#[crate::warning("Figure out how this skip(1) behaves")]` in proof
  code whose behavior should be pinned down as part of the same review.

A Fiat-Shamir challenge that does not bind the **entire statement being proven** admits proof
forgery even when the underlying sigma protocol is sound. This exact defect class ("weak
Fiat-Shamir") was found in the SwissPost/Scytl Internet-voting system in 2019, in both its
shuffle proofs and its decryption proofs, permitting in-principle undetectable vote manipulation.
For this library, the shuffle and decryption proofs are what make "counted as recorded"
publicly verifiable — their soundness is the product's core claim.

### Scope

Audit and complete **every** Fiat-Shamir derivation in the crate, not only the marked sites:

1. Terelius-Wikström shuffle challenges (`zkp/shuffle.rs`) — including the `h_generators`
   derivation and every sub-challenge in the permutation-commitment argument;
2. plaintext-equality proofs (`zkp/pleq.rs`) as used by Naor-Yung encryption/validation;
3. discrete-log-equality proofs (`zkp/dlogeq.rs`) as used by distributed decryption;
4. Schnorr proofs (`zkp/schnorr.rs`), currently unused in the protocol but exported;
5. the caller side: the `election_context` assembled in the trustee actor must carry the full
   statement context (election hash, protocol phase, mix round number, ballot style / batch
   identifier, and the acting trustee's identity), so that transcripts cannot be replayed across
   rounds, styles, trustees, or elections.

### Acceptance criteria

- [ ] A written challenge-input specification per proof type: the exact serialized fields, their
      order, and the domain-separation tags — reviewable against `EVS` Protocols 10.3, 10.8,
      and 12.3.
- [ ] Implementation binds exactly the specified inputs; the shuffle's double-hashing structure
      is either justified in a comment with the reviewer's argument or replaced.
- [ ] Negative tests: for each proof type, a transcript that verifies in its original context
      must fail verification when any statement element is substituted (different election hash,
      round, style, trustee, key, or ciphertext list).
- [ ] The four `"Challenge inputs are incomplete."` warnings and the two `shuffle.rs` question
      warnings are removed.
- [ ] Independent cryptographic review sign-off recorded in the PR.
- [ ] Offered upstream to FreeAndFair/VoteSecure.

---

## K2 — Compile on stable: gate the nightly features behind `custom-warnings`

**Title:** `Allow kernel crates to compile on stable Rust (gate nightly features behind custom-warnings)`
**Suggested labels:** `build`, `good first issue`
**Priority:** **blocks BMVS bootstrap Phase 2.** The verifier and product repositories are pinned
to stable Rust (plan decision D4) and consume these crates as git dependencies; a dependency
declaring `#![feature(…)]` compiles only on nightly, so the first `cargo check` against the
kernel fails on stable. Upstream contribution intended.

> **Status: implemented 2026-07-11** (awaiting Rich's commit). Both crate-level gates are
> `cfg_attr`'d on `custom-warnings`; all warning-attribute uses converted to `cfg_attr` form
> where required (statement-position sites and proc-macro attributes on file modules — the
> latter an unstable position the original draft had not anticipated); the conditionally-unused
> `custom_warning` alias import is `cfg`-gated. Verified: `cargo +stable check --workspace`
> passes; `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` (pinned nightly,
> default features) pass; warnings still emitted with `--features custom-warnings` on nightly;
> stable release test suite run recorded in the implementing commit. Sequencing note:
> `kernel-v1.3-fork.1` was signed and pushed at the pre-K2 docs commit (`0fa9215`), and
> published tags are never moved — so K2 (commit `3a100d2`) is the content of
> **`kernel-v1.3-fork.2`**, the Phase 2 consumption point. The issue may still be posted for
> the record, marked as resolved by the implementing commit.

### Problem

`cryptography/src/lib.rs` (lines 9, 11) and `protocol/src/lib.rs` (lines 15, 17) declare
`#![feature(stmt_expr_attributes)]` and `#![feature(proc_macro_hygiene)]` **unconditionally**,
solely so that `#[crate::warning("…")]` attributes can be placed on statements and expressions.
The warning machinery is already feature-gated (`custom-warnings`), and `custom_warning_macro`
itself already uses the correct pattern internally
(`#![cfg_attr(feature = "on", feature(proc_macro_diagnostic, proc_macro_span))]`) — but the
consumer crates do not, so every build requires nightly even when warnings are off.

### Recommended implementation

Mirror the macro's own pattern in its consumers:

1. In both `lib.rs` files:
   `#![cfg_attr(feature = "custom-warnings", feature(stmt_expr_attributes, proc_macro_hygiene))]`.
2. Convert **statement/expression-position** uses to
   `#[cfg_attr(feature = "custom-warnings", crate::warning("…"))]` (built-in `cfg_attr` on
   statements is stable; the proc-macro attribute then only appears when the feature — and the
   nightly gate — is active). Item-position uses (modules, functions) are already stable and can
   stay as-is. Inventory: `grep -rn '#\[crate::warning\|#\[custom_warning::warning' implementations/rust/workspace --include='*.rs'`
   — 62 sites outside the macro crates at `3b16d14`.
3. Verify both configurations:
   - `cargo +stable check --workspace` (default features) succeeds;
   - `cargo +nightly-2026-07-09 check --workspace --features cryptography/custom-warnings`
     still emits the warnings.
4. Consider a CI job for the stable build so the property cannot regress.

### Acceptance criteria

- [ ] Kernel workspace compiles on pinned stable with default features.
- [ ] Warnings still function on nightly with `custom-warnings` enabled.
- [ ] Existing test suite passes in both configurations.
- [ ] Included in tag `kernel-v1.3-fork.2` (the Phase 2 consumption point; `fork.1` was
      published at the pre-K2 docs commit and, per the fork policy, published tags are never
      moved).
- [ ] Offered upstream to FreeAndFair/VoteSecure.

---

## K3 — Expand the DKG threshold-parameter test matrix

**Title:** `Expand threshold parameter coverage in dkgd tests`
**Suggested labels:** `testing`
**Priority:** opportunistic; becomes required before certification evidence is assembled or any
binding use of a threshold configuration the matrix does not cover.

### Problem

`cryptography/src/dkgd/mod.rs` marks its own test module with
`#[crate::warning("Need more threshold parameter combinations")]`. End-to-end trustee protocol
tests currently cover two configurations (3 trustees / threshold 2, and 9 / threshold 5, plus
checkpoint/resume variants in
`protocol/src/trustee_protocols/integration_tests_basic.rs`). Joint-Feldman DKG and the
Lagrange-combination decryption path have threshold-dependent edge behavior that two interior
points do not exercise.

### Recommended implementation

- Extend the `dkgd` unit-test matrix with the boundary and structure cases: `T = 1` (if the
  protocol admits it — document the decision if not), `T = P` (all trustees required),
  `T = ⌈P/2⌉ + 1` (typical majority), small `P` (2, 3) and a larger configuration (e.g.,
  `P = 15, T = 8`).
- Gate expensive configurations behind the existing `long_running_tests` feature.
- Exercise decryption with *different* quorum subsets of size `T` (not always the first `T`
  participants), since Lagrange coefficients differ per subset.
- Remove the warning marker once the matrix and a short coverage rationale (which configurations
  and why) are in place.

### Acceptance criteria

- [ ] Matrix covers the boundary cases above; heavy cases feature-gated.
- [ ] Quorum-subset variation exercised in combination tests.
- [ ] Coverage rationale documented in the test module.
- [ ] The `"Need more threshold parameter combinations"` warning is removed.
- [ ] Offered upstream to FreeAndFair/VoteSecure.

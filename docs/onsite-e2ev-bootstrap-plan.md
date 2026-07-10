# Bootstrap Plan: Implementing the Three-Repository Organization (Option D)

**Date:** 2026-07-09
**Status:** Proposed plan, for review
**Series:** [Feasibility](./onsite-e2ev-feasibility.md) →
[Feature Variations](./onsite-e2ev-feature-variations.md) →
[Kernel Primer](./onsite-e2ev-crypto-kernel.md) →
[Baseline Architecture](./onsite-e2ev-architecture.md) →
[BMVS Architecture](./onsite-e2ev-architecture-bmvs.md) →
[Code Organization](./onsite-e2ev-code-organization.md) → this document

**Review log:** Plan Review 1 (2026-07-09) — acronym changed from BMBS to BMVS ("Ballot Marking
Voting System") across the document series; repository names fixed as `bmvs` and `bmvs-verifier`,
to be created in `~/git` when this review completes; remote-repository governance ratified as D9.
Plan Review 2 (2026-07-09) — added the [Local vs. GitHub appendix](#appendix-local-vs-github-responsibilities)
clarifying where D6, D8, and the kernel work queue are implemented.

This plan sequences the implementation of Option D from the
[code-organization analysis](./onsite-e2ev-code-organization.md): three repositories — the
**kernel** (this fork, pristine and upstream-tracking), the **product monorepo** (`bmvs`), and the
**independent verifier** (`bmvs-verifier`, owner of the published artifact schema). The plan is
organized as six phases with five review gates (M0–M5). Its central engineering strategy is a
**walking skeleton**: prove the three-repo plumbing, the crate layering, and the kernel pinning
with one thin end-to-end election before implementing breadth.

Guiding rule for sequencing: *the verifier repo is bootstrapped before the product repo*, because
the product depends on the verifier's `bmvs-artifacts` schema crate — the dependency arrow the
organization document establishes (product → verifier schema, never the reverse) must exist from
the first commit, or it will never exist.

---

## Phase 0 — Ratify Decisions (gate: M0, together with Phase 1)

Decisions to fix in writing before any repository surgery. Each has a recommended default; review
means accepting or amending these:

| # | Decision | Recommended default |
|---|---|---|
| D1 | Hosting and names | **Ratified 2026-07-09:** repos `bmvs` and `bmvs-verifier`, created in `~/git` when this plan's review completes; remote hosting on the same forge as this fork; private during bootstrap, with the stated intent to open `bmvs-verifier` first (its value is public auditability) |
| D2 | License | Apache-2.0 across all three (continuity with upstream) |
| D3 | Kernel consumption | Pinned git **tag** dependency initially; revisit a private registry only if tag-pinning becomes painful |
| D4 | Toolchains | Kernel: follow upstream but pin the nightly **date**; product + verifier: pinned **stable** (see K2) |
| D5 | Versioning | Kernel tags `kernel-v<upstream>-fork.<n>`; product release trains `bmvs v0.x` (one version = the certified set); verifier independent semver with additive-only schema changes within a major |
| D6 | Conventions carried over | Conventional Commits (incl. `wip`/`cosmetics`), linear history, signed commits on release branches, pre-commit hooks (fmt/check/clippy `-D warnings`), text hygiene — identical across all three repos |
| D7 | Docs home | The `onsite-e2ev-*` series and all future product documents move to `bmvs/docs/`; this fork's `main` stays upstream-shaped (see 1.2) |
| D8 | CI platform | GitHub Actions, mirroring the fork's per-artifact path-filter pattern |
| D9 | Remote governance | **Ratified 2026-07-09:** only Rich pushes to or otherwise modifies remote repositories (including remote creation, tags, and branch operations); all other work is local — edits, and local commits only when requested |

### Phase 0 execution status (2026-07-09)

**Implemented locally** (all content staged, deliberately uncommitted — Rich makes the initial
commits per D9):

- `~/git/bmvs` and `~/git/bmvs-verifier` created (`git init -b main`) — the local half of D1,
  under Rich's explicit authorization.
- In both repositories, the ratifiable defaults are materialized as reviewable files:
  - **D2** — `LICENSE.md`: the upstream dual scheme (Apache-2.0 for code, CC BY-SA 4.0 for
    standalone documentation), copyright line set to "Richard Cardone" *pending confirmation*;
  - **D4** — `rust-toolchain.toml` pinned to **stable 1.97.0** (the stable installed and active
    in the development environment);
  - **D6** — `.pre-commit-config.yaml` (text hygiene + commitlint; the kernel's Rust hooks are
    deliberately deferred until a cargo workspace exists), `.commitlintrc.js` (copied verbatim
    from the fork, including `wip`/`cosmetics`), `.gitattributes` (LF normalization),
    `CONTRIBUTING.md` (full D6 workflow + D9 governance);
  - **D8** — `.github/workflows/run-precommit-hooks.yml`, the first CI workflow, adapted from the
    fork's (Rust setup step returns with the workspace scaffold);
  - **D3 / D5 / D7** — documented as governing-decision tables in each `README.md` (these three
    have no implementable artifact until Phases 2–3).

**Remaining to complete Phase 0 (Rich):**

- [ ] Ratify D2–D8 by reviewing the materialized files above (amendments welcome — each decision
      is now a concrete file diff rather than an abstraction).
- [ ] Confirm or amend the copyright holder line in both `LICENSE.md` files.
- [ ] Review the staged content and make the initial signed commits in both repositories
      (suggested: `chore: bootstrap repository per BMVS plan Phase 0`).
- [ ] Run `pre-commit install && pre-commit install --hook-type commit-msg` in both clones.
- [ ] Create the two private GitHub remotes; add as `origin`; push `main` (D1 remote half, D9).
- [ ] GitHub settings on both repos: branch protection on `main` (require linear history, signed
      commits; add the pre-commit workflow as a required status check after its first run);
      enable Actions with a restricted/pinned actions policy.
- [ ] Confirm the commit-signing public key is present on the GitHub account.

Note: the M0 gate also requires the Phase 1 items (kernel toolchain date pin, kernel tag,
upstream remote + cadence, kernel work queue), which are tracked separately below.

## Phase 1 — Kernel Repo Hygiene (this fork) — gate M0

Goal: this fork becomes exactly what Option D needs it to be — a pristine, upstream-mergeable
kernel with an explicit consumption point — and nothing is lost in the move.

1. **Preserve current work — done (2026-07-09).** The seven-document `onsite-e2ev-*` series is
   committed on the `no-network` branch and pushed by Rich; the investigation history exists in
   git before anything migrates.
2. **Relocate product documents.** When the product repo exists (Phase 3), move the series to
   `bmvs/docs/` (git history preserved via the commit above; a short pointer file can remain on
   the fork branch). The fork's `main` never carries product material.
3. **Pin the toolchain date.** Change `rust-toolchain.toml` from bare `nightly` to
   `nightly-YYYY-MM-DD` (reproducibility requirement). Offer upstream.
4. **Cut the first kernel tag.** Tag the pinned state (e.g., `kernel-v1.2-fork.1`) — the
   consumption point that Phases 2–4 build against. From here on, kernel changes reach the
   product only through a reviewed tag bump.
5. **Set the upstream cadence.** Add the upstream remote, document a monthly `main` sync
   (`git pull --rebase`, fast-forward only per fork conventions), and a policy note: *what lands
   here* — kernel crates, kernel models, kernel docs, all candidates for upstreaming; *what never
   lands here* — anything BMVS-specific.
6. **Open the kernel work queue** (parallel to bootstrap, not blocking it):
   - **K1 — Fiat-Shamir challenge completion** (the `"Challenge inputs are incomplete"` markers).
     The entire series names this the prerequisite for relying on any transported proof. It gates
     *security claims* about milestone outputs, not the bootstrap itself — schedule it as the
     first substantial kernel engineering task, with independent review, and offer it upstream.
   - **K2 — Stable-toolchain enablement**: make `custom_warning_macro` a no-op on stable (or gate
     its use), so downstream product crates can be stable-pinned. Small, upstreamable.
   - **K3 — Threshold test matrix expansion** (the `dkgd` warning), opportunistic.

**M0 exit criteria:** decisions D1–D9 ratified (D1 and D9 ratified in Plan Review 1); series
committed (done); toolchain pinned; kernel tag exists; upstream remote + policy documented.

## Phase 2 — Verifier Repo Bootstrap — gate M1

Goal: `bmvs-verifier` exists with the schema crate the product will build against, and a verifier
skeleton that compiles and checks *something* real.

1. **Scaffold** the repo: workspace with three crates —
   - `bmvs-artifacts`: the published artifact schema (`TallyTranscript`, `BoardSegmentMsg`,
     `DispositionRecord`, chain-head attestation, tabulator report, instance descriptor), with
     explicit schema versioning from the first field. This crate is the contract of the whole
     system; treat every change as a reviewed event.
   - `verifier-core`: check implementations, organized to mirror F13.2's list — chain integrity,
     configuration/trustee signatures, Naor-Yung proofs, disposition arithmetic, shuffle proofs,
     decryption proofs, tally recomputation (IRV rounds), reconciliation identity, card
     accounting closure. Skeleton first: traits + the checks implementable against kernel
     primitives today (hashing, signatures, NY/TW/CP verification are all in the kernel crate).
   - `bmvs-verifier`: thin CLI (`verify <election-record-dir> --report`).
2. **Kernel dependency** by the Phase 1 tag, using only the verification surface.
3. **Fixtures**: hand-built micro-election fixtures (a handful of entries with valid
   hashes/signatures generated by a small fixture tool) to make verifier tests meaningful before
   the simulator exists. Phase 4 replaces these with simulator-generated golden fixtures — the
   circularity (verifier needs fixtures; fixtures come from the product) is broken by starting
   hand-built and small.
4. **CI**: fmt/clippy/deny/vet/nextest; Linux + macOS builds; container image build of the CLI;
   the same commit conventions as everywhere else.

**M1 exit criteria:** `bmvs-artifacts` published (tag) with versioned schema for the five artifact
types; `bmvs-verifier` CLI verifies the hand-built fixture end-to-end for the checks that exist;
CI green.

## Phase 3 — Product Repo Bootstrap — gate M2

Goal: an empty-but-real `bmvs` monorepo where every later step is "fill in a crate," never
"restructure."

1. **Scaffold the workspace** exactly per the organization document:
   `crates/{bmvs-types,bmvs-encoding,bmvs-board,bmvs-transport,bmvs-hal,bmvs-config,bmvs-audit}`,
   `crates/cores/{bmd,tab,ppc,bcs,vca,eas,pbb,tas,trustee}-core`, `bins/*`, `sim/`. All crates
   compile as documented stubs; rustdoc headers state each crate's single responsibility and its
   flow ownership (see traceability below).
2. **Workspace mechanics from day one** (cheap now, expensive to retrofit): populated
   `[workspace.dependencies]` and `[workspace.lints]` (kernel's strict posture inherited);
   `default-members` = cores + sim; profiles (`release` with `lto`, `codegen-units=1`, `strip`,
   `panic="abort"`; `release-audit` with debug info); `.cargo/config.toml` carrying the musl
   targets and the single-threaded model-checking test rule; pinned stable
   `rust-toolchain.toml`; `deny.toml` + `cargo vet` store seeded from the kernel's; pre-commit
   config; CODEOWNERS.
3. **Dependencies**: kernel by Phase 1 tag; `bmvs-artifacts` by Phase 2 tag.
4. **Product-side RDE structure**: `docs/` receives the migrated series (closing Phase 1 step 2);
   `docs/protocol/specs/` opens with stub specs for the new† subprotocols (session activation,
   card lifecycle, tabulator cast, dispositions, segments/attestation) — *specs precede code*, per
   the RDE discipline; `models/tamarin/` holds `card_lifecycle` and `tabulator_cast` model stubs
   plus a composition Makefile that fetches kernel models by the same pinned tag.
5. **Deployment skeletons**: the parameterized cargo-chef Dockerfile; `compose.yaml` with dev
   profiles; `packaging/` with deb/rpm templates and an OS-image build placeholder; a release
   workflow that, on tag, builds every binary `--locked`, produces SBOMs (`cargo auditable` +
   CycloneDX), and has a signing step stubbed pending the key ceremony (Phase 6).
6. **CI**: per-crate path filters; merge-queue full matrix (fmt, clippy `-D warnings`, deny/vet,
   nextest, model-check suite single-threaded, release build of every binary, service-image
   builds).

**M2 exit criteria:** fresh clone + `cargo test` green on stable; CI green; every architecture
component has its named crate stub; docs and model stubs in place; the three-repo dependency
graph (product → kernel tag, product → artifacts tag) builds.

## Phase 4 — Walking Skeleton — gate M3 (the decisive review)

Goal: one thin, honest, end-to-end election through real crate boundaries — proving the layering,
the schema, the kernel pinning, and the development loop before any breadth is attempted.

Scope (deliberately minimal): one polling place, three voters, one plurality contest, mock HAL,
in-process transport, no provisional/early-voting/write-ins.

1. `bmvs-types`: card Z1/Z2 structures, `SessionAuthMsg`, minimal bulletins glue.
2. `bmvs-encoding`: plurality-only rank encoding, exhaustively property-tested (the P6 crate —
   its first tests assert the byte-equality contract).
3. `bmvs-board`: in-memory + file-backed implementation of the kernel `BulletinBoard` trait;
   segment export to `bmvs-artifacts` format.
4. Cores, thinnest viable: `vca-core` (issue card), `ppc-core` (session auth F3.1 + submission
   acceptance wrapping the kernel DBB actor), `bmd-core` (activate/commit wrapping the kernel VA
   actor, mock print), `tab-core` (validate + cast F15.1), `tas/trustee-core` (drive the kernel
   trustee actors — DKG, mix, decrypt — which already run end-to-end in kernel tests),
   `pbb-core` (assemble the election record + `TallyTranscript`).
5. `bmvs-sim`: scripts the whole run — check-in → card → activate → commit → cast (one voter
   takes the challenge path instead) → close → segment → mix → decrypt → publish — then invokes
   **the verifier from the other repo** against the produced record, including the
   reconciliation identity (three tabulator-counted votes vs. crypto tally).
6. Round-trip the fixtures: the simulator's output becomes the verifier repo's golden fixture
   set, replacing Phase 2's hand-built ones.

**M3 exit criteria (demo + review):** `cargo run -p bmvs-sim -- demo.toml` completes; the
independently built `bmvs-verifier` accepts the record; one deliberately corrupted record (one
flipped bulletin, one bad shuffle proof, one totals mismatch) is *rejected* with pinpoint errors;
the whole loop runs in CI. **This is the gate at which the three-repo structure is judged** —
cheap to reshape before it, expensive after.

## Phase 5 — Breadth: Components, Features, Proofs (iterative; gate M4)

Fill out the skeleton along three parallel tracks, in vertical slices (each slice: spec → model →
implementation → simulator scenario → verifier check):

**Track A — flows and components.** Implement the remaining architecture flows per crate,
following the traceability mapping:

| Architecture flows | Owning crates |
|---|---|
| F1.x, F8.1 (registration side) | `vca-core`, `eas-core` |
| F2.1, F14.1 (card lifecycle) | `bmvs-types`, `vca-core`, `bmd-core`, `bmvs-hal` |
| F3.x (activation, commit, challenge legs) | `bmd-core`, `ppc-core` |
| F4.x (check station) | `bcs-core`, `ppc-core` |
| F5.2 (receipts) | `bmd-core` + `bmvs-hal` |
| F6.x (provisioning, L&A) | `eas-core`, `bmvs-config` |
| F7.1, F9.1 (segments, attestations) | `ppc-core`, `bmvs-board`, `pbb-core` |
| F8.2, F12.2 (dispositions) | `eas-core`, `bmvs-artifacts` consumers |
| F10.x, F11.x (ceremony) | `tas-core`, `trustee-core` |
| F13.x (publication/verification) | `pbb-core`; checks in `verifier-core` |
| F15.x (tabulator) | `tab-core` |

**Track B — features**, in the order that maximizes reuse: M-of-N and IRV contests → free-text
write-ins (padded encoding) → dispositions machinery (L&A + spoil + misprint) → provisional
ballots → early voting (windows, daily segments/attestations, checkpoint/resume) → attestation
tooling.

**Track C — proof obligations** (from the BMVS architecture, run *with* their slices, not after):
P1 `card_lifecycle` and P2 `tabulator_cast` Tamarin models as their flows are implemented; P3
ballot-check model extension; P5 reconciliation identities as executable properties in `bmvs-sim`
(and the first candidate when Cryptol work resumes); P6 continuously enforced in `bmvs-encoding`;
P4 (detection-probability bound) and P7 (X6′ privacy/threat-model delta) as review-ready
documents; P8 purge assertions in `bmd-core` tests and the Tamarin state model. Kernel K1
(Fiat-Shamir) must land before any external security claim is made about M4+ outputs.

**M4 exit criteria:** full BMVS baseline + the three Phase-2 features run in the simulator across
multi-site, multi-day scenarios; verifier covers every F13.2 check; P1/P2/P3 models execute with
their executability lemmas; P4/P7 documents reviewed.

## Phase 6 — Deployment Hardening — gate M5

1. Reproducible-build pipeline: vendored dependency archive per release, pinned build container,
   and a CI job that **rebuilds and byte-compares** the release artifacts (the observers'
   rebuild story, rehearsed continuously).
2. Signing ceremony: release key custody defined with the election authority's key-management
   posture; Ed25519 signatures + SBOMs over all binaries and images; container signing for the
   service tier.
3. Native packaging real: deb/rpm for controller/server hosts; prototype signed A/B OS image for
   one device class (BMD first — it exercises HAL, print, and purge paths).
4. HAL: first real device integration behind its feature flag, with hardware-in-the-loop tests on
   a device bench.
5. Cut **release train `bmvs v0.1`**: tagged, reproducible, signed, SBOM'd — the first version an
   instance descriptor could reference.

**M5 exit criteria:** an independent party, given the release tag and the vendored archive,
reproduces bit-identical binaries; a demo polling place (real or bench hardware) runs the
skeleton election natively; central services run from signed containers.

---

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Kernel API churn breaks the product | Consumption only via reviewed tag bumps; adapter surface concentrated in `bmvs-types`/core wrappers so churn is absorbed in one layer |
| Artifact schema churn ripples everywhere | Verifier owns the schema; changes are reviewed events; schema frozen at each milestone; additive-only within a major |
| Circular fixture dependency (verifier needs product output) | Hand-built micro-fixtures first (Phase 2), simulator-generated goldens after M3 |
| Nightly/stable friction | K2 isolates the one nightly dependency; product pinned stable from the first commit |
| Breadth before structure | The M3 walking-skeleton gate exists precisely to force structural judgment while change is cheap |
| Security claims outrunning proofs | K1 gates claims; Track C runs with implementation slices; the M-gates list proof artifacts as exit criteria, not follow-ups |
| Small-team bandwidth | Phases 0–3 are days-to-weeks each; only Phases 5–6 are programs; every gate is a legitimate pause point |

## Rough Sizing (single senior Rust engineer + review; calendar, not effort-certain)

| Phase | Size |
|---|---|
| 0 + 1 (decisions, kernel hygiene, tag) | ~1 week |
| 2 (verifier bootstrap) | 1–2 weeks |
| 3 (product scaffold) | ~1 week |
| 4 (walking skeleton) | 3–5 weeks |
| 5 (breadth) | multi-month program; first M4 cut ~2–3 months after M3 |
| 6 (hardening) | 3–5 weeks, overlappable with late Phase 5 |
| K1 (Fiat-Shamir, kernel) | parallel; independent crypto review adds calendar time |

## Immediate Next Actions (first two weeks, upon approval)

1. ~~Commit the `onsite-e2ev-*` series~~ — done 2026-07-09 (committed and pushed by Rich).
2. Ratify the remaining decisions D2–D8 (D1 and D9 were ratified in Plan Review 1).
3. Pin the kernel toolchain date; cut `kernel-v1.2-fork.1` (Rich pushes the tag, per D9).
4. Rich creates `bmvs-verifier` and `bmvs` in `~/git` (and their remotes, per D9).
5. Scaffold `bmvs-verifier`: `bmvs-artifacts` v0.1 schema + verifier skeleton + CI.
6. Scaffold the `bmvs` workspace + CI; migrate the docs series (Rich pushes).
7. Open the K1 (Fiat-Shamir) kernel issue with the four warning sites enumerated, and schedule
   its review.

---

## Appendix: Local vs. GitHub Responsibilities

*Added in Plan Review 2, clarifying what Phase 0 items D6 and D8 and Phase 1's kernel work queue
entail, and specifically which parts are implemented in the local development environment versus
in GitHub.*

Every mechanism in this plan lives in one of three places:

- **(a) Files in the repository** — authored and tested locally, pushed by Rich (D9); they take
  effect for every clone once landed. This is where most of the substance lives.
- **(b) Per-machine setup** — one-time configuration in each developer's environment (hook
  installation, signing keys, git config).
- **(c) GitHub-side settings and infrastructure** — repository administration and job execution;
  under D9 all of it is Rich's.

The recurring principle: **local mechanisms are conveniences; GitHub mechanisms are the
authoritative gates.** Local hooks can be bypassed (`git commit --no-verify`), so CI re-runs
everything the hooks do (the fork already does this via `run-precommit-hooks.yml`), and branch
protection is what makes those CI jobs binding.

### D6 — Conventions carried over

**Repo files (a).** The fork's `.pre-commit-config.yaml` defines the whole local enforcement
stack: text hygiene (`end-of-file-fixer`, `trailing-whitespace`, `check-yaml`, shebang checks),
`commitlint` with the conventional-commits config (plus the project's `wip`/`cosmetics` types),
and Rust `fmt` / `cargo check` / `clippy -D warnings` against the workspace manifest. Implementing
D6 means copying and adapting this file into `bmvs` and `bmvs-verifier` (new manifest paths,
stable toolchain), plus a `CONTRIBUTING.md` documenting the rebase / fast-forward-only workflow.

**Per-machine (b).** `pip install pre-commit && pre-commit install` once per clone — this writes
the hooks into `.git/hooks`, after which they run at commit time on the developer's machine.
Commit signing is also machine-local: the signing key lives in the developer's environment, and
`git config commit.gpgsign true` + `user.signingkey` make signing happen at commit time; likewise
`git config pull.rebase true`.

**GitHub (c).** Three things: the signing **public** key uploaded to the GitHub account (what
makes commits show "Verified"); branch-protection rules on `main`/release branches — *require
linear history*, *require signed commits*, *require status checks to pass*; and the merge policy.
Note that per the fork's convention the fast-forward merge itself is executed **locally**
(`git merge --ff-only`, then push) because GitHub's UI cannot perform ff-only merges — GitHub's
role is only to refuse non-linear pushes.

### D8 — CI platform

**Repo files (a).** All the substance of CI is version-controlled YAML in `.github/workflows/`.
The fork's pattern (to be mirrored): per-artifact `test-validity-of-*` workflows triggered by
`paths:` filters so unrelated changes don't run each other's jobs; heavier `verify-*`
continuous-verification jobs; Docker-image workflows; `build-release.yml`; and
`run-precommit-hooks.yml` re-running the local hook suite authoritatively. For `bmvs`:
per-crate path-filtered jobs (fmt/clippy/deny/vet/nextest), the single-threaded model-checking
suite, release builds of every binary, and service-image builds. Design rule adopted from the
fork: workflows stay **thin wrappers around Makefile/cargo targets**, so everything CI does is
reproducible in the local environment with the same command (`make ci` locally = the CI job on a
clean machine).

**Execution.** Once pushed, workflows run on GitHub-hosted runners — nothing executes in the
local environment. Future exception: if heavy jobs (Tamarin proofs, long model-checking) outgrow
hosted-runner limits, a *self-hosted runner* is a machine Rich administers that registers with
GitHub — that piece would live in the local environment.

**GitHub (c).** Enabling Actions on the new repos; the Actions permissions policy (restrict and
SHA-pin third-party actions — supply-chain posture consistent with the project); designating
specific jobs as **required status checks** in branch protection (what turns a workflow from
informational into a gate); enabling the merge queue; and repository secrets — ideally none for
building, with registry credentials only if/when CI is permitted to push container images rather
than publication remaining manual under D9.

### Phase 1 — Open the kernel work queue (K1–K3)

This item creates tracked, scoped work items — it does not perform the work.

**Tracking (GitHub, Rich).** The natural home is GitHub Issues on the fork, because the repo
convention requires every PR to reference an issue (`Closes #N`). Creating issues is a remote
modification, so per D9: issue text is drafted locally (scope, exact code sites, acceptance
criteria, review requirements) and Rich posts it. An in-repo work-queue document is the
alternative if keeping everything in-tree is preferred, at the cost of the PR-references-issue
convention.

**The work itself (local, when scheduled).** All three items are local engineering; GitHub's role
is the issue, PR review, CI runs on push, and the eventual `kernel-v1.2-fork.2` tag (pushed by
Rich):

- **K1 — Fiat-Shamir completion:** code edits at the four
  `#[crate::warning("Challenge inputs are incomplete.")]` sites in
  `protocol/src/trustee_protocols/trustee_application/top_level_actor.rs`, plus resolving the
  `"verify that this double hashing set up is ok"` note in `zkp/shuffle.rs` — binding the full
  statement (election hash, public keys, ciphertext lists, round/slot context) into every
  challenge derivation, with tests; independent review; then an upstream PR to FreeAndFair
  (submission by Rich).
- **K2 — stable enablement:** local changes to `macros/custom_warning_macro` (compile to no-ops
  on stable behind a cfg), verified against a pinned stable toolchain locally; a small upstream
  PR candidate.
- **K3 — threshold matrix:** additional T-of-P test combinations in `cryptography/src/dkgd/`
  (plausibly behind the existing `long_running_tests` feature), removing the module's warning
  marker when satisfied.

### Summary

| Item | Local development environment | GitHub |
|---|---|---|
| D6 conventions | Author config files (`.pre-commit-config.yaml`, `CONTRIBUTING.md`); `pre-commit install` per clone; signing keys + git config; perform ff-only merges | Public key on account; branch protection (linear history, signed commits, required checks) |
| D8 CI | Author workflow YAML; run the same make/cargo targets locally; (later) any self-hosted runner | Actions enablement + permissions policy; required-check designation; merge queue; secrets; job execution on hosted runners |
| Kernel work queue | Draft issue text; all K1–K3 code, tests, and local verification | Issues (created by Rich); PR review + CI; upstream PRs and tags (pushed by Rich) |

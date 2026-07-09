# Bootstrap Plan: Implementing the Three-Repository Organization (Option D)

**Date:** 2026-07-09
**Status:** Proposed plan, for review
**Series:** [Feasibility](./onsite-e2ev-feasibility.md) →
[Feature Variations](./onsite-e2ev-feature-variations.md) →
[Kernel Primer](./onsite-e2ev-crypto-kernel.md) →
[Baseline Architecture](./onsite-e2ev-architecture.md) →
[BMBS Architecture](./onsite-e2ev-architecture-bmbs.md) →
[Code Organization](./onsite-e2ev-code-organization.md) → this document

This plan sequences the implementation of Option D from the
[code-organization analysis](./onsite-e2ev-code-organization.md): three repositories — the
**kernel** (this fork, pristine and upstream-tracking), the **product monorepo** (`bmbs`), and the
**independent verifier** (`bmbs-verifier`, owner of the published artifact schema). The plan is
organized as six phases with five review gates (M0–M5). Its central engineering strategy is a
**walking skeleton**: prove the three-repo plumbing, the crate layering, and the kernel pinning
with one thin end-to-end election before implementing breadth.

Guiding rule for sequencing: *the verifier repo is bootstrapped before the product repo*, because
the product depends on the verifier's `bmbs-artifacts` schema crate — the dependency arrow the
organization document establishes (product → verifier schema, never the reverse) must exist from
the first commit, or it will never exist.

---

## Phase 0 — Ratify Decisions (gate: M0, together with Phase 1)

Decisions to fix in writing before any repository surgery. Each has a recommended default; review
means accepting or amending these:

| # | Decision | Recommended default |
|---|---|---|
| D1 | Hosting and names | Same forge/org as this fork; repos `bmbs` and `bmbs-verifier`; private during bootstrap, with the stated intent to open `bmbs-verifier` first (its value is public auditability) |
| D2 | License | Apache-2.0 across all three (continuity with upstream) |
| D3 | Kernel consumption | Pinned git **tag** dependency initially; revisit a private registry only if tag-pinning becomes painful |
| D4 | Toolchains | Kernel: follow upstream but pin the nightly **date**; product + verifier: pinned **stable** (see K2) |
| D5 | Versioning | Kernel tags `kernel-v<upstream>-fork.<n>`; product release trains `bmbs v0.x` (one version = the certified set); verifier independent semver with additive-only schema changes within a major |
| D6 | Conventions carried over | Conventional Commits (incl. `wip`/`cosmetics`), linear history, signed commits on release branches, pre-commit hooks (fmt/check/clippy `-D warnings`), text hygiene — identical across all three repos |
| D7 | Docs home | The `onsite-e2ev-*` series and all future product documents move to `bmbs/docs/`; this fork's `main` stays upstream-shaped (see 1.2) |
| D8 | CI platform | GitHub Actions, mirroring the fork's per-artifact path-filter pattern |

## Phase 1 — Kernel Repo Hygiene (this fork) — gate M0

Goal: this fork becomes exactly what Option D needs it to be — a pristine, upstream-mergeable
kernel with an explicit consumption point — and nothing is lost in the move.

1. **Preserve current work.** Commit the seven-document `onsite-e2ev-*` series (currently
   untracked) on the `no-network` branch so the investigation history exists in git before
   anything migrates. This is the first concrete action of the whole plan.
2. **Relocate product documents.** When the product repo exists (Phase 3), move the series to
   `bmbs/docs/` (git history preserved via the commit above; a short pointer file can remain on
   the fork branch). The fork's `main` never carries product material.
3. **Pin the toolchain date.** Change `rust-toolchain.toml` from bare `nightly` to
   `nightly-YYYY-MM-DD` (reproducibility requirement). Offer upstream.
4. **Cut the first kernel tag.** Tag the pinned state (e.g., `kernel-v1.2-fork.1`) — the
   consumption point that Phases 2–4 build against. From here on, kernel changes reach the
   product only through a reviewed tag bump.
5. **Set the upstream cadence.** Add the upstream remote, document a monthly `main` sync
   (`git pull --rebase`, fast-forward only per fork conventions), and a policy note: *what lands
   here* — kernel crates, kernel models, kernel docs, all candidates for upstreaming; *what never
   lands here* — anything BMBS-specific.
6. **Open the kernel work queue** (parallel to bootstrap, not blocking it):
   - **K1 — Fiat-Shamir challenge completion** (the `"Challenge inputs are incomplete"` markers).
     The entire series names this the prerequisite for relying on any transported proof. It gates
     *security claims* about milestone outputs, not the bootstrap itself — schedule it as the
     first substantial kernel engineering task, with independent review, and offer it upstream.
   - **K2 — Stable-toolchain enablement**: make `custom_warning_macro` a no-op on stable (or gate
     its use), so downstream product crates can be stable-pinned. Small, upstreamable.
   - **K3 — Threshold test matrix expansion** (the `dkgd` warning), opportunistic.

**M0 exit criteria:** decisions D1–D8 ratified; series committed; toolchain pinned; kernel tag
exists; upstream remote + policy documented.

## Phase 2 — Verifier Repo Bootstrap — gate M1

Goal: `bmbs-verifier` exists with the schema crate the product will build against, and a verifier
skeleton that compiles and checks *something* real.

1. **Scaffold** the repo: workspace with three crates —
   - `bmbs-artifacts`: the published artifact schema (`TallyTranscript`, `BoardSegmentMsg`,
     `DispositionRecord`, chain-head attestation, tabulator report, instance descriptor), with
     explicit schema versioning from the first field. This crate is the contract of the whole
     system; treat every change as a reviewed event.
   - `verifier-core`: check implementations, organized to mirror F13.2's list — chain integrity,
     configuration/trustee signatures, Naor-Yung proofs, disposition arithmetic, shuffle proofs,
     decryption proofs, tally recomputation (IRV rounds), reconciliation identity, card
     accounting closure. Skeleton first: traits + the checks implementable against kernel
     primitives today (hashing, signatures, NY/TW/CP verification are all in the kernel crate).
   - `bmbs-verifier`: thin CLI (`verify <election-record-dir> --report`).
2. **Kernel dependency** by the Phase 1 tag, using only the verification surface.
3. **Fixtures**: hand-built micro-election fixtures (a handful of entries with valid
   hashes/signatures generated by a small fixture tool) to make verifier tests meaningful before
   the simulator exists. Phase 4 replaces these with simulator-generated golden fixtures — the
   circularity (verifier needs fixtures; fixtures come from the product) is broken by starting
   hand-built and small.
4. **CI**: fmt/clippy/deny/vet/nextest; Linux + macOS builds; container image build of the CLI;
   the same commit conventions as everywhere else.

**M1 exit criteria:** `bmbs-artifacts` published (tag) with versioned schema for the five artifact
types; `bmbs-verifier` CLI verifies the hand-built fixture end-to-end for the checks that exist;
CI green.

## Phase 3 — Product Repo Bootstrap — gate M2

Goal: an empty-but-real `bmbs` monorepo where every later step is "fill in a crate," never
"restructure."

1. **Scaffold the workspace** exactly per the organization document:
   `crates/{bmbs-types,bmbs-encoding,bmbs-board,bmbs-transport,bmbs-hal,bmbs-config,bmbs-audit}`,
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
3. **Dependencies**: kernel by Phase 1 tag; `bmbs-artifacts` by Phase 2 tag.
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

1. `bmbs-types`: card Z1/Z2 structures, `SessionAuthMsg`, minimal bulletins glue.
2. `bmbs-encoding`: plurality-only rank encoding, exhaustively property-tested (the P6 crate —
   its first tests assert the byte-equality contract).
3. `bmbs-board`: in-memory + file-backed implementation of the kernel `BulletinBoard` trait;
   segment export to `bmbs-artifacts` format.
4. Cores, thinnest viable: `vca-core` (issue card), `ppc-core` (session auth F3.1 + submission
   acceptance wrapping the kernel DBB actor), `bmd-core` (activate/commit wrapping the kernel VA
   actor, mock print), `tab-core` (validate + cast F15.1), `tas/trustee-core` (drive the kernel
   trustee actors — DKG, mix, decrypt — which already run end-to-end in kernel tests),
   `pbb-core` (assemble the election record + `TallyTranscript`).
5. `bmbs-sim`: scripts the whole run — check-in → card → activate → commit → cast (one voter
   takes the challenge path instead) → close → segment → mix → decrypt → publish — then invokes
   **the verifier from the other repo** against the produced record, including the
   reconciliation identity (three tabulator-counted votes vs. crypto tally).
6. Round-trip the fixtures: the simulator's output becomes the verifier repo's golden fixture
   set, replacing Phase 2's hand-built ones.

**M3 exit criteria (demo + review):** `cargo run -p bmbs-sim -- demo.toml` completes; the
independently built `bmbs-verifier` accepts the record; one deliberately corrupted record (one
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
| F2.1, F14.1 (card lifecycle) | `bmbs-types`, `vca-core`, `bmd-core`, `bmbs-hal` |
| F3.x (activation, commit, challenge legs) | `bmd-core`, `ppc-core` |
| F4.x (check station) | `bcs-core`, `ppc-core` |
| F5.2 (receipts) | `bmd-core` + `bmbs-hal` |
| F6.x (provisioning, L&A) | `eas-core`, `bmbs-config` |
| F7.1, F9.1 (segments, attestations) | `ppc-core`, `bmbs-board`, `pbb-core` |
| F8.2, F12.2 (dispositions) | `eas-core`, `bmbs-artifacts` consumers |
| F10.x, F11.x (ceremony) | `tas-core`, `trustee-core` |
| F13.x (publication/verification) | `pbb-core`; checks in `verifier-core` |
| F15.x (tabulator) | `tab-core` |

**Track B — features**, in the order that maximizes reuse: M-of-N and IRV contests → free-text
write-ins (padded encoding) → dispositions machinery (L&A + spoil + misprint) → provisional
ballots → early voting (windows, daily segments/attestations, checkpoint/resume) → attestation
tooling.

**Track C — proof obligations** (from the BMBS architecture, run *with* their slices, not after):
P1 `card_lifecycle` and P2 `tabulator_cast` Tamarin models as their flows are implemented; P3
ballot-check model extension; P5 reconciliation identities as executable properties in `bmbs-sim`
(and the first candidate when Cryptol work resumes); P6 continuously enforced in `bmbs-encoding`;
P4 (detection-probability bound) and P7 (X6′ privacy/threat-model delta) as review-ready
documents; P8 purge assertions in `bmd-core` tests and the Tamarin state model. Kernel K1
(Fiat-Shamir) must land before any external security claim is made about M4+ outputs.

**M4 exit criteria:** full BMBS baseline + the three Phase-2 features run in the simulator across
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
5. Cut **release train `bmbs v0.1`**: tagged, reproducible, signed, SBOM'd — the first version an
   instance descriptor could reference.

**M5 exit criteria:** an independent party, given the release tag and the vendored archive,
reproduces bit-identical binaries; a demo polling place (real or bench hardware) runs the
skeleton election natively; central services run from signed containers.

---

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Kernel API churn breaks the product | Consumption only via reviewed tag bumps; adapter surface concentrated in `bmbs-types`/core wrappers so churn is absorbed in one layer |
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

1. Commit the `onsite-e2ev-*` series on `no-network` (nothing else in this plan should precede
   securing the existing work).
2. Ratify D1–D8 (a one-hour review against the table above).
3. Pin the kernel toolchain date; cut `kernel-v1.2-fork.1`.
4. Create `bmbs-verifier`; land `bmbs-artifacts` v0.1 schema + verifier skeleton + CI.
5. Create `bmbs`; land the workspace scaffold + CI; migrate the docs series.
6. Open the K1 (Fiat-Shamir) kernel issue with the four warning sites enumerated, and schedule
   its review.

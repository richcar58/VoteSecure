# Feature Variation Points for the On-Site E2E-V Product Line

**Date:** 2026-07-03
**Status:** Investigation / product-line design exploration
**Companion to:** [On-Site E2E-V Feasibility Assessment](./onsite-e2ev-feasibility.md)

This document catalogs candidate variation points (VPs) for an on-site, end-to-end verifiable
voting system built on the VoteSecure cryptographic core and implemented as a product line, per the
Rigorous Digital Engineering refinement methodology described in
[Refinements Among High-Level Models](./papers/refinements_among_high_level_models/refinements_paper.tex).
Ranked-choice voting and accessibility are treated as committed features; the remaining features
are assessed for cost, benefit, and — critically — their impact on provable protocol correctness.

Each variation point lists: the features (alternatives or options) that could be bound at that
point, when the choice is bound, benefits, costs, and a **proof impact** rating:

- **None** — variation lives entirely above or beside the verified protocol; existing models and
  proofs apply unchanged.
- **Parametric** — existing proofs quantify over the choice (or can, with modest generalization);
  no new protocol model is needed.
- **Compositional** — a new or modified subprotocol model is required, but it composes with the
  unchanged kernel (the Tamarin `subprotocols/` + `compositions/` structure supports exactly this).
- **Foundational** — changes the cryptographic assumptions or the shape of the core argument;
  requires substantially new proofs. These features should be treated as separate research tracks,
  not configuration options.

---

## 0. How to Vary Without Losing Provable Correctness

The product line should be organized around an **invariant kernel** that is proven once and present
in every instance:

- Naor-Yung (CCA2) ballot encryption under a threshold election key,
- the verifiable Joint-Feldman DKG (secret shares + public check values),
- the hash-chained, signed bulletin board,
- the Terelius-Wikström mix with proofs,
- threshold decryption with Chaum-Pedersen (dlogeq) correctness proofs.

(A component-by-component primer on this kernel — plain-English definitions, general
applications, and proof-methodology guidance — is in
[The Cryptographic Kernel, Explained](./onsite-e2ev-crypto-kernel.md).)

VoteSecure's architecture already provides the three mechanisms that make variation safe:

1. **Parameterized foundations.** The `Context` trait
   (`cryptography/src/context.rs`) abstracts group, hasher, RNG, and signature scheme; ciphertext
   width is a `const` (`protocol/src/cryptography.rs`, `BALLOT_CIPHERTEXT_WIDTH`); DKG threshold
   parameters are runtime values. The Cryptol models mirror this with group/encoder interfaces
   (`models/cryptography/cryptol/Algebra/`), and Tamarin's symbolic models are curve-agnostic.
   Choices at these points are *parametric* — one proof covers the family.
2. **Composable subprotocols.** The Tamarin models are deliberately modular
   (`models/cryptography/tamarin/subprotocols/` combined via m4 in `compositions/`). Optional
   features that add or replace a subprotocol (a new check-in mechanism, a provisional-ballot
   flow) get their own model and a new composition — *compositional* proof cost, bounded and
   incremental.
3. **Layers above the protocol.** Ballot semantics, languages, accessibility rendering, transport,
   and storage all sit above (or beside) the message-level protocol. The protocol sees only a
   `Ballot { ballot_style, rank }` and opaque ciphertexts. Variation here has *no* protocol proof
   impact, though it needs its own (simpler, self-contained) correctness evidence — chiefly
   bijectivity of encodings.

**Verifiable feature binding.** VoteSecure threads `election_hash` — the hash of the signed
election configuration — through every protocol message, and all trustees endorse that
configuration during setup. The product line should exploit this: the **selected protocol instance
descriptor** (which features are enabled, which crypto suite, which tally method, contest
definitions, languages, thresholds) belongs *inside* the election configuration. Then the feature
selection itself is cryptographically committed, trustee-endorsed, and publicly visible, and every
verifier knows exactly which protocol instance it must check. Misconfiguration becomes a detectable
integrity failure rather than a silent variant mismatch.

---

## A. Cryptographic Foundation Variation Points

### VP-A1: Cryptographic suite

**Binding:** compile time (a `Context` instantiation per profile)
**Features:**
- *Ristretto255 + Ed25519 + SHA-3/512* (current default) — fastest, misuse-resistant, no cofactor
  pitfalls.
- *NIST P-256 + ECDSA + SHA-2 family* (already implemented as an alternate group) — required for
  FIPS 140-3 alignment and plausibly for U.S. federal certification; common in EU procurement.
- *Larger NIST curves (P-384/P-521)* — the Cryptol algebra layer already models these; the Rust
  side would need group implementations.

**Benefits:** jurisdiction-by-jurisdiction certifiability without touching protocol logic; the
"one protocol, several suites" story is easy for evaluators to review.
**Costs:** each suite needs test vectors, benchmarks, and (for FIPS) validated primitive modules;
maintaining two profiles roughly doubles low-level test surface. Low-Medium.
**Proof impact:** **Parametric.** Tamarin models are symbolic; Cryptol models are already
group-generic. The assurance case argues suite adequacy per profile.

### VP-A2: Trustee and threshold configuration

**Binding:** election setup (runtime values in the signed configuration)
**Features:**
- *t-of-n threshold selection* (already implemented and tested at multiple configurations).
- *Trustee composition policies* — party representatives, civil-society observers, officials
  (procedural, recorded in the configuration).
- *HSM-backed trustee keys* (optional hardening; key operations behind a signing/decryption
  interface).
- *Proactive share refresh* for long custody periods (see VP-D3).

**Benefits:** matches statutory trustee arrangements across jurisdictions (e.g., party-balanced
boards vs. independent commissions); HSM support materially strengthens certification.
**Costs:** threshold flexibility is essentially free; HSM integration is a Medium engineering cost
(abstracting the trustee private-key operations); share refresh is High (new protocol).
**Proof impact:** threshold — **Parametric** (proofs quantify over t, n). HSM — **None**
(implementation detail below the model). Share refresh — **Compositional** (new subprotocol model).

### VP-A3: Ballot capacity (ciphertext width and rank size)

**Binding:** compile time today (`BALLOT_CIPHERTEXT_WIDTH` derived from the serialized ballot
size); could become a per-election-family build profile.
**Features:** small ballots (single u128 rank, current), wide ballots (larger padded arrays over
product groups — the `NYCiphertext<Ctx, W>` machinery already exists), per-contest ciphertexts
(one ciphertext per contest instead of one per ballot).

**Benefits:** supports jurisdictions with very long ballots (U.S. general elections routinely
exceed what a single u128 encodes, especially with RCV rankings); per-contest ciphertexts enable
contest-level tally variation (see VP-B3 hybrid).
**Costs:** widening is mechanical (Low); per-contest ciphertexts change bulletin entries, mixing
batches, and the ballot-style bookkeeping (Medium-High) and increase proof/verification runtime
roughly linearly in contest count.
**Proof impact:** widening — **Parametric** (proofs are width-generic). Per-contest ciphertexts —
**Compositional** (mix-input structure changes; the "what constitutes one ballot" invariant must be
restated). Uniform ciphertext shape *within a mix batch* is a hard constraint regardless (see
cross-tree constraints).

### VP-A4: Post-quantum readiness

**Binding:** roadmap decision, then compile time
**Features:**
- *PQ signatures on bulletin-board and trustee messages* (ML-DSA alongside Ed25519) — protects
  integrity/authenticity against future quantum forgery; the discrete-log privacy of ElGamal
  ballots is unaffected.
- *Full PQ ballot privacy* (lattice-based encryption + PQ mixnet proofs) — protects against
  harvest-now-decrypt-later of published ciphertexts, relevant where ballot secrecy is legally
  perpetual (e.g., German constitutional expectations).

**Benefits:** long-horizon secrecy compliance; procurement talking point.
**Costs:** PQ signatures — Medium (larger messages, dual-signature plumbing). PQ mixnet —
research-grade; verifiable shuffles for lattice ciphertexts are immature. Very High.
**Proof impact:** PQ signatures — **Parametric** (signature scheme is a `Context` slot). PQ
privacy — **Foundational**. Recommend: adopt PQ signatures as an optional feature; treat PQ privacy
as out of scope for the product line's first generation and mitigate via publication policy
(VP-D6: publish commitments centrally, restrict bulk ciphertext export).

---

## B. E2E-V Mechanism Variation Points

### VP-B1: Cast-as-intended mechanism

**Binding:** product configuration (subsystem presence) + election setup
**Features:**
- *Benaloh challenge at a dedicated check station* (baseline; current Ballot Check subprotocol with
  the BCA on a separate, authority-controlled device).
- *Voter-supervised spoil-and-compare with VVPAT*: the machine prints the plaintext paper record;
  the voter compares by eye; challenge = spoil the pair and revote. Paper is the audit artifact.
- *Both* (defense in depth): crypto challenge available on request; paper always printed.
- *Return codes* (Norwegian/Swiss style): per-voter pre-computed code sheets confirm choices.

**Benefits/costs:**
- Check station: strongest software-independence argument for the *electronic* record; costs one
  device class per polling place, voter time, and poll-worker training. Medium cost.
- VVPAT comparison: familiar to U.S. certification (and effectively mandatory in several
  jurisdictions — see VP-D5); costs printers, paper custody, and accessibility care (audio
  readback of the printed record for non-visual verification). Medium cost.
- Return codes: requires a pre-election per-voter secure channel (mail) and a heavyweight code
  ceremony; designed for *remote* voting and a poor fit at supervised polling places. High cost,
  little on-site benefit. **Recommend excluding from the on-site product line.**

**Proof impact:** Benaloh check — already modeled (**None** beyond kernel). VVPAT — **None** at the
protocol level (procedural evidence enters the assurance case, not the Tamarin models). Return
codes — **Compositional-to-Foundational** (new setup ceremony + new secrecy arguments); another
reason to exclude.

### VP-B2: Recorded-as-cast publication model

**Binding:** product configuration + jurisdiction policy
**Features:**
- *Tracker receipt + post-close central publication* (baseline): voter takes a printed tracker;
  controllers upload signed board segments after close; voters check later.
- *Precinct chain-head attestation*: at close (or at intervals), the controller prints the current
  bulletin-board head hash; poll workers post it publicly at the precinct and it is included in
  signed close-out paperwork; observers photograph it. Binds the local board to the published board
  with near-zero technology.
- *In-precinct read-only mirror*: a display/kiosk on the polling-place LAN lets a voter confirm
  their tracker is on the local board before leaving (no external connectivity involved).
- *Multi-mirror publication with equivocation detection*: after close, several independent parties
  (parties, press, NGOs) mirror the published board; mirrors gossip and compare heads.
- *Witness cosigning*: bulletin entries or checkpoints countersigned by additional keys (e.g., two
  poll workers' tokens) rather than the controller key alone.

**Benefits:** each feature narrows the window in which a corrupted controller or central authority
could rewrite history; attestation and mirrors are cheap and highly legible to observers.
**Costs:** attestation — Low (printing + procedure). Mirror kiosk — Low-Medium. Multi-mirror
infrastructure — Medium (publication formats, mirror tooling). Cosigning — Medium (key management
for poll-worker tokens).
**Proof impact:** attestation, mirrors — **None** (they strengthen assumptions the models already
make about board consistency). Cosigning — **Compositional** but small (signature-set checks on
bulletins). All are additive; none touch the kernel.

### VP-B3: Counted-as-recorded tally method

**Binding:** election setup, per contest (if per-contest ciphertexts, VP-A3) or per election
**Features:**
- *Verifiable re-encryption mixnet + full decryption* (baseline; implemented). Supports every
  ballot semantics including RCV and write-ins; produces plaintext ballots for downstream rules.
- *Homomorphic aggregation* (exponential ElGamal; decrypt only per-option sums). No individual
  ballot is ever decrypted — a stronger privacy posture and a much cheaper verification burden —
  but only for bounded-sum contests (plurality, approval, M-of-N). Incompatible with RCV and
  write-ins.
- *Hybrid*: homomorphic for simple contests, mixnet for RCV/write-in contests (requires
  per-contest ciphertexts).

**Benefits:** homomorphic mode shrinks the trusted computation and the published artifact
dramatically for the common case; hybrid gives each contest the cheapest sound method.
**Costs:** homomorphic mode needs exponential encoding, bounded-sum decoding, and per-ciphertext
range/well-formedness proofs (new ZKP: disjunctive Chaum-Pedersen or similar) — Medium-High
implementation. Hybrid doubles the verifier's job and the documentation burden — additional Medium.
**Proof impact:** mixnet — kernel (**None**). Homomorphic — **Compositional** (new aggregation
subprotocol + new ballot well-formedness proof obligations; reuses ElGamal, dlogeq, DKG as-is).
Hybrid — the union, plus a composition argument that the two pipelines partition the ballot
correctly. Since RCV is committed, the **mixnet path must exist in every instance**; homomorphic
mode is an optimization feature, not a replacement — sensible as a *later* increment.

### VP-B4: Verification artifacts and independent verifiers

**Binding:** product configuration (always-on core, optional extras)
**Features:**
- *Standard election record bundle* (versioned schema: configuration, board, mix transcripts,
  decryption proofs) — should be a kernel obligation, not an option.
- *Reference verifier* (shipped) and *verifier specification* enabling third-party
  implementations; ideally two independently authored verifiers before first binding use.
- *Voter-facing verification tiers*: tracker lookup only; tracker + local chain-inclusion proof;
  full transcript re-verification for institutions.
- *Observer export at precinct close*: signed board copy on write-once media for party observers.

**Benefits:** E2E-V's public-verifiability claim is only as strong as the tooling that lets
outsiders exercise it; a fixed artifact schema is also what makes the *product line* auditable
(the schema carries the instance descriptor from §0).
**Costs:** schema + reference verifier — Medium; second verifier — Medium (ideally externally
funded/authored); observer export — Low.
**Proof impact:** **None** on the protocol; substantial *positive* impact on the assurance case
(verifier spec becomes the concrete statement of what "counted as recorded" means per instance).

---

## C. Ballot and Election Semantics Variation Points

These all live in the encoding layer above `Ballot { ballot_style, rank }`. Their correctness
evidence is bijectivity and capacity proofs for encodings (well-suited to Cryptol or property-based
Rust tests), not protocol proofs.

### VP-C1: Contest types

**Binding:** election setup (contest definitions in the configuration)
**Features:** plurality (single choice); M-of-N / approval / bloc; ranked-choice (IRV/STV —
committed); score/range and cumulative voting; referenda and ballot measures; straight-ticket
device; fusion/cross-endorsement listings (same candidate under multiple parties).

**Benefits:** contest-type coverage *is* jurisdictional coverage — this VP does more for
international applicability than any other.
**Costs:** each type costs an encoder/decoder, capacity analysis against VP-A3, rendering and
accessibility work, and downstream tally rules (IRV/STV elimination logic operates on decrypted
plaintexts and needs its own tested implementation — STV in particular is notoriously
jurisdiction-specific: Meek vs. Gregory variants, tie-breaking rules). Low per simple type;
Medium-High for STV families.
**Proof impact:** **None** on the protocol (mixnet path handles all of them). Encoding bijectivity
obligations per type; IRV/STV tabulation logic needs its own verification (a good Cryptol or
model-checking target) since it runs *after* the cryptographic guarantees end.

### VP-C2: Write-in candidates

**Binding:** election setup, per contest
**Features:** disabled; constrained write-ins (registered write-in candidates only — selection
from a list, encodes like a normal option); free-text write-ins.

**Benefits:** legally required in many U.S. jurisdictions; constrained mode captures most of the
legal need at a fraction of the cost.
**Costs:** free text is the expensive one: variable-length data must be padded to fixed width so
every ciphertext in a mix batch is shape-identical (otherwise the mix leaks which ballots carry
write-ins); capacity must be budgeted (VP-A3); text normalization/adjudication happens post-mix.
Medium-High. Constrained write-ins: Low.
**Proof impact:** constrained — **None**. Free-text — **Parametric** on width plus one new
invariant ("all ballots of a style pad to identical shape") that the encoding layer must prove and
the mix-input checks should enforce.

### VP-C3: Blank votes, abstention, and spoil semantics

**Binding:** election setup + jurisdiction policy
**Features:** implicit undervote (blank allowed, uncounted); *explicit* blank/protest vote counted
and reported separately (required or customary in several countries); explicit "none of the
above"; overvote prevention vs. overvote-recorded-as-spoiled (some jurisdictions require honoring
a voter's right to spoil); ballot cancellation flow at the machine before submission.

**Benefits:** cheap features with outsized legal importance; explicit-blank reporting is a hard
requirement in some European and Latin American jurisdictions.
**Costs:** Low — encoding slots and tally-report categories.
**Proof impact:** **None**; purely encoding/reporting. One care point: "spoiled by checking"
(Benaloh) vs. "spoiled by voter choice" must be distinguishable on the board so turnout accounting
stays exact — a bulletin-type addition, **Compositional** but trivial.

### VP-C4: Multi-language ballots and ballot-style management

**Binding:** election setup
**Features:** single language; multi-language rendering (U.S. Voting Rights Act §203 obligations;
Canada, Belgium, Switzerland, India multilingual requirements); per-voter language preference at
session activation.

**Benefits:** legally mandatory in many target jurisdictions; also an accessibility feature.
**Costs:** Low-Medium (rendering, audio assets per language, translation QA).
**Proof impact:** **None** — *if and only if* the design rule is enforced that language is a
rendering concern and **never** part of the encoded selection or the ballot style. Encoding
language into styles would fragment mix batches into small anonymity sets (see constraint X3). This
rule should be stated in the domain model and checked by construction in the encoding layer.

### VP-C5: Accessibility (committed)

**Binding:** product configuration (device capabilities) + per-session voter preference
**Features:** audio ballot with tactile controller; adjustable display (contrast, magnification,
timing); switch/sip-and-puff input; seated/curbside operation; plain-language mode; **accessible
verification** — audio readback at the check station, large-print/braille-annotated tracker
receipts, accessible verifier web tooling post-election.

**Benefits:** legal necessity (ADA/HAVA in the U.S., EN 301 549 in the EU) and the moral core of
the requirement; accessible *verification* specifically is where E2E-V systems usually fall short —
doing it well is a differentiator.
**Costs:** Medium-High, almost entirely in host-application UI/hardware, sustained usability
testing with disabled voters (the feature model already requires public accessibility-testing
reports). Protocol cost ~zero.
**Proof impact:** **None** on the protocol. One structural note: accessibility must not create a
distinguishable ballot population (e.g., audio-session ballots encoding differently) — same
constraint discipline as VP-C4.

---

## D. Jurisdictional and Operational Variation Points

### VP-D1: Voter check-in and session authorization

**Binding:** product configuration; one feature per deployment
**Features (all replace the Internet-era authentication subprotocol):**
- *Printed activation code* from the registration device, consumed by the voting machine.
- *Smartcard/token* issued at check-in, inserted at the machine (Belgian/Estonian-style token
  ergonomics).
- *Poll-worker console activation*: worker activates a specific machine for the next voter with a
  ballot style (STAR-Vote pattern); nothing crosses via the voter.
- *Biometric-assisted check-in* on the registration device only (as used in India/Brazil), never
  reaching the voting machine.

**Benefits:** matches local law and equipment culture; codes are cheapest; smartcards resist code
theft/reuse and support ergonomic re-issue; console activation removes all voter-carried artifacts.
**Costs:** codes — Low; smartcards — Medium (card lifecycle, readers); console activation — Medium
(pairing protocol between console and machines); biometrics — cost lives in the registration
system, out of protocol scope, but carries privacy/legal risk that must be firewalled from the
ballot path.
**Proof impact:** **Compositional** — this is a new subprotocol in every case (the feasibility
assessment already scopes it); model each mechanism as a variant of one "session authorization"
subprotocol with a common interface to the DBB (`AuthVoterMsg` shape), so downstream kernel proofs
see a single abstraction. This is the highest-value place to spend Tamarin effort.

### VP-D2: Provisional ballots

**Binding:** product configuration (U.S. deployments effectively require it — HAVA)
**Features:** none; *pended cryptogram* — the ballot is encrypted and posted with a "provisional"
bulletin type, excluded from the mix until adjudication, then included or formally excluded with a
signed, published disposition.

**Benefits:** legally mandatory for U.S. use; the cryptographic version is *better* than paper
provisionals — inclusion/exclusion is publicly accounted for.
**Costs:** Medium-High: new bulletin types, an adjudication workflow with signed dispositions, and
extension of the mix-input completeness check (the trustee mixing spec's check #4 — "a valid
explanation exists for any cast cryptogram not in the cryptogram list" — becomes a structured,
machine-checkable disposition record rather than prose).
**Proof impact:** **Compositional** — extends the mix-input justification invariant; well worth
modeling because it closes what is otherwise a manual gap in counted-as-recorded.

### VP-D3: Voting period model

**Binding:** election setup
**Features:** single-day (baseline); multi-day early voting; both with per-day close-out
attestations (VP-B2) and custody procedures.

**Benefits:** early voting is standard in much of the U.S. and growing elsewhere.
**Costs:** Medium — daily chain-head attestations, secure overnight custody procedures, crash-safe
board persistence and resumption (the trustee side already has checkpoint/resume; the controller
side needs the equivalent). Long custody periods strengthen the case for VP-A2 share refresh and
for HSMs.
**Proof impact:** mostly **None** (procedural); board persistence/resume deserves a small model
extension (**Compositional**, minor) to show the chain invariant survives restarts.

### VP-D4: Results reporting granularity

**Binding:** jurisdiction policy in the election configuration
**Features:** centralized totals only; per-precinct reporting (legally required across most of the
U.S.); per-precinct with *minimum anonymity-set thresholds* (small precincts merged into
aggregation groups before decryption).

**Benefits:** precinct reporting is non-negotiable in many jurisdictions and is also an audit
feature (comparing precinct totals against check-in counts).
**Costs:** Low-Medium — mixing/decryption batches keyed by reporting group (the mixing structures
are already keyed by `BallotStyle`; reporting groups generalize the key).
**Proof impact:** **Parametric** on the batch partition, plus one privacy-side obligation: the
configuration validator must enforce the anonymity-set floor (constraint X4). The threat model
should add small-precinct linkage explicitly.

### VP-D5: Paper record and audit integration

**Binding:** product configuration + jurisdiction policy
**Features:** electronic-only (where lawful); VVPAT printed at the machine (retained at precinct —
never leaves, per the feature model's Supervised Vote Receipt Freedom requirement); post-mix ballot
printing for legacy tabulation (the original CONOPS flow); risk-limiting audit support —
*batch-level comparison* between paper and the cryptographic tally.

**Benefits:** paper is the certification path in the U.S. (and a de facto constitutional
requirement in e.g. Germany after the 2009 judgment); RLA compatibility lets the cryptographic and
statistical audit regimes reinforce each other.
**Costs:** VVPAT — Medium (printers, jams, custody, accessibility of the paper check). RLA
integration — Medium (batch manifests, tally exports).
**Proof impact:** **None** on the protocol, with one bright line: do **not** create ballot-level
linkage between a paper record and a specific ciphertext (it would let an insider with paper access
break secrecy of the electronic record). Batch-level linkage only — this belongs in the domain
model as an explicit anti-requirement.

### VP-D6: Data protection, retention, and publication policy

**Binding:** jurisdiction policy in the election configuration
**Features:** full public board (ciphertexts + proofs, baseline); commitment-only publication
(hashes public, ciphertext bodies held by the authority and released to accredited auditors —
trades public verifiability for harvest-now-decrypt-later caution, see VP-A4); configurable
retention/destruction schedules for off-board artifacts (GDPR alignment — personal data stays in
the registration system, never on the board, so the immutable board holds only pseudonymous
material by construction).

**Benefits:** GDPR and national election-records laws vary widely; making publication/retention a
declared, signed policy keeps instances honest about the trade-offs.
**Costs:** Low-Medium (policy plumbing, redaction tooling).
**Proof impact:** **None** on integrity proofs; commitment-only publication *weakens the public
verifiability claim* and the instance descriptor must say so loudly — a legitimate product-line
variant, but the assurance case forks here.

### VP-D7: Pre-election testing and machine attestation

**Binding:** product configuration (recommend kernel-mandatory)
**Features:** logic-and-accuracy (L&A) test mode with *provably excluded* test ballots (test-flag
bulletin type; excluded from the mix with signed disposition, same machinery as VP-D2); software
attestation (measured boot / signed manifests recorded in the election configuration); public
software-hash publication.

**Benefits:** L&A testing is statutory nearly everywhere; making test-ballot exclusion
cryptographically accountable removes a classic attack surface (test mode on election day).
**Costs:** Low-Medium (reuses the pended-cryptogram machinery; attestation depends on hardware
platform choice).
**Proof impact:** **Compositional**, shared with VP-D2 (one "excluded from mix with signed
disposition" mechanism serves both).

### VP-D8: Participation records (compulsory-voting jurisdictions)

**Binding:** jurisdiction policy
**Features:** none (baseline); signed participation record (voter marked as having voted) exported
from the *registration* system — never derived from the ballot path.

**Benefits:** required for compulsory-voting countries (Australia, Belgium, Brazil, much of Latin
America).
**Costs:** Low — it is a registration-system feature; the only product-line obligation is the
firewall rule.
**Proof impact:** **None**, provided the constraint that participation is attested by check-in,
not by bulletin-board presence (linking the two would erode the pseudonym separation).

---

## Cross-Tree Constraints

These are the Clafer-style constraints that keep arbitrary feature combinations from silently
breaking correctness or privacy. They belong in `models/feature-model/` alongside the instance
descriptor of §0.

| # | Constraint | Reason |
|---|---|---|
| X1 | `HomomorphicTally ⟹ ¬RCV ∧ ¬FreeTextWriteIns` (per contest) | Exponential ElGamal aggregates bounded sums; rankings and text cannot be summed. |
| X2 | `RCV ∨ FreeTextWriteIns ⟹ MixnetTally` (per contest) | Plaintext ballots are required for elimination rounds and adjudication. RCV is committed, so the mixnet is kernel. |
| X3 | `MultiLanguage ⟹ LanguageIndependentEncoding` | Language must be rendering-only; encoding it fragments anonymity sets and can deanonymize. Same rule for accessibility modes (VP-C5). |
| X4 | `PrecinctReporting ⟹ AnonymitySetFloor(k)` | Small reporting groups leak individual votes; the configuration validator must merge below-threshold groups. |
| X5 | `FreeTextWriteIns ⟹ UniformPaddedWidth(style)` | All ciphertexts in a mix batch must be shape-identical or the mix leaks write-in presence. |
| X6 | `VVPAT ⟹ PaperRetainedAtPrecinct ∧ ¬BallotLevelPaperCryptogramLink` | Receipt freedom (feature model) and electronic-record secrecy (VP-D5). |
| X7 | `EarlyVoting ⟹ DailyChainAttestation ∧ (CustodyProcedures ∨ ShareRefresh)` | Multi-day custody widens the tamper window; attestation and refresh close it. |
| X8 | `ProvisionalBallots ∨ LandATestMode ⟹ SignedDispositionRecords` | Mix-input completeness (mixing spec check #4) must be machine-checkable, not prose. |
| X9 | `CommitmentOnlyPublication ⟹ ReducedPublicVerifiabilityClaim` | The instance descriptor and assurance case must downgrade the E2E-V claim explicitly. |
| X10 | `ParticipationRecords ⟹ DerivedFromCheckInOnly` | Bulletin-board-derived turnout would link pseudonyms to identities. |
| X11 | `∀ instances: Kernel` (Naor-Yung, DKG, chained BB, TW mix, threshold decryption with proofs) | The invariant kernel is not a feature; every instance carries it and its proofs. |

---

## Recommended Baseline Instance and Phasing

**Baseline (first product-line instance):** Ristretto255 suite (P-256 profile tracked for
certification); t-of-n trustees per local statute; Benaloh check station **plus** VVPAT; tracker
receipts + precinct chain-head attestation + post-close central publication with at least one
independent mirror; mixnet tally (kernel) for all contests; plurality + M-of-N + IRV contest types;
constrained write-ins; explicit-blank support; multi-language rendering; activation-code check-in;
single-day voting; per-precinct reporting with anonymity floor; L&A test mode with signed
dispositions; standard election-record bundle + reference verifier.

**Phase 2 (compositional additions, each with its own subprotocol model):** provisional ballots;
smartcard and console-activation check-in variants; early voting; STV families; free-text
write-ins; homomorphic aggregation for simple contests; PQ signatures; observer export and second
verifier.

**Deliberately excluded from the product line:** return codes (remote-voting fit only); ballot-level
paper↔cryptogram linkage (privacy); PQ ballot privacy (research track); any feature bound at
runtime outside the signed election configuration (unverifiable variance).

**Priority order for proof investment:** (1) complete the kernel's known gaps (Fiat-Shamir
challenge inputs) before any variation work; (2) session-authorization subprotocol family (VP-D1) —
it is new in every instance; (3) signed-disposition/mix-input-completeness mechanism (VP-D2/D7) —
it converts the weakest prose assumption in the mixing spec into a checked invariant; (4) encoding
bijectivity proofs per contest type (VP-C1/C2); (5) homomorphic-tally composition if and when Phase
2 reaches it.

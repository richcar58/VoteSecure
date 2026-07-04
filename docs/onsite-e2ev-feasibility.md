# Feasibility Assessment: VoteSecure as the Basis for an On-Site E2E-V Voting System

**Date:** 2026-07-03
**Status:** Investigation / feasibility analysis
**Scope:** Evaluates whether the VoteSecure cryptographic core (this repository) is a suitable
foundation for a traditional, on-site (polling place) electronic voting system, written in Rust,
that guarantees end-to-end verifiable (E2E-V) election results.

---

## The Proposed System

The target system differs from VoteSecure's original end-to-end verifiable *Internet* voting
(E2E-VIV) mission in its deployment model:

1. Voting takes place at polling places administered by an election authority with complete
   control of all polling locations, central locations, election workers, and election equipment.
2. Voters use voting machines at polling places to make choices and cast ballots.
3. Voting machines are **never** connected to the Internet or any wireless or public network.
4. Voting machines may be connected by a wired connection to a controller device at their polling
   place. The controller administers the election at that polling place and is **never** connected
   to the Internet or any wireless or public network *while the polls are open*.
5. Voter registration devices verify voters and assign ballot styles at polling places.
   Registration devices are **never** connected to voting machines or controller devices.
6. The election authority maintains tabulation, auditing, and reporting equipment at central
   locations. Information from controller devices is transmitted to central locations via secure
   network and/or physical media *only after the polls have closed*.
7. The system guarantees E2E-V election results:
   1. Each voter can verify their ballot was **cast as intended** — the encrypted version matches
      their actual choices.
   2. Each voter can verify their ballot was **recorded as cast** — the encrypted version appears
      correctly on a public bulletin board.
   3. Anyone can verify that all recorded ballots were **counted as recorded** — the final tally
      correctly includes every legitimate vote.

## Recommendation

**Feasible and advantageous, with one significant protocol redesign and several system layers to
build.** VoteSecure's cryptographic core is exactly the standard E2E-V machinery (threshold
ElGamal, verifiable mixnet, Benaloh-style ballot checking, hash-chained bulletin board),
implemented in pure Rust with no networking dependencies, and its trustee-side protocols already
assume an air-gapped deployment. The on-site constraints *strengthen* the security story: the
threats VoteSecure was designed to survive (malware-riddled phones, cloud servers under active
attack, the open Internet) largely disappear, while the E2E-V verification machinery carries over
intact. The one part that must be redesigned rather than reused is voter authentication, which is
currently built around a third-party Internet identity vendor.

---

## 1. What VoteSecure Actually Provides

The repository is a **protocol library, not a voting system** — a fact that works in favor of
adaptation. From [`protocol/src/lib.rs`](../implementations/rust/workspace/protocol/src/lib.rs):

> This library provides state machine implementations of the e-voting protocol for all
> participants. It is designed to be integrated into host applications that will provide the
> necessary networking and user interface layers.

This claim holds up against the dependency graph: neither crate depends on any networking, TLS, or
async runtime library. The `cryptography` crate pulls only curve arithmetic (`curve25519-dalek`,
`p256`), hashing (`sha3`), signatures (`ed25519-dalek`), and RNG; the `protocol` crate adds only
actor/model-checking infrastructure (`stateright`, `ascent`, `enum_dispatch`). Every participant
(Voting Application, Digital Ballot Box, Election Administration Server, Ballot Check Application,
Trustee Application, Trustee Administration Server) is a pure state machine consuming `ActorInput`
messages and emitting outputs (see
[`participants/voting_application/top_level_actor.rs`](../implementations/rust/workspace/protocol/src/participants/voting_application/top_level_actor.rs)).
**The host application supplies the transport.** A wired polling-place LAN, a USB sneakernet, or a
serial link are all equally valid transports — the library doesn't know or care.

The cryptographic inventory (all implemented, all following the Verificatum EVS draft by Wikström,
cited as "EVS Protocol N.M" throughout the rustdoc):

| Component | Location | Role |
|---|---|---|
| ElGamal + Naor-Yung (CCA2) encryption | `cryptography/src/cryptosystem/` | Ballot encryption with proof of well-formedness |
| Pedersen-style verifiable DKG + threshold decryption | `cryptography/src/dkgd/` | t-of-n election key; no single party can decrypt |
| Chaum-Pedersen dlogeq proofs | `cryptography/src/zkp/dlogeq.rs` | Proves each partial decryption is correct |
| Plaintext-equality proofs | `cryptography/src/zkp/pleq.rs` | Naor-Yung ciphertext validity |
| Terelius-Wikström proof of shuffle | `cryptography/src/zkp/shuffle.rs` | Verifiable re-encryption mixnet |
| Hash-chained, signed bulletin board | `protocol/src/participants/digital_ballot_box/bulletin_board.rs` | Tamper-evident append-only record; storage-agnostic trait |
| Groups: Ristretto255, NIST P-256, product groups | `cryptography/src/groups/` | Product groups give width-W ciphertexts for large ballots |
| Ed25519 signatures, SHA-3/SHA-512 | `cryptography/src/utils/` | Message authenticity throughout |

Engineering hygiene is well above typical research code: 747 test functions including Stateright
model-checking of protocol interleavings and 3-of-2/9-of-5 threshold end-to-end runs,
`cargo deny`/`cargo vet` supply-chain checks, Miri, fuzzing targets, `unsafe_code = "forbid"`, and
`unwrap_used = "deny"` in the crypto crate.

## 2. Why the On-Site Model Fits — Mostly Better Than the Original Target

**The trustee side needs no conceptual change at all.** VoteSecure's threat model already mandates
that the Trustee Application and Trustee Administration Server "must not be connected to the
Internet at any point during the system's lifetime" and defines an air-gapped trust zone where data
enters and exits "exclusively by physical devices"
([`threat-model.tex`](../models/threat-model/threat-model.tex), Air-Gapped Network trust zone). The
[CONOPS](./conops/conops.md) mix-and-decrypt flow — close the election, move encrypted ballots by
removable storage into the air gap, jointly mix, verify, and threshold-decrypt — is precisely
requirement 6 above (central tabulation, transfer only after polls close). This is the hardest,
most security-critical machinery in any E2E-V system, and it is inherited unchanged.

**The voting side maps cleanly onto the polling-place topology:**

| VoteSecure participant | On-site equivalent |
|---|---|
| Voting Application (mobile app) | Voting machine ballot-marking software |
| Digital Ballot Box + local bulletin board | Controller device at each polling place |
| Election Administration Server | Controller (pre-provisioned centrally before poll open) |
| Ballot Check Application | Dedicated check station at the polling place |
| Trustee Application + TAS | Central tabulation facility, air-gapped (unchanged) |
| Authentication Service | **Replaced** — see §4 |

The on-site constraints remove the scariest rows of VoteSecure's threat model. The CONOPS assumes
voting devices with "applications (e.g., TikTok) installed that could be leveraged by a
nation-state threat actor" — on-site voting machines are authority-controlled, purpose-built, and
never networked. The DDoS, network-attack, and consumer-device-malware threat classes that the
threat model flags as "unmitigatable with today's commodity Internet technology" simply don't
apply. Meanwhile the E2E-V machinery still protects against the threats that *remain* in a polling
place — corrupted voting-machine software, a corrupted controller, and insider manipulation during
tabulation — because the whole point of E2E-V is that the equipment need not be trusted.

Notably, the feature model already anticipates supervised deployment: it contains requirements like
"Supervised Vote Receipt Freedom" governing paper proofs at polling stations
([`e2eviv.cfr`](../models/feature-model/e2eviv.cfr)), reflecting its Council of Europe requirement
sources. The project's own "protocol family" framing — variants specialized per election type —
explicitly invites this kind of adaptation.

## 3. How VoteSecure's Protocols Deliver the Three E2E-V Requirements

**Cast as intended → Ballot Submission + Ballot Check subprotocols (Benaloh challenge).** The
voting machine encrypts the ballot under the election public key using Naor-Yung and *commits* to
that encryption by submitting it before knowing whether the voter will cast or check it. If the
voter chooses to check, the voting machine discloses the encryption randomizers to a *separate*
device (the Ballot Check Application), which independently decrypts and displays the choices.
Because the machine cannot predict which ballots will be audited, any machine that alters votes is
caught with probability growing in the number of checks. A checked ballot is spoiled; the voter
re-votes. In the on-site setting the BCA becomes a dedicated check station on the polling-place
LAN — device separation is physical and authority-controlled, which is *stronger* than the original
two-apps-on-adjacent-phones design.
(Specs: [ballot-submission-spec.md](./protocol/specs/ballot-submission-spec.md),
[ballot-check-spec.md](./protocol/specs/ballot-check-spec.md),
[ballot-cast-spec.md](./protocol/specs/ballot-cast-spec.md).)

**Recorded as cast → Ballot Submission/Cast subprotocols + the Public Bulletin Board.** Every
accepted ballot is appended to a hash-chained, DBB-signed bulletin board; the **ballot tracker**
returned to the voter is the hash of that bulletin entry
([ballot-submission-spec.md](./protocol/specs/ballot-submission-spec.md), Phase 3). The voter keeps
the tracker (printed receipt at the polling place — it reveals nothing about vote content). One
deployment delta: since the controller is offline while polls are open, the voter's tracker check
against the *published* board happens after close, when controllers upload their board segments to
the central authority for publication (requirement 6 permits exactly this). The hash chain makes
post-hoc tampering between local recording and central publication detectable, and equivocation is
detectable by comparing the published chain against trackers in voters' hands.

**Counted as recorded → Trustee Mixing + Trustee Decryption subprotocols, publicly verifiable.**
After close, trustees bring a verified snapshot of the bulletin board into the air gap. Naor-Yung
proofs are checked and ciphertexts stripped to ElGamal; each trustee in turn re-encrypts and
permutes the full set, producing a **Terelius-Wikström proof of shuffle**; then a quorum produces
partial decryptions, each with a **Chaum-Pedersen dlogeq proof** of correctness against that
trustee's public key share. The complete transcript — input ciphertexts, every mix round with its
proof, every partial decryption with its proof, all trustee-signed — is published alongside the
bulletin board. *Anyone* can then re-verify: every ballot on the board entered the mix, no ballot
was added/dropped/modified (shuffle proofs), and the plaintext tally follows from the ciphertexts
(decryption proofs). Vote privacy holds as long as one mixing trustee is honest; tally integrity
holds even if *all* of them are malicious, because the proofs would fail.
(Specs: [trustee-mixing-spec.md](./protocol/specs/trustee-mixing-spec.md),
[trustee-decryption-spec.md](./protocol/specs/trustee-decryption-spec.md).)

Underpinning all three: the **Setup and Election Key Generation subprotocols**
([setup-spec.md](./protocol/specs/setup-spec.md),
[election-key-gen-spec.md](./protocol/specs/election-key-gen-spec.md)) establish trustee PKI (all
trustees sign the election configuration) and run the verifiable DKG so the election key is never
held by any single party — the root of trust for the whole election.

One design consequence worth appreciating: because VoteSecure uses a **mixnet** rather than
homomorphic tallying (the ElectionGuard approach), it supports arbitrary ballot semantics —
including ranked-choice — since full plaintext ballots emerge from the mix. The `Ballot.rank: u128`
encoding hints this was a design goal. For an election authority that may face RCV requirements,
that is a real advantage.

## 4. What Must Be Redesigned or Built

**Voter authentication is the one genuine protocol redesign.** The current subprotocol
([voter-authentication-spec.md](./protocol/specs/voter-authentication-spec.md)) drives a
third-party identity vendor over TLS/JSON APIs and issues voter pseudonyms remotely — meaningless
in a polling place, and requirement 5 (registration devices never connect to voting machines or
controllers) rules out simply rehosting it. The good news: the *downstream* interface is narrow.
What the DBB actually needs is an `AuthVoterMsg` binding a voter pseudonym + session verifying key
+ ballot style, authorized by the EAS. In the on-site design, poll-worker check-in on the
registration device authorizes a voting session, and the authorization crosses the air gap between
registration device and controller via the voter — a printed activation code, smartcard, or token
that the voting machine consumes to bind a fresh session key to a ballot style and anonymous
pseudonym. This is the well-trodden STAR-Vote pattern. It is a new subprotocol to specify, model,
and implement, but it *replaces* the most fragile part of the original design (remote identity
proofing) with physical procedure, and it is a simplification.

**System layers the library deliberately leaves to the integrator:**

1. **Ballot semantics.** `Ballot` is `{ ballot_style: u16, rank: u128 }` — contest/candidate
   structures, ballot manifests, and the encoding of selections into rank values are explicitly
   "handled at a higher level"
   ([`elections.rs`](../implementations/rust/workspace/protocol/src/elections.rs)). This is a
   substantial, correctness-critical layer (encode/decode must be bijective and tested
   exhaustively).
2. **Transport + provisioning.** Wired-LAN message framing between machines and controller,
   controller-to-central publication after close, and pre-election provisioning of election
   configuration and keys onto controllers.
3. **Public verifier.** E2E-V is only as good as independent verification; a standalone verifier
   tool (ideally more than one, separately authored) is needed to check the published board +
   mix/decryption transcript.
4. **Paper.** The CONOPS prints decrypted ballots post-mix for tabulation compatibility. A
   polling-place system almost certainly wants a voter-verified paper record printed *at the
   machine* as well — orthogonal to the cryptography, but central to certification (VVSG 2.0) and
   to risk-limiting audits, and it interacts with receipt-freeness rules already captured in the
   feature model (paper proof must not leave the polling station).

## 5. Maturity Caveats

These do not change the recommendation, but should be budgeted for:

- **The formal verification is less complete than the repo's framing might suggest.** The Tamarin
  models are protocol *specifications* with executability lemmas; security lemmas are
  "opportunistic" and not all proven ([Tamarin README](../models/cryptography/tamarin/README.md)).
  The Cryptol effort was explicitly paused in mid-2025 ("our statement of work does not promise
  formal verification" — [cryptol-specs.md](./cryptol-specs.md)), Isabelle contains only a Schnorr
  completeness example, and the assurance case is a skeleton. The RDE scaffolding is a genuine head
  start for a certification campaign — but the proofs are mostly still to be done.
- **Known-incomplete spots are honestly marked** via the `#[warning]` macro: "Challenge inputs are
  incomplete" at four sites in the trustee top-level actor
  ([`trustee_protocols/trustee_application/top_level_actor.rs`](../implementations/rust/workspace/protocol/src/trustee_protocols/trustee_application/top_level_actor.rs))
  deserves particular attention — under-specified Fiat-Shamir challenge inputs are precisely the
  bug class behind the SwissPost/Scytl findings. Also `dkgd` is marked unoptimized, the shuffle has
  open performance and "verify that this double hashing set up is ok" notes, and some Miri tests
  fail on Stacked Borrows.
- **Crate maturity:** versions 0.2.x, nightly-only toolchain, a pinned `generic-array`. Fine for
  development; worth hardening before anything binding.
- Forking means **upstream's future assurance work doesn't automatically flow to the fork** — plan
  for an independent cryptographic review of the variant, especially the new authentication
  subprotocol and the Fiat-Shamir completions.

## 6. Verdict

Use it. The parts of an E2E-V system that are genuinely hard to get right — verifiable DKG, CCA2
ballot encryption, the Terelius-Wikström mixnet, threshold decryption with correctness proofs, and
the tamper-evident bulletin board — are implemented, tested, model-checked, and specified here, in
Rust, under Apache-2.0, with a design (transport-agnostic actors, storage-agnostic bulletin board,
air-gapped trustee ceremonies) that fits the on-site architecture with remarkably little friction.
The work ahead is: (1) specify and implement a polling-place authentication/session-authorization
subprotocol, (2) build the ballot-semantics, transport, publication, and paper layers, (3) complete
the marked gaps (Fiat-Shamir inputs foremost) and extend the Tamarin/assurance artifacts to cover
the variant. That is a much smaller and much safer project than building the cryptographic core
from scratch — and it inherits an RDE evidence trail that most alternatives can't offer.

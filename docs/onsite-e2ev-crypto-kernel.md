# The Cryptographic Kernel, Explained: Five Components of an On-Site E2E-V System

**Date:** 2026-07-04
**Status:** Investigation / explanatory primer
**Series:** [Feasibility Assessment](./onsite-e2ev-feasibility.md) →
[Feature Variation Points](./onsite-e2ev-feature-variations.md) → this document

The feature-variation catalog defines an **invariant kernel** — five cryptographic components
present in *every* instance of the proposed on-site E2E-V product line, proven once and reused
everywhere. This document explains each component for readers who are not cryptographers: what it
is, where the same idea is used outside of voting, where a VoteSecure-based system uses it, and how
we gain confidence that it is correct — including what the various kinds of "proof" actually mean.

Each component section is layered. It opens in plain English with an analogy, and ends with a
**Technical detail** subsection containing code paths, protocol references, and proof specifics.
Readers who only need the concepts can stop at each Technical detail heading and lose nothing
essential.

Throughout, `EVS` citations refer to the [Verificatum EVS draft
textbook](https://github.com/verificatum/evs-draft) by Douglas Wikström, which the Rust
implementation cites (as "EVS Protocol/Definition *N.M*") in its documentation.

## Vocabulary

Four terms are enough to read everything below:

- **Public-key encryption.** A scheme with two mathematically linked keys: anyone holding the
  *public* key can lock (encrypt) a message; only the holder of the *private* key can unlock
  (decrypt) it.
- **Digital signature.** The mirror image: the holder of a private *signing* key can stamp a
  message, and anyone holding the public *verifying* key can confirm the stamp is genuine and the
  message unaltered.
- **Cryptographic hash.** A fingerprinting function: any data, of any size, is reduced to a short
  fixed-size fingerprint. Changing even one bit of the data produces a completely different
  fingerprint, and nobody can find two different inputs with the same fingerprint.
- **Zero-knowledge proof (ZKP).** A mathematical receipt that some claim is true ("this ciphertext
  was decrypted correctly," "these two sealed boxes hold the same contents") that reveals *nothing
  else* — not the secret used, not the contents involved.

## How the Five Components Fit Together

| Kernel component | Primary role in E2E-V |
|---|---|
| 1. Naor-Yung ballot encryption | Keeps each ballot secret and unforgeable-by-modification; its checkable randomness enables **cast as intended** |
| 2. Joint-Feldman distributed key generation | Ensures no single party *can* decrypt ballots — the trust foundation for ballot secrecy |
| 3. Hash-chained, signed bulletin board | The tamper-evident public record; delivers **recorded as cast** and fixes the input to the count |
| 4. Terelius-Wikström verifiable mix | Unlinks ballots from voters before decryption, with proof that no ballot was added, dropped, or altered — half of **counted as recorded** |
| 5. Threshold decryption with Chaum-Pedersen proofs | Opens the mixed ballots with a public receipt for every step — the other half of **counted as recorded** |

---

## 1. How Cryptographic Components Are Proven Correct

The question "is this component correct?" is answered by *different methodologies at different
levels*, and each produces a different kind of evidence. This section defines them once; the
component sections then say which apply where. The honest one-line summary: **no single
methodology proves a voting system correct — an assurance case assembles complementary evidence,
each piece covering a specific class of error.**

### 1.1 Game-based security proofs (reductions)

*What it is.* Cryptographers define security as a game between a hypothetical attacker and the
scheme ("the attacker picks two ballots, receives the encryption of one, and must guess which").
A **reduction proof** shows: any attacker who wins this game can be mechanically converted into a
solver for a mathematical problem believed intractable (here, the *Decisional Diffie-Hellman* (DDH)
problem on elliptic curves).

*What the output signifies.* A conditional guarantee: "breaking ballot secrecy is at least as hard
as solving DDH." It says nothing about implementation bugs, side channels, or misuse — and it
depends on stated idealizations (typically the *random-oracle model*, which treats the hash
function as a perfect random function). These proofs exist on paper for all the primitives used
here (in `EVS` and the literature); they can be *mechanized* (machine-checked) in frameworks such
as CryptHOL or EasyCrypt — the project's own comparison notes are in
[isabelle-crypto.md](./isabelle-crypto.md).

*Status here:* paper proofs inherited from the literature/`EVS`; no mechanization yet.

### 1.2 Symbolic protocol verification (Tamarin)

*What it is.* A protocol model checker. The cryptography is treated as *perfect* (an attacker can
never break encryption or forge a signature), and the tool searches — exhaustively, over unbounded
protocol runs — for **message-level attacks**: replaying, reordering, mixing sessions,
impersonating, exploiting a missing check.

*What the output signifies.* A proven lemma means "no attack of this kind exists in the model,
even for an attacker who fully controls the network" (the *Dolev-Yao* attacker). A failed lemma
produces a concrete attack trace — a recipe for the flaw. What it cannot see: weaknesses *inside*
the cryptographic primitives, or bugs in code.

*Status here:* every subprotocol has a model
([`models/cryptography/tamarin/subprotocols/`](../models/cryptography/tamarin/subprotocols/)), with
**executability lemmas** proven (the protocols can actually run to completion — this catches
specification errors) and security lemmas stated opportunistically, not all proven — see the
[Tamarin README](../models/cryptography/tamarin/README.md). Completing these is the single largest
outstanding verification task.

### 1.3 Executable formal specification (Cryptol)

*What it is.* A precise, executable mathematical definition of each *algorithm* (as opposed to
each *protocol message flow*). Think of it as the authoritative reference implementation, written
in a language designed for specifying cryptography.

*What the output signifies.* An unambiguous answer to "what is this algorithm supposed to
compute?" — usable as a *digital twin* to cross-test the Rust code, and as a generator of
known-answer tests. The repo's `make verify` for Cryptol typechecks the modules and verifies their
embedded properties.

*Status here:* group/encoder algebra and ElGamal/Pedersen primitives are specified
([`models/cryptography/cryptol/`](../models/cryptography/cryptol/)); the effort was deliberately
paused in 2025 ([cryptol-specs.md](./cryptol-specs.md)) and is a natural place to resume for the
product line.

### 1.4 Sigma-protocol properties and the Fiat-Shamir transform

*What it is.* Four of the five kernel components rely on a family of ZKPs called **sigma
protocols**. Each is judged by three properties:

- **Completeness** — an honest prover always convinces the verifier (testable directly);
- **Soundness** — a cheating prover convinces the verifier only by guessing an unpredictable
  challenge, with odds comparable to guessing a 256-bit number: not "unlikely" but *never happens
  in the lifetime of the universe* (this is what cryptographers mean by "negligible");
- **Zero-knowledge** — the receipt reveals nothing beyond the truth of the claim.

The **Fiat-Shamir transform** makes these proofs non-interactive (a standalone receipt anyone can
check later) by deriving the challenge from a hash of the proof context. The pitfall: the hash
must cover the *entire* statement being proven. In 2019, researchers showed the SwissPost/Scytl
Internet-voting system used an incomplete ("weak") Fiat-Shamir transform in its shuffle proofs,
making it possible in principle to alter votes while producing proofs that still verified. This is
directly relevant here: the trustee actor carries explicit
`#[warning("Challenge inputs are incomplete.")]` markers
([`top_level_actor.rs`](../implementations/rust/workspace/protocol/src/trustee_protocols/trustee_application/top_level_actor.rs))
— completing and reviewing the challenge derivation is the top proof-investment priority named in
the previous two documents.

*What the output signifies.* A verified proof transcript is permanent, publicly checkable
mathematical evidence that the claimed relation holds — the property that makes "anyone can verify
the election" more than a slogan.

### 1.5 Implementation assurance (testing, model checking, analysis)

*What it is.* Evidence that the Rust code faithfully implements the specified algorithms: roughly
750 test functions, doctests on public APIs, [Stateright](https://www.stateright.rs/) model
checking that explores message orderings and failures across the actor state machines, Miri
(undefined-behavior detection), fuzzing, coverage analysis, and supply-chain checks
(`cargo deny`/`cargo vet`). The crypto crate forbids `unsafe` code entirely.

*What the output signifies.* Confidence over the *explored envelope* — evidence, not proof. Its
proper role in the assurance case is discharging the gap that no design-level proof can cover:
"the code does what the specification says."

### 1.6 Machine-checked theorem proving (Isabelle/HOL)

*What it is.* Interactive theorem provers mechanically check every logical step of a mathematical
proof. This is the gold standard of certainty per theorem, at the highest effort per theorem.
CryptHOL (§1.1) runs *inside* Isabelle, so game-based security proofs can be machine-checked too.

*Status here:* a seed exists — a completeness proof for the Schnorr ZKP
([`models/cryptography/isabelle/`](../models/cryptography/isabelle/README.md)). Since the
Chaum-Pedersen proof (§6) is a close cousin of Schnorr, extending this seed to the kernel's actual
sigma protocols is a natural, bounded next step.

### 1.7 The assurance case ties it together

The [AdvoCATE assurance case](../assurance/) (currently a skeleton) is where each claim ("ballots
are secret," "the tally matches the record") is decomposed until every leaf is discharged by one of
the evidence types above. For certification, *this* is the deliverable; the methodologies are its
suppliers.

---

## 2. Naor-Yung (CCA2) Ballot Encryption

### What it is

When a voting machine encrypts a ballot, ordinary encryption is not enough. The gold standard is
**CCA2 security** (security against *adaptive chosen-ciphertext attacks*), which additionally
guarantees **non-malleability**: nobody can take an encrypted ballot and produce a *related*
encrypted ballot — a copy, a tweak, a re-randomized duplicate — without first decrypting it.

Why does that matter in an election? Consider a corrupt insider who copies Alice's encrypted
ballot and submits it as ten new ballots. The tally would shift by ten votes toward whatever Alice
chose — and comparing tallies would *reveal Alice's vote*. Non-malleable encryption makes this
entire attack class impossible: an encrypted ballot is usable only as-is, once, by its author.

**Analogy.** A ballot is sealed into *two* identical locked boxes, shipped together with a
tamper-evident certificate attesting "both boxes contain the same thing." Anyone can check the
certificate without opening anything. A forger cannot produce a valid certificate for boxes he
didn't fill himself — so nobody can pass off a modified or copied box as a fresh, legitimate one.

The Naor-Yung construction is exactly this: the ballot is encrypted twice under a well-understood
scheme (ElGamal), and a zero-knowledge *plaintext-equality proof* certifies both ciphertexts hold
the same plaintext. Checking the proof requires no keys; it happens at the ballot box door.

### Where it is used in general

- CCA2 security is the required standard for public-key encryption wherever attackers can submit
  ciphertexts of their choosing and observe outcomes — which is essentially every networked
  system. (TLS and modern messaging achieve it with different constructions; Naor-Yung is the
  classic *generic* construction, prized for its clean security proof.)
- The "encrypt twice + prove consistency" pattern recurs across cryptographic protocol design
  whenever a system must accept ciphertexts from untrusted parties and still reason about them —
  including other verifiable-election designs.

### Where a VoteSecure-based system uses it

Every ballot is Naor-Yung-encrypted on the voting machine under the threshold election key (§3)
before submission. The Digital Ballot Box (the polling-place controller, in the on-site design)
verifies the proof before accepting the ballot onto the bulletin board — malformed or copied
ballots are rejected at the door. Before mixing, the trustees re-verify every proof and *strip* the
ciphertexts down to single ElGamal ciphertexts, which is what the mix (§5) operates on.

The same encryption's *randomizers* (the coin flips used during encryption) power **cast as
intended**: in the Benaloh-challenge flow, the machine discloses the randomizers for a challenged
ballot to a separate check station, which can then reproduce the encryption and show the voter
exactly what was inside — see the [ballot check spec](./protocol/specs/ballot-check-spec.md).

**Technical detail.**
Implementation: [`cryptosystem/elgamal.rs`](../implementations/rust/workspace/cryptography/src/cryptosystem/elgamal.rs)
(`EVS` Def. 11.15) and
[`cryptosystem/naoryung.rs`](../implementations/rust/workspace/cryptography/src/cryptosystem/naoryung.rs)
(`EVS` Def. 11.31), with the plaintext-equality proof in
[`zkp/pleq.rs`](../implementations/rust/workspace/cryptography/src/zkp/pleq.rs) (`EVS`
Prot. 10.8). The second Naor-Yung public key is derived by hashing public information, so *its
private key never exists* — a neat trick that upgrades an existing ElGamal key pair
(`KeyPair::augment`). Proof validation + stripping is `PublicKey::strip`. Protocol usage:
[ballot submission spec](./protocol/specs/ballot-submission-spec.md) (checks #6: "the ciphertext
has a valid Naor-Yung proof"; #7: duplicate rejection). Ciphertexts are width-`W` vectors over a
product group (`BALLOT_CIPHERTEXT_WIDTH` in `protocol/src/cryptography.rs`), sized from the
serialized ballot.

**How it is proven, and what the proofs mean.**
- *Game-based (§1.1):* IND-CCA2 security of Naor-Yung from the CPA security of ElGamal (DDH) plus
  the soundness of the equality proof — the classic paper argument; mechanizing it in
  CryptHOL/EasyCrypt would be the flagship candidate if design-level mechanization is funded. The
  output means: ballot secrecy and ballot independence hold *unless DDH falls or the hash
  idealization fails*.
- *Sigma properties (§1.4):* completeness/soundness/zero-knowledge of `pleq`, and — critically —
  Fiat-Shamir challenge derivation covering the full statement (both ciphertexts, both keys, and
  the context labels the API already threads through encryption).
- *Cryptol (§1.3):* the ElGamal primitive spec exists
  ([`Primitives/ElGamal`](../models/cryptography/cryptol/Primitives/)); extending it to Naor-Yung
  and generating known-answer tests against the Rust is a bounded, high-value task.
- *Symbolic (§1.2):* Tamarin treats the encryption as perfect and checks the *surrounding*
  message flow (who can submit what, when).
- *Implementation (§1.5):* doctests and property tests exercise round-trips and proof
  verification; `unsafe` is forbidden crate-wide.

---

## 3. Joint-Feldman Distributed Key Generation

### What it is

If one party held the election's decryption key, that party could decrypt every ballot as it
arrived. **Distributed key generation (DKG)** ensures the decryption key *never exists in one
place*: it is born already split among the trustees. The election public key — used by every
voting machine to encrypt ballots — is computed jointly, and decryption later requires a quorum
(any `T` of the `P` trustees). Fewer than `T` trustees, even conspiring, learn *nothing*.

The "verifiable" part matters just as much: each trustee can check that the share they received is
genuine, using published **check values**, *without* anyone revealing a share. A trustee who deals
corrupt shares is caught before the election key is ever accepted.

**Analogy.** `P` officers jointly forge a vault key such that any `T` of them, working together,
can open the vault — but `T-1` cannot even in principle, because the full key was never assembled
anywhere. Alongside the fragments, each officer publishes a set of reference measurements (the
check values); every other officer verifies their own fragment against the measurements without
showing it to anyone. Only when all fragments check out do the officers jointly announce, and
sign, the vault's public face — the election public key.

### Where it is used in general

- Certificate authorities and DNSSEC root ceremonies (splitting root keys among officers),
- hardware security module (HSM) clusters and enterprise key-management services,
- cryptocurrency custody (threshold wallets used by exchanges and institutions),
- distributed randomness beacons and threshold-signing networks.

Anywhere a single key would be a single point of failure or corruption, a DKG of this family is
the standard remedy.

### Where a VoteSecure-based system uses it

At election setup — inside the air-gapped trustee ceremony in the on-site design — the trustees
run the DKG through the Trustee Administration Server's message board. Each trustee posts check
values and encrypted pairwise shares; each verifies everything received; each independently
computes the election public key; and the protocol succeeds only when *all* trustees compute and
sign the *same* key ([election key generation spec](./protocol/specs/election-key-gen-spec.md)).
That key is then published in the signed election configuration, and every ballot in the election
is encrypted under it. The check values do double duty later: they let anyone compute each
trustee's *verification key*, which anchors the decryption receipts of §6.

**Technical detail.**
Implementation:
[`dkgd/dealer.rs`](../implementations/rust/workspace/cryptography/src/dkgd/dealer.rs) and
[`dkgd/recipient.rs`](../implementations/rust/workspace/cryptography/src/dkgd/recipient.rs)
(`EVS` Prot. 16.20). Each dealer samples a random polynomial of degree `T-1`; shares are
evaluations, check values are commitments to coefficients; the joint public key and all
verification keys are computable from public data alone. Threshold parameters are runtime values;
end-to-end tests cover 3 trustees/threshold 2 and 9 trustees/threshold 5, with checkpoint/resume
variants (`protocol/src/trustee_protocols/integration_tests_basic.rs`). Known repo caveats: the
`dkgd` module is marked `not optimized`, and its test module carries
`#[warning("Need more threshold parameter combinations")]`. Tamarin model:
[`election_key_generation.spthy.m4`](../models/cryptography/tamarin/subprotocols/election_key_generation.spthy.m4).

**How it is proven, and what the proofs mean.**
- *Correctness:* shares-consistent-with-check-values and reconstruct-above-threshold are directly
  testable properties (and good Cryptol targets). Output: a corrupt dealer is detected before the
  key is accepted; any `T` honest trustees can decrypt.
- *Game-based secrecy (§1.1):* a simulation argument that coalitions below `T` learn nothing.
  One known subtlety should be stated explicitly in the design documentation: for Joint-Feldman
  specifically, Gennaro, Jarecki, Krawczyk, and Rabin (1999) showed an active adversary can *bias
  the distribution* of the resulting public key. This does not help decrypt anything and is widely
  regarded as benign for encryption keys, but a certifier will ask — the design-level proof should
  say why it is acceptable here (or the protocol upgraded to the GJKR variant if not).
- *Symbolic (§1.2):* the Tamarin model covers the message flow — share delivery, board-slot
  discipline, the all-keys-identical agreement at the end. Proven lemmas here mean: no replay,
  substitution, or session-mixing attack can cause honest trustees to accept inconsistent keys.
- *Implementation (§1.5):* the threshold test matrix, plus Stateright exploration of the trustee
  actors' message interleavings.

---

## 4. The Hash-Chained, Signed Public Bulletin Board

### What it is

E2E-V stands or falls on a public record that cannot be quietly rewritten. The bulletin board is
an **append-only ledger** with two properties. First, every entry embeds the hash — the
fingerprint — of the entry before it, so entries form a chain: altering or removing any entry
changes its fingerprint, which breaks the entry after it, and so on to the end. Tampering with
history is not impossible — it is *loud*. Second, every entry is digitally signed by the ballot
box, so entries cannot be fabricated by outsiders and the operator cannot disown them.

The voter's **ballot tracker** — printed on their receipt — is simply the fingerprint of their own
entry. Finding that fingerprint on the published board *is* the "recorded as cast" check.

**Analogy.** A notary's bound ledger in which each page's wax seal is made by melting in fragments
of the previous page's seal. Tear out or rewrite page 40, and the seals on pages 41 onward no
longer match — visibly, to anyone who checks, forever. The voter's receipt is a copy of one page's
seal.

### Where it is used in general

This is the most widely deployed idea in the kernel:

- **Git** — every commit ID is a hash chaining the entire prior history;
- **Certificate Transparency** — the append-only logs that keep web-certificate issuance honest;
- **Blockchains** — hash-chained blocks, plus a consensus mechanism this design does not need;
- signed audit logs in regulated industries; software supply-chain transparency logs (e.g.,
  Sigstore's Rekor).

### Where a VoteSecure-based system uses it

The polling-place controller (Digital Ballot Box) maintains the board. Three entry types matter:
ballot *submission* entries (the encrypted ballot, in full), voter *authorization* entries, and
ballot *cast* entries — the chain interleaves them in arrival order
([`bulletins.rs`](../implementations/rust/workspace/protocol/src/bulletins.rs)). In the on-site
design, each polling place accumulates its own board while offline; after close, the boards are
published centrally, and the printed **chain-head attestation** (the final fingerprint, posted
publicly at the precinct at close) binds what the precinct saw to what the world sees. The
trustees' mixing ceremony begins by snapshotting and verifying this board — the board *is* the
commitment to the set of ballots that must be counted.

**Technical detail.**
Implementation: the storage-agnostic `BulletinBoard` trait in
[`bulletin_board.rs`](../implementations/rust/workspace/protocol/src/participants/digital_ballot_box/bulletin_board.rs)
(append validates `previous_bb_msg_hash`, computes the entry hash, returns it as the tracker);
entry structures in
[`bulletins.rs`](../implementations/rust/workspace/protocol/src/bulletins.rs). Chain and tracker
semantics: [ballot submission spec](./protocol/specs/ballot-submission-spec.md) (Phases 2–3);
cast-time rules preventing double-casting:
[ballot cast spec](./protocol/specs/ballot-cast-spec.md) (checks #4–5). Tamarin: board behavior is
exercised within the submission/cast subprotocol models and a dedicated composition test
([`compositions/`](../models/cryptography/tamarin/compositions/)).

**How it is proven, and what the proofs mean.**
- *Reduction to primitives (§1.1):* tamper-evidence reduces to hash collision resistance;
  entry authenticity to signature unforgeability. These are standard, well-studied reductions —
  the board needs no exotic cryptography.
- *Data-structure invariants (§1.5):* "every accepted append references the current head,"
  "trackers are unique," "reads reflect appends" are exactly the kind of invariants property tests
  and Stateright model checking discharge well — including under adversarial message orderings.
- *Symbolic (§1.2):* Tamarin lemmas cover what the surrounding protocol *does* with the board —
  e.g., that a ballot accepted for casting really has a matching submission entry.
- *What the outputs do — and do not — signify.* The proofs give **tamper-evidence and ordering**,
  not availability (a destroyed board is a detected disaster, not a prevented one — hence backups
  and paper), and not **non-equivocation**: a corrupt operator could try showing different board
  versions to different people. Equivocation is *detected*, not prevented, by cross-comparison —
  which is precisely why the on-site design adds printed chain-head attestations and independent
  post-close mirrors (feature VP-B2 in the
  [variation catalog](./onsite-e2ev-feature-variations.md)).

---

## 5. The Terelius-Wikström Verifiable Mix

### What it is

After the polls close, the board holds encrypted ballots *in submission order, attached to voter
pseudonyms*. Decrypting them directly would link every vote to a voter. A **mix-net** breaks that
link: each trustee, in turn, re-randomizes every ciphertext (same contents, new mathematical
wrapping — so the outputs cannot be matched to the inputs by appearance) and shuffles the order.
After all trustees have mixed, nobody — not even the trustees, unless *all* of them collude — can
say which output ballot was which input ballot.

The obvious danger: a trustee doing the shuffling in secret could swap a ballot for a forged one.
The **proof of shuffle** eliminates the danger: each trustee produces a zero-knowledge receipt
that their output is *exactly* a re-randomized permutation of their input — nothing added, nothing
dropped, nothing altered — while revealing nothing about the permutation itself.

**Analogy.** A stack of sealed envelopes passes through several clerks. Each clerk places every
envelope into a fresh outer envelope (contents untouched) and shuffles the stack. Each clerk also
produces a mathematical receipt that the outgoing stack contains precisely the contents of the
incoming stack. Observers can check every receipt without opening anything — but even with all
receipts in hand, cannot reconstruct any clerk's shuffle.

This yields the kernel's signature trust property: **privacy holds if even one mixing trustee is
honest; integrity holds even if every trustee is corrupt** — because forged outputs cannot carry
valid receipts.

### Where it is used in general

- Verifiable mixing is the core of several deployed national Internet-voting systems — the
  Verificatum mix-net, built on this same proof family by its designer, has been used in binding
  government elections (e.g., Norway's Internet-voting pilots and Estonia's IVXV framework).
- Mix networks for anonymous communication and metadata-resistant messaging.
- Anonymous surveys, whistleblowing systems, and privacy-preserving data publication — anywhere a
  set of contributions must be published *unlinkably but provably completely*.

### Where a VoteSecure-based system uses it

In the air-gapped tally ceremony. The trustees verify the board snapshot, validate and strip every
Naor-Yung ciphertext (§2), and then mix in a publicly known order, each posting shuffled
ciphertexts plus their shuffle proof to the trustee board; every trustee verifies every other
trustee's proof before the ceremony proceeds
([trustee mixing spec](./protocol/specs/trustee-mixing-spec.md)). The published transcript —
inputs, every round's outputs, every proof — is what lets *anyone* verify, after the fact, that
the set of counted ballots is exactly the set of recorded ballots. Because full plaintext ballots
emerge after decryption, this approach supports every ballot semantics in the product line,
including ranked-choice — the reason the mix is kernel rather than optional.

**Technical detail.**
Implementation:
[`zkp/shuffle.rs`](../implementations/rust/workspace/cryptography/src/zkp/shuffle.rs) (`EVS`
Prot. 12.3), operating on width-`W` ElGamal ciphertexts, with rayon parallelization behind the
`server` feature and benchmarks in `cryptography/benches/shuffle.rs`. The proof follows
Terelius-Wikström: a commitment to a permutation matrix plus a batched argument that the committed
permutation relates inputs to outputs; both prover and verifier derive `N` independent group
generators from common data. Mixing is batched per ballot style (`BTreeMap<BallotStyle, …>` in the
trustee messages). Tamarin model:
[`trustee_mixing.spthy.m4`](../models/cryptography/tamarin/subprotocols/trustee_mixing.spthy.m4).
Known repo caveats: optimization warnings, one `skip(1)` behavior question, and Miri
Stacked-Borrows failures in shuffle tests — tracked via `#[warning]` markers in the file.

**How it is proven, and what the proofs mean.**
- *Sigma/argument properties (§1.4):* completeness (honest mixes always verify — testable),
  soundness (a verifying transcript implies the multiset of plaintexts is preserved, except with
  negligible probability), and zero-knowledge (the transcript reveals nothing about the
  permutation). **Fiat-Shamir challenge binding is the critical obligation** — this is exactly
  where SwissPost/Scytl went wrong, and exactly where this repo's
  `"Challenge inputs are incomplete"` warnings sit. No variation work should proceed until this
  is completed and reviewed.
- *Mechanized verification:* machine-checked proofs of Terelius-Wikström-style shuffle arguments
  exist in the academic literature, as do *verified verifiers* — verifier implementations proven
  correct in a proof assistant. Given that election integrity rests on the verifier accepting only
  valid transcripts, a verified (or at least independently re-implemented) verifier is the
  highest-leverage mechanization target in the whole system.
- *Symbolic (§1.2):* the Tamarin model covers ceremony orchestration — ordering, board-slot
  discipline, who must verify what before proceeding.
- *Implementation (§1.5):* round-trip and negative tests (tampered outputs must fail
  verification), benchmarks for the performance envelope, fuzzing of deserialization.

---

## 6. Threshold Decryption with Chaum-Pedersen Proofs

### What it is

After mixing, the anonymous ciphertexts must be opened — *without* ever assembling the private key
that the DKG (§3) deliberately never created. In **threshold decryption**, each trustee applies
their own key share to each ciphertext, producing a *partial decryption* (a "decryption factor").
Any `T` valid partial decryptions combine mathematically into the plaintext ballot.

The accountability layer: each partial decryption ships with a **Chaum-Pedersen proof** — a
zero-knowledge receipt that the trustee really used their genuine, registered key share, and not
some other value that would corrupt the result. The receipt is checked against the trustee's
public verification key, which anyone can compute from the DKG's public check values. A trustee
who submits a bad partial decryption is not merely detected — they are *identified*, immediately
and attributably.

**Analogy.** Opening the vault from §3: each officer turns their own key and hands over a
work-receipt. Anyone can hold a receipt up against that officer's public reference measurement and
confirm the turn was genuine. `T` genuine turns open the vault; one fake turn is spotted on the
spot, with the culprit's name on it.

### Where it is used in general

- Threshold decryption and signing services (cloud KMS/HSM offerings, notarization networks),
- key escrow designs where recovery must be possible but *accountable*,
- distributed randomness beacons (partial evaluations with correctness proofs),
- other verifiable-election systems — homomorphic-tally designs (e.g., Helios-style) use the very
  same Chaum-Pedersen receipts to prove their published totals were decrypted honestly.

### Where a VoteSecure-based system uses it

The final step of the air-gapped ceremony. Each participating trustee posts partial decryptions —
with proofs — for every mixed ciphertext; every trustee verifies every proof; the verified factors
are combined into plaintext ballots, which all trustees sign
([trustee decryption spec](./protocol/specs/trustee-decryption-spec.md)). Together with the mixing
transcript (§5) and the board snapshot (§4), this completes **counted as recorded**: a member of
the public can start from the published encrypted ballots and mechanically re-verify every step
down to the plaintext tally. Nothing about the count needs to be taken on trust — that is the
entire point of the kernel.

**Technical detail.**
Implementation: partial decryption in
[`dkgd/recipient.rs`](../implementations/rust/workspace/cryptography/src/dkgd/recipient.rs)
(`Recipient::decryption_factor`, combination via Lagrange interpolation in the exponent in
`combine`); the proof in
[`zkp/dlogeq.rs`](../implementations/rust/workspace/cryptography/src/zkp/dlogeq.rs) (`EVS`
Prot. 10.3) — a proof of *discrete-logarithm equality* showing one secret exponent
simultaneously links the trustee's verification key and the decryption factor (width-1 against
width-`W` statements). Tamarin model:
[`trustee_decryption.spthy.m4`](../models/cryptography/tamarin/subprotocols/trustee_decryption.spthy.m4).

**How it is proven, and what the proofs mean.**
- *Sigma properties (§1.4):* the Chaum-Pedersen trio — completeness, soundness (a verifying
  receipt means the genuine share was used, up to negligible probability), zero-knowledge (the
  receipt leaks nothing about the share). Fiat-Shamir binding applies here too (the SwissPost
  findings included a decryption-proof variant of the same flaw).
- *Machine-checked seed (§1.6):* the repo's existing Isabelle proof covers Schnorr completeness;
  Chaum-Pedersen is structurally a two-statement Schnorr, making it the cheapest place to grow the
  Isabelle work into the real kernel.
- *Correctness (§1.3/§1.5):* "any `T` valid factors combine to the plaintext, for all threshold
  configurations" is a directly testable (and Cryptol-specifiable) property; the current test
  matrix covers two configurations and should grow (the repo's own warning says so).
- *Symbolic (§1.2):* the Tamarin model covers the ceremony flow — that honest trustees abort on
  any invalid partial decryption, and that the final signed plaintexts correspond to the mixed
  inputs.

---

## 7. Summary Catalog

| # | Component | `EVS` ref | Rust implementation | Protocol spec | Tamarin model | Evidence today | Highest-value next step |
|---|---|---|---|---|---|---|---|
| 1 | Naor-Yung ballot encryption | Def. 11.15/11.31, Prot. 10.8 | `cryptosystem/{elgamal,naoryung}.rs`, `zkp/pleq.rs` | ballot-submission | `ballot_submission.spthy.m4` | Paper proofs (literature); tests/doctests; Cryptol ElGamal spec | Fiat-Shamir binding review; Cryptol Naor-Yung spec + KATs |
| 2 | Joint-Feldman DKG | Prot. 16.20 | `dkgd/{dealer,recipient}.rs` | election-key-gen | `election_key_generation.spthy.m4` | Executability + tests (2 threshold configs); known optimization/test-coverage warnings | Prove Tamarin secrecy/agreement lemmas; widen threshold matrix; document the key-bias subtlety |
| 3 | Bulletin board | — (hash + signatures) | `digital_ballot_box/bulletin_board.rs`, `bulletins.rs` | ballot-submission / ballot-cast | submission/cast models + BB composition | Property tests; Stateright actor checking | Equivocation-detection features (attestation, mirrors) per VP-B2; chain-invariant persistence model |
| 4 | Terelius-Wikström mix | Prot. 12.3 | `zkp/shuffle.rs` | trustee-mixing | `trustee_mixing.spthy.m4` | Implemented + tested + benchmarked; **challenge-input warnings open** | **Complete Fiat-Shamir challenge binding**; independent/verified verifier |
| 5 | Threshold decryption + Chaum-Pedersen | Prot. 10.3 | `dkgd/recipient.rs`, `zkp/dlogeq.rs` | trustee-decryption | `trustee_decryption.spthy.m4` | Tests across threshold configs; Isabelle Schnorr seed | Extend Isabelle seed to Chaum-Pedersen; same Fiat-Shamir review |

Read down the "next step" column and one theme dominates: **complete and review the Fiat-Shamir
challenge derivations before anything else**. Every zero-knowledge receipt in the system — ballot
validity, shuffle, decryption — hangs on that transform being done right, and the repo's own
warnings mark it unfinished. It is the one place where a small amount of code is load-bearing for
the entire public-verifiability claim.

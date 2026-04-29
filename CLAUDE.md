# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

VoteSecure is the cryptographic core of an end-to-end verifiable Internet voting (E2E-VIV) system, developed by Free & Fair using a Rigorous Digital Engineering (RDE) methodology. The repository is *not* a deployable voting system; it is a library plus the formal models, proofs, and assurance evidence that justify the library's correctness and security.

The project spans multiple artifact families that must remain consistent with one another: domain/feature/threat/SysML models, formal cryptographic models (Cryptol, Tamarin, Isabelle), a Rust implementation, and an AdvoCATE assurance case. Changes in one family often require coordinated changes elsewhere.

## Repository Layout

- `models/` — RDE models: `domain-model` (Lando), `feature-model` (Clafer + SysMLv2), `threat-model` (LaTeX + Python tooling), `sysml-model`, and `cryptography/` (`cryptol`, `cryptol-specs`, `tamarin`, `isabelle`).
- `implementations/rust/workspace/` — Rust workspace with two crates: `cryptography` (primitives: groups, ElGamal/Naor-Yung, DKG/distributed decryption, ZKPs, signatures, hashing, serialization) and `protocol` (actor implementations and data structures for the VoteSecure protocol).
- `assurance/` — AdvoCATE assurance case (skeleton; not yet populated with implementation evidence).
- `docker/` — Image definitions for the three CI/CV containers (`cpv-e2eviv`, `de-ple-e2eviv`, `isabelle-e2eviv`) plus an aggregator Makefile.
- `docs/` — Project documentation including `ci_cd_cv.md`, `team.md`, CONOPS, and protocol/spec docs.
- `examples/needham-schroeder/` — Small RDE example (used by `ci-lando`).
- `utils/` — Tooling helpers for cryptol, lando, tamarin, vscode.

Each subdirectory has its own README and (where applicable) its own Makefile. The top-level Makefile delegates to those localized Makefiles — do not duplicate component logic at the top level.

## Build, Lint, Test

### Top-level orchestration (from repo root)

```
make ci            # Lando + feature model + threat model + Tamarin + Rust + assurance CI
make cv            # Tamarin CV + Cryptol verify + Rust tests
make cv-parallel   # CV in parallel; requires GNU Make >= 4.0 (gmake on macOS); CV_JOBS=<n> to override
make docker-pull   # Pull required CI/CV images
make docker-build  # Build all repo Docker images locally
make clean
```

Useful overrides: `DOCKER=docker|podman`, `IMAGE_PLATFORM=linux/amd64` (for arm64 hosts when an x86 image is needed), `USE_DOCKER=yes` (force container-backed execution where supported), `PYTHON_BOOTSTRAP=yes` (set up Python venvs for feature/threat/assurance helpers via `make ci-python`).

The `USE_DOCKER` policy: tools run locally if installed; otherwise the localized Makefile delegates to the appropriate Docker container. `USE_DOCKER=yes` forces container-backed execution. Containers must already exist locally — run `make docker-pull` or `make docker-build` first.

### Rust workspace (`implementations/rust/workspace/`)

Requires the **nightly** toolchain. `rust-toolchain.toml` selects this automatically.

```
make ci            # fmt --check, cargo check, clippy, cargo deny (bans/advisories/sources), release build
make cv            # cargo test --release -- --test-threads=1
```

Run specific tests directly with cargo:

```
cargo test -p cryptography                          # all tests in one crate
cargo test -p cryptography <name_substring>         # filter by test name
cargo test --doc                                    # doctests only
cargo test --lib --bins --tests                     # everything except doctests
```

`--test-threads=1` is **required** for the full suite: many tests use Stateright model checking with their own internal concurrency, and cargo-level parallelism causes severe slowdown. Keep this when adding new tests.

Other Rust tooling (see `implementations/rust/workspace/cryptography/DEVELOPER.md`):
- Coverage: `cargo llvm-cov` (add `--branch` for branch analysis, `report --html --open` for a browsable report)
- Supply chain: `cargo deny check`, `cargo vet`
- UB checks: `cargo miri nextest run -j<cores>`
- Fuzzing: `cargo fuzz list` / `cargo fuzz run <target>`
- Custom warnings: build with `--features=custom-warnings` (defined in `workspace/macros/custom_warning_macro`)

### Cryptol (`models/cryptography/cryptol/`)

```
make verify   # cryptol --project . — typechecks all modules and runs docstring property verification
```

### Other model components

Each delegated target invokes `make ci` (and where applicable `cv`) inside the component directory:
- `make -C models/cryptography/tamarin {ci,cv}`
- `make -C models/feature-model ci`
- `make -C models/threat-model ci`
- `make -C models/domain-model/lando ci`
- `make -C assurance ci`

## Conventions That Aren't Obvious from the Code

- **Conventional Commits** are enforced by the `commitlint` pre-commit hook. Standard prefixes plus two project additions: `wip` (for draft PR work expected to be squashed) and `cosmetics` (cosmetic-only changes).
- **Linear history** on `main` and release branches: PR branches must be rebased and merged fast-forward (`git merge --ff-only` from the command line — GitHub's UI cannot do this currently). Use `git pull --rebase` for shared branches and `--force-with-lease` (never `--force`) when rewriting.
- **Signed commits** are required for anything that lands on `main` or a release branch (enforced via branch protection).
- **Text file hygiene** (enforced by pre-commit hooks): UTF-8, LF line endings, no trailing whitespace, file ends with a newline.
- **Pre-commit hooks** (`.pre-commit-config.yaml`) run `cargo fmt --check` (style edition 2024), `cargo check`, and `cargo clippy -- -D warnings` against `implementations/rust/workspace/Cargo.toml`. Install locally with `pip install pre-commit && pre-commit install`.
- **PRs must reference an issue** (use `Closes #N` for auto-close). Open as drafts early for visibility; only minor maintenance changes may merge without a second reviewer.

## CI / CV Surface

GitHub Actions live in `.github/workflows/`. Per-artifact validity workflows (`test-validity-of-*`) run on changes to that artifact's directory; `verify-*` workflows run the heavier CV checks (Cryptol docstrings, Tamarin proofs, Rust tests). The Docker image workflows (`run-makefile-for-*-docker-image.yml`) only build images when their definitions change on `main` — there is no automated push to a registry.

When changing CI/CD/CV behavior, update `docs/ci_cd_cv.md` in the same PR.

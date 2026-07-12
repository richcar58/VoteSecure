# BMVS Issues Log

**Date opened:** 2026-07-11
**Status:** Living tracking document
**Series:** companion to the [Bootstrap Plan](./onsite-e2ev-bootstrap-plan.md); migrates to
`bmvs/docs/` with the rest of the series per decision D7

This document tracks work items that have been **deliberately postponed** during the BMVS
bootstrap, plus any items postponed in the future. Each entry records (1) what the issue is and
why it is important, and (2) a recommended implementation, so that when an item's trigger fires,
the work is fully specified and nothing has to be re-derived.

**Conventions.** Entries are numbered `ISS-N` and are never deleted: when an item is completed,
its state changes to *Resolved* with a date and a short resolution note. States: **Postponed**
(waiting on a named trigger), **Open** (actionable now), **Resolved**. New entries append at the
bottom; the index below is kept current.

## Index

| ID | Title | State | Opened | Trigger | Affects |
|---|---|---|---|---|---|
| [ISS-1](#iss-1--install-pre-commit-hooks-in-both-clones) | Install pre-commit hooks in both clones | Postponed | 2026-07-11 | Repositories made public | `bmvs`, `bmvs-verifier` |
| [ISS-2](#iss-2--github-branch-protection-and-actions-policy) | GitHub branch protection and Actions policy | Postponed | 2026-07-11 | Repositories made public | `bmvs`, `bmvs-verifier` |

---

## ISS-1 — Install pre-commit hooks in both clones

- **State:** Postponed (trigger: repositories made public; may be pulled forward at any time —
  see below)
- **Opened:** 2026-07-11, from Phase 0 checklist item 4
- **Affects:** `bmvs`, `bmvs-verifier` (each clone, each machine)

### What the issue is, and why it is important

The pre-commit hooks are the **commit-time enforcement layer** for decision D6: the text-hygiene
hooks (UTF-8/LF, no trailing whitespace, final newline, YAML validity) and the `commitlint` hook
that enforces Conventional Commits — the latter requiring the `commit-msg` hook type, which is a
separate installation step that is easy to miss. Both repositories already carry the
configuration (`.pre-commit-config.yaml`, `.commitlintrc.js`); installation is what makes it
execute.

Until the hooks are installed, **no automated enforcement of D6 exists anywhere**, because the
CI backstop (`run-precommit-hooks.yml`) is also inactive until GitHub Actions is enabled
(ISS-2). Everything currently rests on discipline. This is not hypothetical: the initial commits
in both repositories (`bootstrap repository per BMVS plan Phase 0`) lack a Conventional Commits
type prefix (`chore: …`) and would have been rejected by the commitlint hook had it been
installed.

Why it matters beyond tidiness: the repositories are private now but intended to become public,
so **history accumulated during the private phase becomes permanent public history**. Under the
linear-history and signed-commits rules, retrofitting message format later means rewriting
`main` — cheap only while the repos are private and single-user. The conventions also feed
machine consumers later (changelog generation, release tooling), and for a voting product the
repository history is part of the reviewable evidence trail.

Note on the trigger: unlike ISS-2, this item has **no technical dependency on repository
visibility**. `pre-commit install` writes into the local clone's `.git/hooks`, and the hook
environments are fetched from public upstream repositories (`pre-commit/pre-commit-hooks`,
`FreeAndFair/commitlint-pre-commit-hook`) regardless of the visibility of `bmvs` and
`bmvs-verifier`. The postponement is a scheduling choice, and the item can be pulled forward at
zero cost whenever convenient — doing so before further commits accumulate maximizes its value.

### Recommended implementation

1. Once per machine (if not already present): `pipx install pre-commit`
   (or `pip install --user pre-commit`).
2. Once per clone, in each of `~/git/bmvs` and `~/git/bmvs-verifier`:

   ```bash
   pre-commit install
   pre-commit install --hook-type commit-msg
   ```

3. Baseline check: `pre-commit run --all-files` in each repo (expected to pass — all bootstrap
   files were authored to the hygiene rules).
4. Pre-publication history audit (recommended before the repos go public): list nonconforming
   commit messages, e.g. `npx commitlint --from=<root-commit> --to=HEAD`, then decide between
   (a) amending the small private-phase history to conventional format while rewriting is still
   cheap and non-disruptive, or (b) grandfathering the existing commits and enforcing from the
   hook-installation point forward. Either is defensible; deciding *before* publication is the
   valuable part.
5. When the cargo workspaces are scaffolded (bootstrap Phases 2–3), extend
   `.pre-commit-config.yaml` with the kernel repository's Rust hooks (`fmt`, `cargo-check`,
   `clippy -D warnings`) as already noted in the config file's trailing comment, and re-run
   `pre-commit run --all-files`.

---

## ISS-2 — GitHub branch protection and Actions policy

- **State:** Postponed (trigger: repositories made public)
- **Opened:** 2026-07-11, from Phase 0 checklist item 6
- **Affects:** `bmvs`, `bmvs-verifier` (GitHub repository settings; plus one small in-repo file
  change for SHA-pinning)

### What the issue is, and why it is important

Two distinct controls, both GitHub-side, both *authoritative* in the sense that no local
mechanism can substitute for them:

**(a) Branch protection on `main`.** Requiring **linear history**, **signed commits**, and
**passing status checks** converts decision D6 from a convention into a gate that even the
repository owner cannot accidentally bypass with a stray push. Without it, an unsigned commit or
a merge commit lands silently and — once public — permanently. Signed, linear, checked history
is also part of the certification story: it is what lets a reviewer trust that the published
history is exactly what the maintainer produced.

**(b) Actions permissions policy.** CI workflows execute third-party code (`actions/checkout`,
`actions/setup-python`, `pre-commit/action`) with access to a workflow token. GitHub's defaults
are permissive: any action may run, and the token has broad permissions. For a voting product,
the CI pipeline is inside the trust boundary of the build evidence (the reproducible-build and
supply-chain posture of the code-organization document), so the Actions surface should be
minimized: an allowlist of specific actions, **SHA-pinned** references (a tag like `@v6` is
mutable; a full commit SHA is not), and a read-only default token.

Consequence while postponed: `run-precommit-hooks.yml` never executes (Actions disabled), so
combined with ISS-1 there is currently no automated enforcement in either repository — the two
issues compound, which is another reason to pull ISS-1 forward. The postponement rationale is
sound (Actions minutes on private repositories are metered, and settings would be revisited at
publication anyway); the compounding risk is managed by resolving ISS-1 early.

### Recommended implementation

**(a) Branch protection** (per repo: *Settings → Branches → Add branch protection rule* for
`main`, or the equivalent Ruleset):

1. Enable **Require signed commits**.
2. Enable **Require linear history**.
3. Enable **Require status checks to pass** and add *Run pre-commit Hooks* as a required check —
   this option appears only after the workflow has executed at least once, so enable Actions
   first, let the workflow run on `main` or a PR, then designate it.
4. Leave **Require a pull request before merging** off initially: the working convention (from
   the kernel fork) merges fast-forward from the command line and pushes, which a PR requirement
   would block. Revisit when a second regular reviewer exists.

**(b) Actions policy** (per repo: *Settings → Actions → General*):

1. Actions permissions: *Allow `richcar58` actions and select non-owner actions*, with the
   allowlist containing exactly the actions used by the workflows.
2. Workflow permissions: set the default `GITHUB_TOKEN` to **read-only**; leave "Allow GitHub
   Actions to create and approve pull requests" disabled.
3. Once public: require approval for workflow runs from outside collaborators (fork PRs).
4. **In-repo half (local file change, prepared on request):** pin the three action references in
   `.github/workflows/run-precommit-hooks.yml` to full commit SHAs in both repositories
   (currently tag references mirroring the kernel fork's workflow), with the tag recorded in a
   trailing comment for readability, e.g.
   `uses: actions/checkout@<full-sha> # v6`. The same change is an upstreaming candidate for the
   kernel fork's workflows.
5. After both halves: verify end-to-end by pushing a branch with a deliberate hygiene violation
   and confirming the required check blocks it.

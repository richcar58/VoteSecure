# Fork Policy: Upstream Tracking and Scope

**Date:** 2026-07-11
**Status:** Fork governance (BMVS bootstrap plan, Phase 1 item 5)

This repository (`richcar58/VoteSecure`) is the **kernel repository** of the BMVS
three-repository organization: a fork of
[FreeAndFair/VoteSecure](https://github.com/FreeAndFair/VoteSecure) that stays pristine and
upstream-mergeable. The BMVS product and verifier repositories consume it by pinned git tag. The
governing plan lives in the [`bmvs` repository](https://github.com/richcar58/bmvs) under
`docs/onsite-e2ev-bootstrap-plan.md`.

## Scope charter

**What lands in this fork:** kernel-grade work only — changes to the kernel crates
(`cryptography`, `votesecure-protocol-library`, their proc-macros), the kernel models
(Cryptol, Tamarin, Isabelle), kernel documentation, and CI for those artifacts. Every change made
here should be a candidate for contribution upstream.

**What never lands here:** anything BMVS-specific — product crates, product protocol
specifications and models, product documentation (which lives in `bmvs/docs/` in the
[`bmvs` repository](https://github.com/richcar58/bmvs)), deployment or packaging definitions.

## Remotes

| Remote | URL | Role |
|---|---|---|
| `origin` | `https://github.com/richcar58/VoteSecure.git` | The fork; all pushes are Rich's (plan decision D9) |
| `upstream` | `https://github.com/FreeAndFair/VoteSecure.git` | Read-only tracking of Free & Fair's development |

## Sync cadence and procedure

**Cadence:** monthly, and additionally whenever upstream publishes a security-relevant change
(watch the upstream repository's releases and security advisories on GitHub so the signal arrives
between syncs).

**Procedure** (fast-forward only, per the fork's linear-history convention):

```bash
git fetch upstream --tags
git checkout main
git merge --ff-only upstream/main
git push origin main            # Rich (D9)
# then bring working branches up to date as needed:
git checkout <branch> && git rebase main
```

If the fast-forward merge fails, `main` has diverged — either upstream rewrote history (should
not happen) or non-upstream commits landed on `main` in violation of this policy. Stop and
investigate before proceeding; do not resolve with a merge commit.

**Fork-carried kernel changes** (e.g., the pinned toolchain date, the kernel work-queue items in
[`kernel-work-queue.md`](./kernel-work-queue.md)) live on fork branches until offered upstream;
whenever upstream accepts one, the next sync absorbs it and the fork-local commit disappears in
the rebase.

## Consumption tags

The product and verifier repositories depend on this kernel **only through tags** named
`kernel-v<upstream-version>-fork.<n>` (plan decision D5). Cutting a tag is a reviewed event: the
tag commit must build (`cargo check --workspace`), pass the test suite, and its changelog-worthy
delta from the previous tag should be one screenful. Tags are signed (`git tag -s`) and pushed by
Rich.

## Sync log

| Date | Result |
|---|---|
| 2026-07-11 | First tracking check: `main` is identical to `upstream/main`; upstream has tagged the current head (`3b16d14`) as `v1_3`, so the kernel lineage is **1.3** and the first consumption tag is `kernel-v1.3-fork.1`. No sync action needed. |

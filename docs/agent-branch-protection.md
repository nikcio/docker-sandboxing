# Branch policies for working with an AI agent

Autonomous agents commit fast, commit often, and can push anywhere your
token can. Branch protection on GitHub is the safety net that makes "the
agent broke `main`" impossible: every change — the agent's and yours — has
to pass through a PR that CI validates before it can land.

Set these once as a repo admin. Two prerequisites keep the net intact:

- **Give the agent a least-privilege token** — the fine-grained PAT from
  [Create a GitHub PAT](github-pat.md) (no Administration, workflows
  unset). A classic `repo`-scoped token or any Administration token can
  edit or delete the rules themselves.
- **Never bypass-list the account the agent's token belongs to.** A
  ruleset bypass entry silently disables every rule for that account.

Use branch **rulesets** for everything below (Settings → Rules →
Rulesets → **New branch ruleset**): they apply to admins by default, are
free on all plans, and also cover tags. Classic branch protection
(Settings → Branches) is the fallback — there, tick "Do not allow
bypassing the above settings" or admins slip past every rule.

## 1. Require a pull request before merging

Enable **Require a pull request before merging** and leave **Required
approvals at 0** (classic: don't tick "Require approvals"):

- A PR author can never approve their own PR, so any approval
  requirement is a human gate — on a solo repo, every merge becomes an
  override click. At 0 approvals, required checks (step 2) decide.
- Direct pushes are where agent work becomes unreviewable: the agent
  pushes to `main` mid-session and nobody sees the diff until something
  breaks. With "require a PR" on, the agent's push to `main` is rejected
  and it must open a PR instead — exactly the flow
  [worktrees](../agent-guidance/worktrees.md) prescribes: branch from
  `origin/main`, commit there, open a PR.
- If you do want a human gate, set 1 approval and also enable **Dismiss
  stale pull request approvals when new commits are pushed** and
  **Require conversation resolution**.

No CI yet? "Require a PR" alone still buys reviewable, revertable diffs
and no direct pushes — read the diff, merge by hand. Adding one cheap
always-run check (lint or build, no path filters) gives step 2 something
to require.

## 2. Require status checks to pass

Enable **Require status checks to pass**, select the checks your CI
always produces, and enable **Require branches to be up to date before
merging**:

- Only code that passes CI can land, and the agent cannot merge on top
  of a stale `main` and clobber parallel work.
- Only require checks that run on every PR. A workflow with a path or
  branch filter never reports its check for PRs outside the filter, so
  the PR can never merge. (Job-level `if:` skips are fine — a skipped
  job reports success; it's workflow-level filters that deadlock.)
- Up-to-date-branch re-runs full CI whenever `main` moves. Once PR
  volume makes that churn painful, GitHub's merge queue replaces it —
  add a `merge_group` trigger to your CI first, or required checks
  never report.

## 3. Block force pushes and branch deletions

Both are blocked by default in rulesets and in classic protection — the
rule is to **not turn them off**:

- No force pushes: history stays append-only. An agent recovering from
  a mistake may try `git push --force`; blocking it means a bad commit
  is reverted, never erased.
- Restrict deletions: a runaway agent session can't delete other
  sessions' worktree branches.

`main`'s ruleset covers only `main`. Add a second ruleset targeting all
branches — just these two rules, no PR or check requirements — so
agent-created branches are append-only too.

## 4. Protect release tags

Tags and releases sit outside branch protection, and a Contents:write
token can create both: mint a `v9.9.9` tag on any commit, publish the
release, and every workflow that runs on releases — this repo pushes
its Docker images that way — fires with no human in the loop.

Add a **tag ruleset** for `v*`: restrict deletions and updates, and
restrict creations with your release bot in the bypass list —
release-please (or whatever tags your releases) must still be able to
create tags at merge time, or the release flow deadlocks.

## 5. Block leaked secrets at push time

Enable **secret scanning with push protection** (Settings → Code
security; on by default for public repos, Advanced Security on private
ones). It rejects a push that contains a live credential before it lands
anywhere — the only mechanical enforcement of "never commit secrets",
since a revert can't unpublish a leaked key.

## 6. Require code-owner review on the dangerous paths

At 0 approvals the agent is author *and* merger of ordinary PRs. Keep
that convenience, and gate the few files that poison everything else
(`.github/` workflows, release/image manifests, deploy configs): add a
CODEOWNERS file listing your account for those paths, then enable
**Require review from Code owners** in the ruleset. Everyday agent PRs
stay approvals-0; only PRs touching owned paths need a human click.
Keep release-owned version pins out of CODEOWNERS so bot PRs stay
friction-free.

## 7. Keep bot PRs mergeable

The release PR is an ordinary PR: it must pass the same required checks.
Two GitHub behaviors decide whether that works:

- A PR created with the repo's own `GITHUB_TOKEN` has its workflow runs
  held behind a manual "Approve workflows to run" click — it can't
  collect required checks on its own. The release App from
  [GitHub repository setup](repo-setup.md) avoids that gate; keep it.
- Docs-only commits after a release PR opens leave it stale ("not up to
  date"), and the App only rebases it when the release content changes —
  click **Update branch** when you merge.

## Leave off

- **Require signed commits** — sandboxed agent commits (and the release
  bot's) aren't signed; enabling it makes every PR unmergeable.
- **Lock branch / restrict branch creation** — read-only machinery for
  frozen release branches, not an active default.

## The agent view

| Policy | What it does for you when an agent works in the repo |
| ------ | ----------------------------------------------------- |
| Require a PR | the agent's work is always a reviewable, revertable diff — nothing lands unseen |
| Required checks | the agent cannot merge code that fails CI; you review intent, CI reviews correctness |
| Up-to-date branch | the agent cannot merge on top of a stale `main` and clobber parallel work |
| No force pushes, on every branch | history is append-only: mistakes are reverted, never erased |
| Tag ruleset on `v*` | the agent cannot mint or move the refs your releases and pins depend on |
| Secret scanning + push protection | a leaked key is blocked at push time, not discovered in history later |
| Code-owner review | a human signs off exactly where damage would be repo-wide |
| Approvals = 0 elsewhere | checks gate the merge, not you — you review when you want to |

Combined with the [worktree workflow](../agent-guidance/worktrees.md)
(every change in its own branch off `origin/main`), this means a runaway
agent session costs you at most a branch to delete.

# Branch policies for working with an AI agent

Autonomous agents commit fast, commit often, and can push anywhere your
token can. Branch protection on GitHub is the safety net that makes "the
agent broke `main`" impossible: every change — the agent's and yours — has
to pass through a PR that CI validates before it can land.

Set these once as a repo admin (Settings are admin-only). If you already
follow [GitHub repository setup](repo-setup.md) this is the same screen;
this guide explains what to require and why it matters when an agent works
in your repo.

## 1. Require a pull request before merging

Settings → Branches → **Add branch ruleset** (or classic branch
protection) for `main`:

- **Require a pull request before merging.**
- **Required approvals: 0** (or 1 if you also want a human gate). An agent
  cannot approve PRs — with approvals required, you are the bottleneck; set
  to 0 and let required checks decide.
- Optionally **Dismiss stale pull request approvals when new commits are
  pushed** — keeps a human "LGTM" honest when the agent force-pushes more
  commits to the branch afterwards.

Direct pushes are where agent work becomes unreviewable: the agent pushes
to `main` mid-session and nobody sees the diff until something breaks. With
"require a PR" on, the agent's push to `main` is rejected and it must open a
PR instead — which is exactly the flow
[worktrees](../agent-guidance/worktrees.md) prescribes: branch from
`origin/main`, commit there, open a PR.

## 2. Require status checks to pass

Select the checks your CI produces, then **require branches to be up to date
before merging**.

- The agent can push ten commits an hour; up-to-date-branch + required
  checks mean only code that builds and passes CI can land.
- Pick a fixed set of checks that always runs. If a check only runs when
  matching files change, an agent PR touching other files will never produce
  it and the PR cannot merge — pick checks that run on every PR (this
  repo's Validate workflow is designed that way,
  see [GitHub repository setup](repo-setup.md)).

## 3. Block force pushes and deletions

- **Do not allow force pushes.** An agent recovering from a mistake may try
  to `git push --force`; blocking it keeps history append-only — a bad
  commit is reverted, not erased.
- **Restrict deletions** (default in rulesets).

## 4. Keep the release bot's PRs mergeable

The release PR is an ordinary PR: it must pass the same required checks.
Since the release PR is created by a GitHub App
([why](repo-setup.md#1-create-the-release-please-github-app)), and a
`GITHUB_TOKEN`-authored PR would never trigger the checks it needs, keep the
App setup from [GitHub repository setup](repo-setup.md) in place — otherwise
the release flow deadlocks behind the protection you just added.

## The agent view

| Policy | What it does for you when an agent works in the repo |
| ------ | ----------------------------------------------------- |
| Require a PR | the agent's work is always a reviewable, revertable diff — nothing lands unseen |
| Required checks | the agent cannot merge code that fails CI; you review intent, CI reviews correctness |
| Up-to-date branch | the agent cannot merge on top of a stale `main` and clobber parallel work |
| No force pushes | history is append-only: mistakes are reverted, never erased |
| Required reviews = 0 | checks gate the merge, not you — you review when you want to |

Combined with the [worktree workflow](../agent-guidance/worktrees.md)
(every change in its own branch off `origin/main`), this means a runaway
agent session costs you at most a branch to delete.

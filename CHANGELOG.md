## 0.2.0 · 2026-11-27

- Triage by Haiku: issues of one project with the same cause grouped, with **Merge in Sentry**.
- Issues with nothing to fix (a deployment, a local environment, a test) set aside with their reason, to resolve or ignore one by one or all at once; they get no automatic analysis.
- **Not seen for [n] days › List**: the unresolved issues with no event for that many days, the number kept per project, to resolve when you choose.
- The analysis now writes a diagnosis, a numbered plan and what to check. **Analyse and plan** runs it on any issue when automatic analysis is off.
- **Fix with an agent** asks the agent to show its plan and wait for your go-ahead before changing any file.
- Merging and resolving all ask for a second click.

## 0.1.0 · 2026-11-24

- First version: the unresolved Sentry errors of the projects you follow, in a panel next to your agents.
- Each new or returning error brings a notification and a fix proposed by Claude Code, which reads the repository without changing it: a diagnosis, a diff and what to check.
- Fix with an agent opens New agent with the error, the proposal and the stack trace; Resolve and Ignore act on Sentry.
- Agents can read the errors and their stack traces with `sentry_issues` and `sentry_issue`.

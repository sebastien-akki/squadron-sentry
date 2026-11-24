## 0.1.0 · 2026-11-24

- First version: the unresolved Sentry errors of the projects you follow, in a panel next to your agents.
- Each new or returning error brings a notification and a fix proposed by Claude Code, which reads the repository without changing it: a diagnosis, a diff and what to check.
- Fix with an agent opens New agent with the error, the proposal and the stack trace; Resolve and Ignore act on Sentry.
- Agents can read the errors and their stack traces with `sentry_issues` and `sentry_issue`.

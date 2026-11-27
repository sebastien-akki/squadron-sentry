# Sentry

The Sentry errors of each project next to your agents: sorted by Haiku, analysed by Claude Code with a plan when you want it, and no agent started without you.

## Use

- In Settings › Extensions › Sentry, add your organization: the server (`sentry.io`, `de.sentry.io` for the EU, or your own), the organization slug and a personal token with `event:read`, `event:write` and `org:read`. **Test the connection** checks it.
- Open **Sentry** in an agent's toolbar and **Choose Sentry projects** to follow for this Squadron project.
- Every few minutes (5 by default), the extension reads the unresolved issues of the followed projects. The first look only remembers what is already there. After that, each **new** issue, and each known issue that **came back** (regressed), brings a system notification and a badge on the Sentry button.

### The list

- At the top, the total of unresolved errors of the followed projects over 14 days (Sentry's count, beyond the 100 shown), and **Sort by** last seen, newest, most events or most users.
- Each error shows its key, title, project, users and last seen, then its **events** and its **age** (since first seen) in columns, and **Resolve** and **Ignore** on its right. A click on it opens its detail: events, users, first and last seen with their dates, status, level, project, the analysis and the stack trace.
- Check boxes select errors, or all of them from the header. A bar then gives **Resolve**, **Ignore** and **Merge in Sentry** for the selection; merging needs at least two errors of one Sentry project, and a second click.

### Triage

- When the list of open issues changes, and at most once an hour otherwise, Haiku reads one line per issue (title, culprit, environment, release, URL, volume, first and last seen) without any tool, and answers:
  - **Same subject**: issues of one Sentry project that share a cause. **Tick these errors** selects them, to merge them from the selection bar.
  - **Nothing to fix**: an error of a deployment (`MEP`), of a local or development environment, of a test, or another obvious case such as a robot, each with its reason. **Resolve**, **Ignore**, or **Resolve all** after a second click.
- These issues are tagged in the list, and their notification says *Probably nothing to fix*. They get no automatic analysis.
- **Sort again** runs the triage at once.
- **Not seen for [n] days › List** shows the unresolved issues of this project with no event for that many days, up to 89. The number is kept per Squadron project, 30 at first. **Resolve** one, or **Resolve all** after a second click. Nothing is resolved without you; an issue that comes back reopens in Sentry.

### Analysis and plan

- With **Analyse each new or returning error** on, Claude Code runs for each of them in the project's folder with read-only tools (`Read`, `Grep`, `Glob`), reads the code around the stack trace, and writes a **diagnosis**, a numbered **plan** (files, changes, test, and a diff when the fix is a few lines) and **what to check**. Nothing is changed in the repository. It takes about half a minute; an analysis that has not finished after 2 minutes is dropped.
- With it off, or for any issue, **Analyse and plan** in the issue does the same when you ask.
- **Fix with an agent** opens the New agent window in this project with the error and the plan as its first message, the stack trace as context, and a link to the issue on the agent's row. Nothing starts until you start it, and the agent is asked to show its plan and wait for your go-ahead before changing any file.
- Agents can call `sentry_issues` and `sentry_issue` to read the errors of their project and their stack traces. Claude Code asks you first.
- Settings: automatic analysis (off by default, so that several people following one Sentry project do not each analyse every error; one of them can turn it on), the model of the analyses (Sonnet by default), how often to check Sentry, and which environments to watch (all by default).

Each analysis and each triage runs Claude Code with your own plan; a triage is one short Haiku call.

## What it asks Squadron for

- **Run `claude`**: the Claude Code set up in Squadron, in print mode: with the `Read`, `Grep` and `Glob` tools only for the analyses, and with no tool, no MCP server and no settings file for the triage.
- **Network**: only the Sentry server of each account you add, with its token, which stays in your system keyring. The triage reads the latest event of each issue once.

## Build

```sh
rustup target add wasm32-wasip2
cargo test
cargo build --release --target wasm32-wasip2
```

Then in Squadron: Settings › Extensions › Add an extension › A folder on this computer.

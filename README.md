# Sentry

The Sentry errors of each project next to your agents, each new or returning error with a fix proposed by Claude Code.

## Use

- In Settings › Extensions › Sentry, add your organization: the server (`sentry.io`, `de.sentry.io` for the EU, or your own), the organization slug and a personal token with `event:read`, `event:write` and `org:read`. **Test the connection** checks it.
- Open **Sentry** in an agent's toolbar and **Choose Sentry projects** to follow for this Squadron project.
- Every few minutes (5 by default), the extension reads the unresolved issues of the followed projects. The first look only remembers what is already there. After that, each **new** issue, and each known issue that **came back** (regressed), brings a system notification and a badge on the Sentry button.
- For each of them, Claude Code runs in the project's folder with read-only tools (`Read`, `Grep`, `Glob`), reads the code around the stack trace, and writes a **diagnosis**, a **proposed fix** as a diff, and **what to check**. Nothing is changed in the repository. It takes about half a minute; a proposal that has not finished after 2 minutes is dropped.
- In the panel, an issue shows its proposal, its stack trace with the code around each line, and:
  - **Fix with an agent**: opens New agent in this project with the error and the proposal as its first message, the stack trace as context, and a link to the issue on the agent's row;
  - **Propose again**, **Resolve** and **Ignore**, which act on Sentry.
- Agents can call `sentry_issues` and `sentry_issue` to read the errors of their project and their stack traces. Claude Code asks you first.
- Settings: automatic proposals (on by default), the model (Sonnet by default), how often to check Sentry, and which environments to watch (all by default).

Each proposal runs Claude Code with your own plan. Turn automatic proposals off to propose only on demand.

## What it asks Squadron for

- **Run `claude`**: the Claude Code set up in Squadron, in print mode with the `Read`, `Grep` and `Glob` tools only, to write the proposals.
- **Network**: only the Sentry server of each account you add, with its token, which stays in your system keyring.

## Build

```sh
rustup target add wasm32-wasip2
cargo test
cargo build --release --target wasm32-wasip2
```

Then in Squadron: Settings › Extensions › Add an extension › A folder on this computer.

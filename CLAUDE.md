## Conventions

Run `cargo fmt --all` after changing Rust code. A Stop hook in `.claude/settings.json` also does this automatically at the end of each turn.

Linux-only code can leave warnings or errors that only show on Windows. After changing `cfg`-gated or platform-specific code, run `cargo check --target x86_64-pc-windows-gnu --workspace --all-targets`. Setup is in `scripts/windows-vm/README.md`.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

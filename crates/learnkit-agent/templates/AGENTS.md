# LearnKit — Codex agent rules

This project uses LearnKit. Codex MUST follow this workflow:

- Before any LearnKit task, run `learnkit status --json` to see the real project state.
- Never bypass a failed guard, and never edit `.learnkit/` validation manifests or state files by hand to "unblock" a phase.
- Use the relevant LearnKit skill under `.agents/skills/` for guidance, but treat the CLI as the source of truth.
- Finish any change that affects a workflow phase by running `learnkit validate` (or the relevant `learnkit run` command) and reacting to its result — do not assume success.

See `learnkit-implementation-spec/docs/09-agent-integration.md` for the full rationale.

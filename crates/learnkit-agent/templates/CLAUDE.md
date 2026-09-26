# LearnKit — Claude agent rules

This project uses LearnKit. Follow the same workflow as any other agent:

- Check `learnkit status --json` before starting a task.
- Never edit `.learnkit/` state or validation manifests directly to make a phase pass.
- Use the LearnKit skills under `.claude/skills/` for guidance; the CLI's guards are the real enforcement.
- Finish with `learnkit validate` (or the relevant `learnkit run` command) and act on its result.

See `learnkit-implementation-spec/docs/09-agent-integration.md` for details.

---
name: learnkit-session
description: Start, inspect, and inventory a LearnKit session for the current project.
---

# LearnKit session

Use this skill when the user wants to start or continue a LearnKit session.

1. Run `learnkit status --json`. If the project is not initialized, tell the user and stop.
2. Run `learnkit session list` / `learnkit session show` to see existing sessions before creating a new one.
3. Never mark a phase as complete yourself — always run the corresponding `learnkit` command and read its result.
4. If a command reports a failure (`"ok": false`), fix the underlying artifact and re-run the same command; do not edit `.learnkit/` files directly.

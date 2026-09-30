---
name: learnkit-session
description: Orchestrate the lifecycle of a LearnKit session. Resolve the active session, inspect its state and sources, run mechanical prerequisites, and delegate specialised work to the appropriate LearnKit skills without duplicating their responsibilities.
---

# LearnKit session — session orchestrator

Use this skill when the user wants to start, continue, inspect, process, or resume work on a LearnKit session.

This skill coordinates the session lifecycle.

It does not replace specialised skills such as `learnkit-analyse`, `learnkit-language`, or `learnkit-image-prompts`.

Its job is to determine the current state, identify the next required action, execute mechanical LearnKit operations when appropriate, and delegate specialised judgement to the correct skill.

## 1. Verify the project

Run:

`learnkit status --json`

If the current directory is not an initialised LearnKit project, report that clearly and stop.

Never create or edit `.learnkit/` files directly.

All state changes must go through supported `learnkit` commands.

## 2. Resolve the session

Before creating a new session, inspect existing sessions.

Use structured output where available.

Run the appropriate session listing/status commands and determine whether the user's request refers to:

- an explicitly named or identified existing session;
- the clearly active/relevant existing session;
- a genuinely new class or study session.

Do not create a new session merely because the user did not provide a `session_id`.

Reuse an existing session when the user's intent clearly refers to it.

If multiple sessions are genuinely plausible and choosing incorrectly could mix unrelated material, ask the user to choose.

## 3. Create a session only when needed

When the user is clearly starting a new class/session and no existing session corresponds to it, create one with:

`learnkit session new "<title>"`

Use a clear title derived from the user's supplied context.

Do not create duplicate sessions for the same class merely because processing is resumed later.

After creation, retain the returned `session_id` for subsequent commands.

## 4. Inspect the current session state

Once the session is resolved, inspect both:

`learnkit status --session <session_id> --json`

and, when its material is relevant:

`learnkit session show <session_id> --json`

Use the status as the source of truth for phase/item state.

Do not infer that a phase is complete merely because files exist.

Do not mark phases complete manually.

### Accumulated level

Once per session, right after resolving the session and before deciding what to do with its material, run:

`learnkit learn level show`

(The level is project-level, not per-session — it takes no `--session` flag.)

Do this even when nothing else in this step depends on the result. Recording the level here means no downstream skill (`learnkit-analyse`, `learnkit-language`) has to rediscover it on its own.

If no level is stored yet, continue with an unknown level, exactly as `learnkit-language` section 21 already specifies. Do not repeat that logic here — just make sure the check happens.

## 5. Inspect sources and inventory

Determine whether the session already contains the material required for the user's request.

Sources may include:

- notes;
- audio;
- images;
- transcripts;
- worksheets;
- other class material.

When new files have been supplied for the session, ingest them using the supported CLI and run:

`learnkit inventory --session <session_id>`

Inventory is idempotent. Re-run it when source material may have changed.

Never modify the inventory or source metadata manually.

## 6. Resolve transcription prerequisites

If the requested workflow requires understanding audio content, verify whether usable transcripts already exist.

When audio requires transcription:

- use LearnKit's transcription workflow when the required local tooling is available; or
- import an existing transcript when the user has supplied one.

Do not fabricate transcript content.

Do not treat an audio source as analysed merely because it has been inventoried.

When a transcript contains uncertain or unintelligible segments, preserve that uncertainty for downstream skills.

## 7. Determine the next workflow action

Use the user's actual goal together with the current session state.

Examples:

### User wants to analyse the class

Ensure required sources are inventoried and required audio is transcribed.

Then delegate the pedagogical analysis to `learnkit-analyse`.

Do not reproduce its analysis logic here.

### User wants vocabulary

Ensure the session material is available.

Delegate lexical extraction and confirmation to `learnkit-language`.

Do not automatically persist vocabulary from this orchestrator.

### User wants cards or Anki

Check the prerequisites required by the card workflow.

Do not silently invent vocabulary merely to unblock card generation.

Use the existing confirmed learning items.

Delegate image judgement to the appropriate image skill when needed.

### User wants an exam

Check the prerequisites required by the assessment workflow and invoke the supported LearnKit commands.

### User only wants to add new class material

Ingest and inventory the material.

Do not automatically run analysis, vocabulary extraction, cards, or assessments unless the user's request implies that broader processing.

### User supplies new class material without naming a specific phase

When the user hands over new class documentation and does not ask for a specific named phase, propose — do not silently run — the default pipeline for this material:

1. `learnkit-analyse` — summary, mind map, concept pages (including grammar).
2. `learnkit-language` — vocabulary, dialogues, pronunciation. For any grammar page, dialogue, or minimal pair the user confirms is worth a flashcard, that skill uses `learn item promote` followed by `cards set`.
3. `learnkit cards build`.
4. The exporters, offered explicitly by name: `export study-guide` (Markdown study guide), `export study-pack` (10-tab Excel workbook), `export anki` (Anki `.apkg` package), and `export exam` (self-contained HTML exam). Ask which of these the user wants generated — never run them without saying so first.

This is a suggested sequence, not a mandatory one. If the user asks for one specific phase, do only that phase.

## 8. Respect completed and pending work

LearnKit operations are designed to be resumable and idempotent.

Do not redo work that the current status reports as complete and current unless:

- the source material changed;
- the user explicitly requests regeneration;
- the corresponding workflow determines that an update is necessary.

When an item is `pending`, continue the appropriate workflow.

When an item is `pending_user_decision`, do not silently skip it or fabricate content.

Surface the stored reason and obtain the required explicit decision when necessary.

Independent work that is not blocked may continue when the LearnKit workflow permits it.

## 9. Handle command failures by category

Whenever possible, use structured JSON output and inspect the error code/state.

Do not treat every `ok: false` as a broken generated file.

Classify the failure first.

Typical cases include:

### Missing prerequisite

Run the missing supported prerequisite when it is safe and unambiguous.

Then retry the original operation.

### Invalid generated artifact

Correct the generated artifact using the skill responsible for producing it, then retry its LearnKit command.

### Pending user decision

Do not resolve it automatically.

Explain the concrete pending decision.

### External dependency failure

Examples include unavailable transcription tooling, network access, or TTS failure.

Report the actual dependency failure and preserve the session state.

### Unsupported or inconsistent state

Do not edit `.learnkit/` directly to force progress.

Report the structured error and use only supported recovery commands.

## 10. Preserve source boundaries

Never mix source material from unrelated LearnKit sessions.

Never move learning content between sessions merely because the topics appear similar.

A session is the boundary for the class material being processed unless the user explicitly requests another operation supported by LearnKit.

## 11. Re-check state after mutations

After meaningful state-changing operations, use the relevant LearnKit status/read command to verify the resulting state.

Do not claim that an operation completed based only on intention or file creation.

The persisted LearnKit state is authoritative.

## 12. Report the session state

After completing the requested work, give a concise report containing the relevant parts of:

- resolved `session_id`;
- sources added or updated;
- transcription state;
- analysis state;
- delegated workflow completed;
- pending items;
- explicit user decisions still required;
- external dependency failures, if any.

Do not dump the complete internal checklist unless it is useful to the user.

## Core orchestration principle

The orchestrator answers:

> What does this session need next to satisfy the user's request?

It should not answer:

> How should every specialised LearnKit artefact be generated?

That responsibility belongs to the specialised skills.

Prefer:

**inspect → resolve prerequisites → delegate → verify**

over duplicating domain logic inside this skill.

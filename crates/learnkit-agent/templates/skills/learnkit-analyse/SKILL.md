---
name: learnkit-analyse
description: Analyse a LearnKit session from its inventoried material and produce a source-grounded class summary, quick-review study map, and concept pages. Handles uncertainty, source conflicts, declared knowledge gaps, incremental updates, and pedagogical quality control. Subject-agnostic; language-specific extraction belongs in learnkit-language.
---

# LearnKit analyse — summary, study map, and concept pages

Use this skill when the user wants to analyse a LearnKit session in any subject.

Its purpose is not merely to shorten the source material. It transforms notes, transcripts, and other inventoried material into study-ready resources while preserving traceability to what actually happened in the session.

This skill is deliberately subject-agnostic.

Do not add language-learning-specific processing such as lexical extraction, IPA, pronunciation, translations, or Anki vocabulary generation here. Those belong in `learnkit-language`, which may run independently against the same session.

## 1. Inspect the session and its current state

Run:

`learnkit session show --session <session_id> --json`

Read `text_material`, including the session notes and transcript segments associated with audio sources.

While reading, preserve source distinctions whenever they are available. In particular, distinguish information originating from:

- written notes;
- each individual audio transcript;
- timestamped transcript segments;
- corrections or later clarifications;
- other inventoried source material.

Do not flatten conflicting sources into a single interpretation before analysing them.

Then run:

`learnkit status --session <session_id> --json`

Read the `analyse.checklist`.

Each item may be:

- `done` — confirmed and still current. Do not regenerate it.
- `pending` — missing, unconfirmed, or invalidated because source material changed. Draft or redraft it.
- `pending_user_decision` — deliberately blocked because material is missing or a previous decision is required. Read `pending_reason` and do not silently resolve it.

Continue processing independent items even when another item is blocked.

Never redo a `done` item simply to make the whole analysis look uniform.

## 2. Apply an evidence model

Treat information in the analysis as belonging to one of four categories.

### Source-backed

The information is directly supported by the session material.

Prefer this whenever possible.

When timestamps, source names, or other useful anchors exist, retain enough context in the drafted material to make important claims easy to trace back to their origin.

### Filled gap

The session clearly introduces or relies on a concept but does not explain enough of it to make the study material useful.

You may add limited background knowledge when it is stable, relevant, and pedagogically necessary.

Never present added knowledge as though the teacher, notes, or transcript contained it.

Declare every such addition using the CLI's gap mechanism wherever that mechanism is supported.

### Uncertain

The source is ambiguous, incomplete, inaudible, badly transcribed, or otherwise unreliable.

Preserve the uncertainty.

Do not convert a guess into study material.

### Conflicting

Two source elements disagree.

Do not silently choose whichever version seems more plausible.

If the session itself clearly contains a later correction, prefer the corrected version and preserve the fact that a correction occurred when pedagogically useful.

Otherwise state the disagreement or leave the affected item pending.

## 3. Select what is worth studying

Do not treat every sentence in the session as equally important.

Prioritise material that increases the learner's ability to understand, remember, apply, or explain what was taught.

Typical high-value material includes:

- new concepts;
- important relationships;
- rules or procedures;
- distinctions that are easy to confuse;
- teacher corrections;
- learner mistakes or misconceptions with pedagogical value;
- representative examples;
- useful terminology and definitions;
- questions that revealed an important concept;
- exceptions;
- recurring difficulties;
- ideas that connect several parts of the session.

Normally omit:

- repetition with no additional value;
- administrative conversation;
- irrelevant tangents;
- redundant examples;
- transcription noise;
- trivial mistakes immediately corrected when they have no learning value.

## 4. Draft the ClassSummary

Create a `ClassSummary` that allows the learner to review the session without rereading the complete notes or transcript.

It should normally cover, when relevant:

- session focus;
- key ideas;
- important concepts and relationships;
- rules, procedures, or frameworks;
- representative examples;
- corrections, mistakes, or misconceptions worth reviewing;
- important terminology or definitions;
- unresolved questions or uncertainties;
- recommended review points.

The summary must preserve the logical structure of the session rather than merely compressing the transcript chronologically.

Clearly separate an incorrect form, assumption, or solution from its correction.

Do not fabricate completeness. If an important part of the session cannot be reconstructed reliably, say so or mark the element pending.

## 5. Draft the StudyMap

Create a `StudyMap` for rapid review.

It must be structurally different from the summary.

Use a hierarchy of concise nodes showing relationships such as:

- main topic → subtopics;
- concept → properties;
- rule → conditions → exceptions;
- process → stages;
- category → members;
- problem → causes → consequences;
- concept A ↔ concept B;
- misconception → correction.

Prefer short nodes and meaningful relationships over explanatory prose.

A study map should help the learner reconstruct the session mentally at a glance.

It must not merely be the ClassSummary reformatted as bullets.

## 6. Draft ConceptPages

Create a `ConceptPage` only for a substantial idea that benefits from a standalone explanation.

Do not create one page per term or artificially fragment closely related material.

A concept page should be useful outside the chronological context of the session while remaining faithful to what was actually studied.

When relevant, it may contain:

- concept name;
- concise explanation;
- why it matters;
- structure, rule, or mechanism;
- relationship to nearby concepts;
- representative examples;
- common confusion, mistake, or misconception;
- correction or distinction;
- review cue;
- useful source anchors.

Avoid duplicating the complete ClassSummary inside concept pages.

If two concepts are inseparable for learning purposes, prefer one coherent page rather than two thin pages.

## 7. Fill gaps conservatively

If the source mentions an important concept without explaining it sufficiently, limited external knowledge may be added only when necessary to make the study material coherent.

For `ClassSummary`, declare each addition using:

`--filled-gap "<concept>:<note>"`

Both the concept and note must be non-empty.

The note should briefly state what was added and why.

Do not use gap filling to:

- invent what the teacher probably meant;
- repair an uncertain transcript by guessing;
- introduce large amounts of material that were never part of the session;
- silently replace a source explanation with a preferred explanation;
- expand the session into a general textbook chapter.

`--filled-gap` works the same way for every item kind (`summary`, `mindmap`, or a `page:<id>`) on `analyse set` — it is never mixed between items.

## 8. Handle insufficient material explicitly

If an element genuinely cannot be produced from the available material, do not create empty, generic, or fabricated content merely to make the checklist complete.

Use:

`learnkit analyse flag-pending <item_id> --session <session_id> --reason "..."`

where `<item_id>` is `summary`, `mindmap`, or the relevant `page:<stable_id>`.

The reason must describe the concrete missing evidence.

Examples include:

- transcript missing;
- relevant audio segment unintelligible;
- notes refer to an exercise whose contents are absent;
- source conflict cannot be resolved;
- material is too thin to identify the concept reliably.

## 9. Perform pedagogical quality control

Before confirming an artefact, check that:

- it is faithful to the session material;
- important concepts have not been omitted without reason;
- uncertainty is visible rather than guessed away;
- source conflicts are not silently resolved;
- mistakes and corrections are clearly distinguished;
- examples actually illustrate the intended concept;
- concepts are not unnecessarily duplicated across pages;
- the summary is explanatory rather than a shortened transcript;
- the StudyMap is genuinely hierarchical and concise;
- ConceptPages are substantial enough to justify their existence;
- externally supplied knowledge is explicitly declared where supported;
- no unsupported information has been presented as session content.

Only confirm the artefact after these checks pass.

## 10. Persist each completed artefact

Write each drafted artefact to a file and confirm it through one command, `analyse set`, addressed by a stable `item_id`:

`learnkit analyse set --session <session_id> --item <item_id> --file <file> [--concept <display_name>] [--filled-gap "<concept>:<note>"]... [--force]`

`item_id` is `summary`, `mindmap`, or `page:<stable_id>` — you choose the `<stable_id>` for a concept page (e.g. `page:layover`, `page:present-perfect`) and reuse the same one every time you update that page; it is never positional and never derived from the display name.

Summary:

`learnkit analyse set --session <session_id> --item summary --file <summary.md> [--filled-gap "<concept>:<note>"]...`

Study map:

`learnkit analyse set --session <session_id> --item mindmap --file <map.md>`

Concept page (new or updated):

`learnkit analyse set --session <session_id> --item page:<stable_id> --concept "<concept>" --file <page.md>`

These calls validate and persist the artefact; they do not generate or judge its prose.

Respect idempotency. `analyse set` is an idempotent upsert for every item kind, including `page:<id>`:

- The item doesn't exist yet → it is created and confirmed. `--concept` is required the first time you create a `page:<id>`.
- The item exists but is `pending` because its source changed → re-running `analyse set` with the same `item_id` updates that same item in place (no `--force` needed) — it never creates a duplicate like `page:<id>-2`.
- The item is already `done` and nothing changed → a no-op; the command reports `already_done` rather than re-writing anything.
- You want to deliberately replace an already-`done`, unchanged item's content → pass `--force`.

### Existing ConceptPages

`analyse set --item page:<id>` is the only way to create or update a concept page — there is no separate "create" command. To rename a page's display name without changing its identity, call `analyse set` again with the same `--item page:<id>` and a new `--concept` (`--concept` can be omitted on an update — it keeps the existing display name).

If an existing concept page becomes `pending` because its source changed, re-run `analyse set` with the same `page:<id>` to update it in place — never create a new `page:<id>-2` as a substitute for updating it.

## 11. Recheck the final state

After persisting all possible artefacts, run:

`learnkit status --session <session_id> --json`

Verify that each item you intended to complete is actually `done`.

Do not assume that a successful-looking command means the session state is correct.

Check for newly pending or unresolved items before reporting completion.

## 12. Report back to the person

Give a concise completion report in the conversation.

Include:

- which artefacts were confirmed;
- which existing artefacts were left unchanged because they were already current;
- which concept pages were created or updated;
- every declared filled gap;
- any important source uncertainty or conflict;
- every item left pending and its reason.

When a `pending_user_decision` remains, explain the concrete choice.

Where the CLI supports it, the alternatives may include supplying more source material or explicitly skipping the item:

`learnkit analyse skip <item_id> --session <session_id> --reason "..."`

Skipping must always be an explicit decision.

Never silently convert an unresolved item into `done`.

---
name: learnkit-analyse
description: Draft a session's class summary, mind map, and concept pages from its inventoried material, filling declared gaps and confirming each piece with LearnKit's CLI. Subject-agnostic — no language/domain-specific instructions.
---

# LearnKit analyse — summary, mind map, and concept pages

Use this skill when the user wants to analyse a LearnKit session (any
subject, not just languages) — turning its inventoried notes/transcript into
a study-ready summary, mind map, and one or more concept pages.

This skill is deliberately generic: it never assumes a language-learning
session. For language-specific vocabulary extraction, see the
`learnkit-language` skill instead — the two are independent and can both run
against the same session.

## 1. Read what is already known

1. Run `learnkit session show --session <session_id> --json` and read
   `text_material`: the session's notes plus, for each audio source, its
   transcript segments.
2. **Also check every source of type `image`.** `session show --json`'s
   `sources` array lists every inventoried file with its `kind` and `path` —
   `text_material` only covers notes/transcripts, it never includes what's in
   a photo. Before treating step 1 as complete, open and read each
   `sources[].kind == "image"` file directly (e.g. a photo of a blackboard,
   handout, or notebook page) — do not skip this because `text_material`
   already looks substantial. Found in real use: a summary was drafted from
   only the text material and a narrative digest of the photos, and two
   complete vocabulary tables that existed only in the photos were silently
   dropped as a result.
3. If you delegate reading the source material (notes, transcript, or
   images) to a subagent instead of reading it yourself, require it to
   return the literal, structured content it found — full tables verbatim,
   full lists, exact wording — never just a narrative summary of what it
   saw. A narrative digest is exactly what caused the dropped tables above;
   treat any subagent report that only paraphrases as incomplete and ask it
   to re-read and return the literal content instead.
4. Run `learnkit status --session <session_id> --json` and look at the
   `analyse` entry's `checklist` array. Each item (`summary`, `mindmap`,
   `page:<stable_id>`) already tells you its current state:
   - `done` — already confirmed and still current; do not redo it.
   - `pending` — not confirmed yet, or the source material changed since it
     was last confirmed; draft (or redraft) it.
   - `pending_user_decision` — a previous run marked it as blocked on missing
     material, with a `pending_reason` explaining what is missing (`skip`
     and `flag-pending` are used to reach this state and resolve it; if you
     don't see them as available commands yet, treat the item as still
     needing more material and tell the person, rather than inventing one).

## 2. Draft the content

From the material you read in step 1, draft:

- **A summary** (`ClassSummary`): what was worked on, the important
  concepts, rules, examples worth keeping, mistakes to review, and
  vocabulary to study.
- **A mind map / quick-review outline** (`StudyMap`): deliberately different
  in shape from the summary — a hierarchy of short nodes for fast review,
  never a restatement of the summary's prose. LearnKit's CLI rejects a
  mindmap that is byte-for-byte identical to the confirmed summary.
- **One or more concept pages** (`ConceptPage`): each one concentrates the
  explanation of a single idea actually worked on in the session.

### Filling a gap in the source material

If the material mentions a concept but never explains it, you may fill that
gap using your own knowledge — but you MUST declare it, never blend it
silently into content that looks like it came from the material. Declare it
with `--filled-gap "<concepto>:<nota>"` on `analyse set` (both the concept
and the note must be non-empty). `--filled-gap` works the same way for every
item kind (`summary`, `mindmap`, or a `page:<id>`) — it is never mixed
between items.

### When the material is not enough

If an element genuinely cannot be drafted from the available material (for
example, a source that is empty or too thin even to summarize), do not
invent content just to make the item "pass". Use
`learnkit analyse flag-pending <item_id> --session <session_id> --reason "..."`
(`item_id` is `summary`, `mindmap`, or the relevant `page:<id>`) and tell the
person in the conversation what is missing — never fail silently and never
mark an empty or fabricated element as done.

## 3. Confirm each element via the CLI

The CLI never generates or judges prose — it only validates structure and
traceability, then persists. Every result of `analyse` is confirmed through
one command, `analyse set`, addressed by a stable `item_id`:

```
learnkit analyse set --session <session_id> --item <item_id> --file <file> \
  [--concept <display_name>] [--filled-gap "<concepto>:<nota>"]... [--force]
```

`item_id` is `summary`, `mindmap`, or `page:<stable_id>` — you choose the
`<stable_id>` for a concept page (e.g. `page:layover`, `page:present-perfect`)
and reuse the same one every time you update that page; it is never
positional and never derived from the display name.

- `learnkit analyse set --session <session_id> --item summary --file <resumen.md> [--filled-gap "<concepto>:<nota>"]...`
- `learnkit analyse set --session <session_id> --item mindmap --file <mapa.md>`
- `learnkit analyse set --session <session_id> --item page:<stable_id> --concept "<texto>" --file <pagina.md>`

`analyse set` is an idempotent upsert for every item kind, including
`page:<id>`:

- The item doesn't exist yet → it is created and confirmed. `--concept` is
  required the first time you create a `page:<id>`.
- The item exists but is `pending` because the source material changed →
  re-running `analyse set` with the same `item_id` updates that same item in
  place (no `--force` needed) — it never creates a second page like
  `page:<id>-2`.
- The item is already `done` and nothing changed → a no-op; the command
  reports `already_done` rather than re-writing anything.
- You want to deliberately replace an already-`done`, unchanged item's
  content → pass `--force`.

To rename a concept page's display name without changing its identity, call
`analyse set` again with the same `--item page:<id>` and a new `--concept`
(`--concept` can be omitted on an update — it keeps the existing display
name).

## 4. Report back to the person

Tell the person, in the conversation (never a terminal prompt):

- Which elements you confirmed (`summary`, `mindmap`, each `page:<id>`).
- Any gap you filled and declared via `--filled-gap`.
- Any element you could not draft and marked `flag-pending`, with the reason,
  and ask explicitly whether they'd rather provide more material or have you
  skip it (`learnkit analyse skip <item_id> --session <session_id> --reason "..."`)
  — the decision is always explicit, never silent.

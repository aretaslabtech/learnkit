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
2. Run `learnkit status --session <session_id> --json` and look at the
   `analyse` entry's `checklist` array. Each item (`summary`, `mindmap`,
   `page-<n>`) already tells you its current state:
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
with `--filled-gap "<concepto>:<nota>"` on `analyse summary set` (both the
concept and the note must be non-empty).

### When the material is not enough

If an element genuinely cannot be drafted from the available material (for
example, a source that is empty or too thin even to summarize), do not
invent content just to make the item "pass". Use
`learnkit analyse <item> flag-pending --session <session_id> --reason "..."`
(one of `summary`, `mindmap`, or the relevant `page-<n>`) and tell the person
in the conversation what is missing — never fail silently and never mark an
empty or fabricated element as done.

## 3. Confirm each element via the CLI

The CLI never generates or judges prose — it only validates structure and
traceability, then persists. Confirm each drafted element by writing it to a
file and calling:

- `learnkit analyse summary set --session <session_id> --file <resumen.md> [--filled-gap "<concepto>:<nota>"]...`
- `learnkit analyse mindmap set --session <session_id> --file <mapa.md>`
- `learnkit analyse page add --session <session_id> --concept "<texto>" --file <pagina.md>`

Each of these is idempotent: re-running it with unchanged source material and
without `--force` is a no-op (it reports the item is already done rather
than re-writing it). `analyse page add` is the exception — it always creates
a brand new, independent page, so call it once per concept page you draft,
not to update an existing one.

## 4. Report back to the person

Tell the person, in the conversation (never a terminal prompt):

- Which elements you confirmed (`summary`, `mindmap`, each `page-<n>`).
- Any gap you filled and declared via `--filled-gap`.
- Any element you could not draft and marked `flag-pending`, with the reason,
  and ask explicitly whether they'd rather provide more material or have you
  skip it (`learnkit analyse <item> skip --session <session_id> --reason "..."`)
  — the decision is always explicit, never silent.

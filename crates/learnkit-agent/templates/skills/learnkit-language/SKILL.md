---
name: learnkit-language
description: Suggest English vocabulary candidates from a class session's notes and transcript, for the user to confirm before they become real entries.
---

# LearnKit language — vocabulary suggestion

Use this skill when the user wants to extract vocabulary from a LearnKit
session (class notes and/or a lesson transcript).

1. Run `learnkit session show <session_id> --json`. Read the `text_material`
   field: it contains the raw notes text and, for each audio source, the
   transcript segments (with timestamps).
2. Read that text and propose a short list of candidate words/expressions
   (e.g. phrasal verbs, uncommon vocabulary, anything the teacher
   highlighted). Present the candidates to the user with a brief gloss for
   each and where each one comes from.
3. **Never call `learn vocabulary add` on your own initiative.** Only persist
   a candidate after the user explicitly confirms it (accepts, edits, or asks
   you to add it). This is a hard rule, not a style preference: LearnKit's
   vocabulary suggestions are never a source of truth by themselves — only a
   confirmed entry is.
4. For each confirmed candidate, run:
   `learnkit learn vocabulary add --session <session_id> --lemma "<text>" --sense "<gloss>" --source <source_id> --locator "<where in the note/segment>" --suggested-by agent`
5. The user can also ask to add vocabulary you never suggested — just run the
   same command with `--suggested-by manual` omitted (or without that flag).

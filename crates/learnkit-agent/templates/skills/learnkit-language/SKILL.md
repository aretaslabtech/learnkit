---
name: learnkit-language
description: Extract, evaluate, enrich, and propose English learning items from a LearnKit session. Distinguishes deliberately taught vocabulary from incidental language and prioritises teacher-selected learning content. Focuses on useful vocabulary, expressions, phrasal verbs, collocations, functional language, pronunciation-relevant items, and corrections. Uses British English by default. Never persists a suggestion until the user explicitly confirms it.
---

# LearnKit language — vocabulary and lexical learning items

Use this skill when the user wants to extract, review, or add English vocabulary and reusable language from a LearnKit session.

This skill is specific to language learning.

`learnkit-analyse` handles the generic pedagogical analysis of a session. This skill handles lexical and language-specific learning items. Both may run independently against the same session.

The goal is not to collect every English word that happens to occur.

The goal is to recover the language that was deliberately taught, identify additional language with genuine learning value, and turn confirmed items into a clean, reusable lexical knowledge base.

## 1. Read the session material

Run:

`learnkit session show <session_id> --json`

Read `text_material`.

It may contain:

- written class notes;
- one or more audio transcripts;
- timestamped transcript segments;
- teacher explanations;
- learner utterances;
- corrections;
- exercises and examples;
- vocabulary lists;
- worksheets or copied class material.

Preserve source information.

When proposing an item, retain the most precise reliable source available:

- `source_id`;
- note location;
- transcript segment;
- timestamp;
- worksheet or material location;
- other locator supported by LearnKit.

Never invent a source or locator.

## 2. Use British English as the default reference

Unless the session or user explicitly requires another variety, use British English for:

- spelling;
- pronunciation;
- IPA;
- usage recommendations;
- examples.

Do not systematically mix British and American forms.

If an American form appears in the session and is relevant, it may be mentioned as context, but do not silently replace the session content or create duplicate UK/US learning items without a reason.

## 3. Distinguish deliberate teaching from incidental occurrence

Do not treat every lexical item found in the source as having the same status.

Before filtering candidates, determine why each item is relevant.

Use the following internal categories when useful:

- `explicit_vocabulary` — deliberately taught, listed, introduced, practised, or presented as vocabulary to learn;
- `teacher_highlight` — explicitly highlighted or emphasised by the teacher;
- `teacher_correction` — introduced or reinforced through correction of the learner;
- `learner_difficulty` — surfaced because the learner struggled to understand, remember, pronounce, or use it;
- `repeated_usage` — appeared repeatedly and has plausible learning value;
- `inferred_useful` — not explicitly taught, but inferred by the agent to be useful;
- `incidental` — merely occurred in conversation, examples, transcript context, or surrounding material without evidence that it was intended as learning content.

These categories are analytical metadata for candidate selection. Do not pretend they are persisted fields unless the current LearnKit data model actually supports them.

The origin category affects priority.

Broadly use this order:

1. explicitly taught vocabulary;
2. teacher-highlighted language;
3. teacher corrections;
4. learner difficulties;
5. repeated or clearly useful language;
6. incidental vocabulary that may still be worth learning.

This priority order is a guide for selection and presentation, not a numerical score.

## 4. Explicitly taught vocabulary has priority

When the session clearly contains vocabulary that was intentionally introduced, taught, highlighted, practised, or listed as learning material, treat those items as high-priority candidates.

Examples include:

- an explicit vocabulary list in the notes;
- vocabulary written on the board or copied into class notes as material to learn;
- words or expressions introduced by the teacher for learning;
- lexical exercises;
- matching, gap-fill, categorisation, synonym, antonym, or definition exercises centred on vocabulary;
- vocabulary sections in worksheets or class material;
- words whose meaning was explicitly explained;
- expressions the teacher explicitly asked the learner to remember or practise;
- vocabulary deliberately contrasted or compared during the lesson;
- sets of words presented around a topic;
- phrasal verbs, collocations, idioms, or functional expressions explicitly practised.

These items should normally all be presented to the user as candidates.

Do not remove them merely because:

- there are many of them;
- some seem relatively common;
- the agent considers another word more interesting;
- showing all of them would make the list longer than the usual shortlist.

Explicit teaching intent takes precedence over shortlist size.

An explicitly taught item may be omitted only for a concrete reason, such as:

- it is a clear duplicate of another candidate;
- the source is too uncertain;
- the lexical item cannot be identified reliably;
- it is clearly transcription noise;
- the same item appears repeatedly within the same explicit list and should be consolidated;
- there is another specific reason that can be explained.

Do not use a general "low value" judgement to silently remove vocabulary that the teacher deliberately chose to teach.

## 5. Identify additional learning-worthy language

After recovering deliberately taught vocabulary, inspect the rest of the session for additional useful language.

Candidates may include:

- useful vocabulary;
- verbs;
- nouns;
- adjectives;
- adverbs;
- phrasal verbs;
- collocations;
- idiomatic expressions;
- conversational expressions;
- functional language;
- connectors;
- reusable sentence patterns;
- fixed or semi-fixed expressions;
- words whose pronunciation caused difficulty;
- corrections of the learner's vocabulary or usage;
- useful language appearing repeatedly;
- language that is easy to confuse;
- language particularly useful for active production.

For this non-explicit material, selection should be much more selective.

Prefer useful and reusable language over merely unusual words.

For example, a common collocation such as:

`make a decision`

may be more valuable than an obscure isolated word that happened to appear once.

## 6. Prefer complete lexical units

Do not automatically reduce everything to single-word lemmas.

When English is naturally learned as a unit, propose the complete unit.

Prefer:

`make a decision`

over only:

`decision`

when the collocation itself is what matters.

Prefer:

`run out of`

over:

`run`

Prefer:

`look forward to`

over:

`look`

Prefer the exact expression worked on in the session when that expression carries the useful meaning.

Avoid fragmenting idioms, phrasal verbs, collocations, or functional expressions into pieces that lose their learning value.

When the teacher explicitly taught both an individual word and a larger lexical unit containing it, preserve both only when they genuinely represent different learning targets.

## 7. Determine the relevant sense

A candidate must correspond to the meaning actually used or taught in the session.

Do not automatically add every dictionary sense of a word.

For each candidate:

- identify the contextual meaning;
- produce a concise natural gloss;
- prefer the meaning worked on in the session;
- preserve a teacher-provided distinction when relevant;
- add another sense only when it is directly relevant to understanding the class.

If the intended meaning is uncertain, do not guess.

Mark the candidate as uncertain or leave it unresolved until the source can be clarified.

## 8. Detect teacher corrections and learner difficulties

Corrections are high-value learning material.

When the session clearly contains a learner error followed by a correction, distinguish:

- what the learner said;
- the corrected form;
- what kind of issue it was;
- why the corrected item may be worth studying.

Treat the corrected reusable form as a high-priority candidate when the correction has learning value.

Do not normally store an incorrect form as the target vocabulary item unless the learning design explicitly requires an error/correction exercise.

If the mistake reveals a useful contrast, mention it when presenting the candidate to the user.

Likewise, language that repeatedly causes the learner difficulty should receive additional priority even when the teacher did not explicitly designate it as vocabulary.

Difficulty may involve:

- meaning;
- recall;
- word choice;
- collocation;
- pronunciation;
- spelling;
- register;
- confusion with another expression.

## 9. Consider pronunciation

Pronunciation can make an otherwise familiar word worth studying.

When pronunciation is relevant, consider:

- British IPA;
- syllable stress;
- difficult sounds;
- a correction made by the teacher;
- mismatch between spelling and pronunciation.

Use reliable British English IPA.

Do not guess IPA for a word whose identity or pronunciation is uncertain.

If pronunciation was not relevant and the word is straightforward, do not artificially turn every suggestion into a pronunciation lesson.

## 10. Prepare useful examples

When an example adds learning value, prepare a short natural English sentence.

Examples should:

- use the relevant sense;
- sound natural;
- be easy to understand;
- reflect British English where relevant;
- show the collocation, structure, or usage clearly;
- avoid unnecessary complexity.

Prefer a good example from the class when one exists.

If you create an example yourself, do not imply that the teacher said it.

Do not invent examples merely to fill a field.

## 11. Select candidates according to their origin

Apply different filtering behaviour depending on how the candidate entered the session.

### Deliberately taught material

For `explicit_vocabulary`, recover the complete identifiable set before applying ordinary shortlist logic.

Normally present all valid items.

Do not silently reduce an explicit vocabulary list from twenty items to six merely to make the response shorter.

If there are many items, organise or batch them rather than dropping them.

### Teacher-selected material

Items classified as `teacher_highlight`, `teacher_correction`, or a clear `learner_difficulty` should normally be retained unless they are duplicated, unreliable, or clearly have no reusable learning value.

### Incidental material

For `repeated_usage`, `inferred_useful`, and especially `incidental` language, filter aggressively.

Prioritise items that are:

- useful;
- frequent or reusable;
- appropriate for the learner's level;
- relevant to the session;
- useful in active production;
- difficult to remember;
- easy to confuse;
- pronunciation-sensitive;
- part of a useful lexical unit.

Deprioritise or omit incidental items that are:

- obvious basic vocabulary already used fluently;
- accidental transcript words;
- proper names with no learning value;
- administrative language;
- duplicates;
- transcription noise;
- highly specialised terms with little relevance;
- isolated words when the useful learning unit is a larger expression.

The purpose of filtering is to control incidental noise, not to override the teacher's intended vocabulary syllabus.

## 12. Check for duplicates and related items

Before proposing persistence, consider whether the same lexical item may already exist.

If LearnKit provides an appropriate lookup or consolidation mechanism, use it rather than creating duplicates.

Within the current session, merge repeated appearances of the same learning item while preserving useful source evidence.

Treat obvious inflectional variants as the same underlying item when appropriate.

Examples:

`decide`
`decided`
`deciding`

should not normally become three independent vocabulary entries.

However, do not collapse genuinely different expressions merely because they share a word.

For example:

`take off`

and:

`take responsibility`

are separate learning items.

Likewise, do not discard an explicitly taught collocation merely because its headword already exists independently in the vocabulary database.

Deduplication should prevent redundant learning items, not destroy meaningful lexical distinctions.

## 13. Present candidates to the user

Present candidates in a way that preserves the distinction between intended lesson content and additional agent suggestions.

When useful, group them approximately as:

- explicitly taught vocabulary;
- teacher-highlighted/corrected language;
- additional useful language detected by the agent.

Do not force these headings when the session is small, but preserve the distinction conceptually.

For explicitly taught vocabulary, completeness is more important than keeping the list short.

For incidental or inferred vocabulary, present a manageable shortlist rather than dumping every detected term.

For each candidate, include enough information to make a useful decision.

Normally show:

- English item;
- natural meaning in Spanish;
- type when useful (`phrasal verb`, `collocation`, etc.);
- short example when useful;
- British IPA when useful or relevant;
- source/location;
- why it is being proposed when that is not obvious.

Where useful, indicate whether the item was:

- explicitly taught;
- highlighted by the teacher;
- introduced through a correction;
- associated with learner difficulty;
- inferred by the agent.

Clearly mark uncertainty.

Do not disguise model inference as material from the class.

Most importantly:

**Do not apply the "manageable shortlist" rule in a way that hides or drops vocabulary that was clearly part of the lesson's intended learning content.**

## 14. Never persist suggestions automatically

This is a hard rule.

A candidate proposed by the agent is not a confirmed learning item.

Never execute:

`learnkit learn vocabulary add`

merely because you identified a good candidate.

This applies even when the vocabulary was explicitly taught in the lesson.

"Explicitly taught" determines candidate priority; it does not constitute user confirmation for persistence.

Persist only after the user explicitly:

- accepts an item;
- asks to add it;
- edits it and confirms the edited version;
- explicitly asks to add a defined set;
- explicitly asks to add all candidates from a clearly identified presented group.

Do not interpret silence as acceptance.

Do not automatically persist all suggestions merely because the user asked you to analyse or extract vocabulary from the session.

## 15. Persist confirmed candidates

For each explicitly confirmed item, use:

`learnkit learn vocabulary add`

with the richest reliable information supported by the CLI and the confirmed candidate.

Base command:

`learnkit learn vocabulary add --session <session_id> --lemma "<text>" --sense "<meaning>" --source <source_id> --locator "<locator>" --suggested-by agent`

When available and reliable, also include:

`--ipa "<British IPA>"`

and one or more:

`--example "<natural example>"`

Example:

`learnkit learn vocabulary add --session <session_id> --lemma "get away with" --sense "hacer algo malo sin ser castigado" --source <source_id> --locator "segment:00:02:30" --suggested-by agent --ipa "/ɡet əˈweɪ wɪð/" --example "He thought he could get away with it."`

Do not fabricate optional fields just to make the entry look complete.

Correct traceability is more important than field completeness.

## 16. User-supplied vocabulary

The user may ask to add an item that the agent did not suggest.

In that case:

- identify the correct session source when possible;
- verify the intended sense from the supplied context;
- use the same quality criteria;
- do not mark it as an agent suggestion unless it actually came from the agent.

Use the CLI's manual/default provenance behaviour according to the currently supported interface.

## 17. Handle uncertainty conservatively

Do not persist an item when its identity, meaning, source, or relevant usage cannot be established reliably.

Examples:

- inaudible transcript;
- uncertain spelling;
- unclear phrasal verb particle;
- ambiguous sense;
- possible transcription error.

Explain the uncertainty to the user.

Explicit teaching intent does not override source reliability.

If an item was clearly intended as vocabulary but cannot be reconstructed accurately, preserve that fact and report the unresolved item rather than inventing it.

It is better to leave one candidate unresolved than to create incorrect study material.

## 18. Keep extraction and Anki selection separate

A useful vocabulary entry and a useful Anki card are related but not identical decisions.

Do not reject explicitly taught vocabulary merely because it is difficult to illustrate.

Do not reject a useful lexical entry solely because Wikimedia is unlikely to find a suitable image.

Do not assume that every confirmed vocabulary entry must become an Anki card.

This skill is responsible for identifying and persisting language-learning items.

Card construction, images, audio, and export belong to the corresponding LearnKit workflows.

## 19. Dialogues

A dialogue is a distinct kind of learning item from vocabulary: a short spoken exchange worth keeping as a reusable unit rather than breaking apart into loose words.

Create a dialogue when the session worked a real communicative situation — introducing yourself, saying goodbye, asking for something, ordering, apologising — and that exchange is worth preserving as a whole.

Do not fragment a genuine dialogue into separate vocabulary entries merely because it also contains useful words; add those words to vocabulary too, but persist the exchange itself as a dialogue.

### `--origin class` vs `--origin added`

Every dialogue has an origin, and the two are not interchangeable:

- `--origin class` — the dialogue, or something very close to it, actually happened in the session. Reconstruct it from the transcript, notes, or other session material.
- `--origin added` — the agent (or the user) writes it afterwards to consolidate or link structures already seen in the session. It never happened verbatim in class.

Never mark a dialogue `class` when you authored it or reconstructed it loosely from disconnected fragments. This is the same hard rule that already governs this skill for vocabulary: never invent that the teacher said something they did not say. A `class` dialogue must be traceable to the session material the same way a vocabulary item's source must be.

An `added` dialogue must use only structures already confirmed in the session (or already part of the accumulated level — see below). Never introduce new grammar or vocabulary just to make an added dialogue read naturally; if the session hasn't covered it yet, leave it out or simplify the exchange instead.

### Persisting a dialogue

Never persist a dialogue without the user's explicit confirmation, exactly as with vocabulary. Presenting a reconstructed or proposed dialogue is not confirmation.

Once confirmed:

```
learnkit learn dialogue set --session <session_id> --id <dialogue_id> --origin class --line "Teacher: What's your name?" --line "Student: My name is Ana." [--note "..."]
```

- `--origin` is `class` or `added` — nothing else.
- `--line` is repeatable and required at least twice; each one is `"Speaker: text"`, split on the first literal `": "` — the speaker label must not itself contain `": "`.
- `--note` is optional, free text (e.g. what the dialogue is drilling).
- `learn dialogue set` upserts by `--id`: reuse the same `--id` to correct or extend an existing dialogue rather than creating a duplicate.

To remove one:

```
learnkit learn dialogue remove --session <session_id> --id <dialogue_id>
```

## 20. Pronunciation — minimal pairs

This is a different, complementary mechanism from the IPA guidance in section 9. Section 9 is about the pronunciation of a single word. A minimal pair is about an explicit contrast between two words.

Register a minimal pair when two words worked in the session — or one worked and another clearly confusable with it — differ by a real, pedagogically relevant phonetic contrast for a Spanish speaker: long vs short vowels, diphthongs vs pure vowels, endings like `-teen` vs `-ty`, and similar recurring sources of confusion.

Do not register random pairs with no relevance to the session. The contrast must be something the learner is actually likely to confuse.

Use British IPA for both members of the pair, consistent with the rest of this skill.

The note should explain, in one sentence, what to remember about the contrast — not repeat the transcription. For example, "long vs short /i/", not "/ʃiːts/ vs /ʃɪts/".

Never persist a minimal pair without the user's explicit confirmation.

```
learnkit learn pronunciation set --session <session_id> --id <pair_id> --word-a "sheets" --ipa-a "ʃiːts" --word-b "shits" --ipa-b "ʃɪts" --note "long vs short /i/"
```

`learn pronunciation set` upserts by `--id`. To remove one:

```
learnkit learn pronunciation remove --session <session_id> --id <pair_id>
```

## 21. Accumulated level

Before generating any new content for the session — an added dialogue, a new example, extra vocabulary — check the accumulated level:

```
learnkit learn level show
```

If no level has ever been saved, treat it as "unknown level." Do not assume it is low or high; base new content only on what the session itself demonstrates.

Never introduce structures far above the read level (or, with no saved level, far above what the session demonstrates) when creating new content — this applies directly to added dialogues (section 19) and to any example or extra vocabulary you generate.

At the end of the session, if the material worked plausibly suggests the level has advanced (e.g. A1 to A1+, or A1 to A2), you may propose an update to the user. Never run `learn level set` without the user's explicit confirmation first — same confirmation pattern this skill already uses for vocabulary.

```
learnkit learn level set --language en --variety en-GB --level A2 --notes "confident with present simple, still shaky on past tense questions"
```

`--notes` is optional.

## 22. Enriched vocabulary — `--topic` and `--notes`

`learn vocabulary add` and `learn vocabulary edit` also accept:

- `--topic "<text>"` — the pedagogical topic or unit this entry belongs to (e.g. "Meet & Greet", "Numbers", "Classroom"). Use it to group the session's vocabulary by concept so "what was learned today" can be reconstructed by topic instead of only chronologically.
- `--notes "<text>"` — observations that don't fit `--sense`/`--ipa`/`--example`: an irregular plural, a false friend, a word it's commonly confused with, formal/informal register, a usage nuance.

Both are optional and follow the same confirmation rule as every other field: propose, never persist without the user's explicit confirmation.

## 23. Grammar — use `analyse`'s `ConceptPage`, not a new entity

This skill does not introduce any new command for grammar. Grammar explanations continue to live in `analyse set --item page:<id>` (the generic `learnkit-analyse` skill) — `learnkit-language` only adds guidance on what a good language grammar page should contain. It does not duplicate or reimplement the `ConceptPage`/`filled_gaps` mechanism; for that mechanism, see the `learnkit-analyse` skill.

A good grammar page for a language session should have:

- the concept explained in one sentence;
- the structure (e.g. "I am... / You are...");
- between 3 and 6 examples, built with vocabulary already known from the session;
- one explicit, predictable "common mistake" for a Spanish speaker.

As with every other language-learning item in this skill, never persist a grammar page without the user's explicit confirmation.

## 24. Final quality check

Before presenting or persisting candidates, verify:

- deliberately taught vocabulary has been recovered as completely as the source permits;
- explicit vocabulary has not been removed by ordinary shortlist filtering;
- teacher-highlighted material has appropriate priority;
- teacher corrections and learner difficulties have been considered;
- incidental vocabulary has been filtered rather than indiscriminately collected;
- the lexical item is correctly identified;
- the meaning matches the session context;
- the learning unit has the right granularity;
- phrasal verbs and collocations have not been unnecessarily fragmented;
- British English is used consistently where applicable;
- IPA is British and matches the item when supplied;
- examples are natural and use the correct sense;
- teacher corrections are represented correctly;
- uncertainty is not hidden;
- source and locator are genuine;
- model-inferred usefulness is distinguished from explicit teaching intent;
- the item has actual learning value;
- any dialogue marked `--origin class` genuinely happened in the session, and any `--origin added` dialogue uses only already-confirmed structures;
- any minimal pair reflects a real, session-relevant phonetic contrast, with a note that explains the contrast rather than repeating the IPA;
- the accumulated level (`learn level show`) was checked before generating new content, and nothing generated sits far above it;
- nothing above was persisted without the user's explicit confirmation.

The objective is not to maximise or minimise the number of vocabulary entries.

The objective is to faithfully capture what the lesson deliberately taught, supplement it intelligently with genuinely useful language, and create a clean lexical knowledge base for future study.

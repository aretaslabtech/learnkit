---
name: learnkit-cards
description: Design, build, and review study cards from confirmed LearnKit learning items. Select the appropriate retrieval task, card structure, modality, and resources according to the learning objective. Subject-agnostic: cards may practise vocabulary, rules, concepts, structures, procedures, dialogue, code, diagrams, facts, or other knowledge.
---

# LearnKit cards — learning activity design and card quality

Use this skill when the user wants to create, build, inspect, improve, or prepare study cards from confirmed LearnKit learning material.

This skill is subject-agnostic.

A card is not defined by the subject being studied.

It is defined primarily by:

1. what the learner should learn;
2. what mental operation should be practised;
3. what stimulus should be presented;
4. what response should be retrieved;
5. what feedback should be shown.

The same LearningItem may support several useful cards.

Do not assume:

`one LearningItem = one Card`

and do not assume:

`Card = vocabulary flashcard`.

## 1. Start from the learning objective

Before selecting a card format, determine what the learner should be able to do.

Examples:

- recall a fact;
- recognise a concept;
- produce a word or structure;
- complete a pattern;
- transform one representation into another;
- classify something;
- enumerate members of a set;
- compare related concepts;
- order a sequence;
- explain a rule;
- identify an error;
- correct an error;
- interpret code, text, a map, a diagram, or audio;
- respond naturally to a scenario or dialogue.

Choose the card from the objective.

Do not choose the objective from an existing template merely because that template is available.

## 2. Separate knowledge from practice

A LearningItem represents knowledge worth learning.

A Card represents one way of practising that knowledge.

For example:

`present perfect`

may support cards for:

- recognition;
- production;
- transformation;
- comparison;
- error correction.

Similarly:

`for loop`

may support:

- definition recall;
- recognition;
- code interpretation;
- comparison with `while`;
- error detection;
- transformation.

Do not duplicate the underlying LearningItem merely to support multiple practice forms.

## 3. Core cognitive card types

Use the smallest useful set of generic card types.

### `recall`

Retrieve information from a cue.

Example:

`Capital of Portugal?`

### `recognition`

Identify the correct concept, interpretation, or alternative.

### `production`

Produce the target expression, answer, structure, or solution from meaning/context.

### `completion`

Fill missing information in text, code, a sequence, formula, dialogue, or other material.

### `transformation`

Convert one valid representation into another.

Examples:

- active → passive;
- sentence → target grammatical structure;
- `while` → equivalent `for`;
- fraction → percentage.

### `classification`

Assign something to the correct category.

### `enumeration`

Recall the members of a meaningful set.

Example:

`Which loop structures are available in C#?`

### `comparison`

Distinguish two or more related concepts.

### `sequence`

Recall or reconstruct an ordered process.

### `explanation`

Explain why or how something works.

### `error_detection`

Inspect material and identify one or more problems.

### `correction`

Produce a corrected version of faulty material.

### `scenario_response`

Produce an appropriate response to a realistic situation or dialogue.

### `interpretation`

Read or inspect source material and explain what it means or does.

### `identification`

Identify a target in an image, diagram, map, audio sample, text, code sample, or other stimulus.

These categories describe the learning operation, not the visual template.

## 4. Stimulus and card type are independent

A card may use:

- text;
- image;
- audio;
- code;
- map;
- diagram;
- formula;
- table;
- timeline;
- dialogue;
- mixed media.

Do not assume that a particular cognitive card type always requires a particular medium.

For example, `error_detection` may operate on:

- an English sentence;
- a paragraph;
- source code;
- a diagram;
- a mathematical solution;
- a map.

Likewise, `identification` may use:

- image;
- audio;
- diagram;
- text;
- code.

## 5. Response shape may vary

Not every card has one exact answer.

Determine the appropriate response form.

Possible response shapes include:

### `single`

One principal answer.

### `set`

Several answers where order does not matter.

Example:

`for`, `foreach`, `while`, `do...while`.

### `ordered_set`

Several answers where order matters.

### `alternatives`

Several different responses may all be acceptable.

Useful for dialogue or natural-language production.

### `free_response`

The learner should formulate an explanation or solution.

### `rubric`

The response should contain one or more expected ideas rather than an exact string.

Useful for:

- code review;
- explanation;
- analysis;
- open-ended error detection.

Do not force open learning tasks into exact-string answers.

## 6. Language-learning cards

Language profiles may use specialised combinations of the generic model.

Examples:

### Vocabulary production

Stimulus:

- image/context;
- audio when useful.

Response:

- target lexical item.

### Listening recognition

Stimulus:

- audio.

Response:

- recognised word/expression or meaning.

### Grammar rule recall

Stimulus:

- rule name, context, or example.

Response:

- condition, structure, or usage.

### Grammar transformation

Stimulus:

- source sentence.

Response:

- sentence using the target structure.

### Error correction

Stimulus:

`I used to playing football.`

Response:

`I used to play football.`

Feedback may explain why.

### Dialogue / functional language

Stimulus:

`Would you like some coffee?`

Response may contain several natural alternatives.

Do not require one artificial exact response when multiple authentic responses are valid.

### Structural production

Stimulus:

a communicative situation.

Response:

use the target structure naturally.

## 7. Programming cards

Programming profiles may use the same generic mechanisms.

Examples:

### Enumeration

`Which loop constructs are available in C#?`

Expected response:

- `for`;
- `foreach`;
- `while`;
- `do...while`.

### Interpretation

Show a code fragment.

Ask:

`What does this code do?`

### Prediction

Show executable logic.

Ask:

`What output does this produce?`

This may be represented as a specialised interpretation/recall task if no dedicated card type is required.

### Error detection

Show source code.

Ask:

`What problems can you identify?`

Expected response may use a rubric with several findings.

### Clean Code review

Show code.

Ask the learner to identify violations or improvements.

Do not require every possible observation.

Persist the expected core findings as feedback/rubric.

### Transformation

Ask the learner to refactor or rewrite code under a stated constraint.

## 8. Use atomic cards where appropriate

Prefer one clear retrieval objective per card.

Avoid cards that casually ask several unrelated questions.

However, do not apply atomicity mechanically.

Some learning objectives are inherently sets or integrated analyses.

For example:

`Which loop structures exist in C#?`

is legitimately one enumeration card.

Likewise:

`Find the Clean Code problems in this method`

may legitimately require several findings.

Atomicity means one coherent learning objective, not necessarily one word in the answer.

## 9. Avoid trivial cueing

Do not expose the target answer accidentally in the prompt.

Check:

- text;
- image labels;
- filenames or captions;
- audio;
- code comments;
- examples;
- multiple-choice wording;
- surrounding context.

The information that may be shown depends on what the learner is expected to retrieve.

Do not remove information that is legitimately part of the stimulus.

## 10. Use explanation as feedback

When useful, the reverse/feedback should not only state the answer.

It may explain:

- why;
- rule;
- contrast;
- common mistake;
- reasoning;
- relevant example;
- alternative accepted answers.

Keep feedback focused on the objective of the card.

A card is a retrieval activity first and a miniature lesson second.

## 11. Use multiple cards only when they train genuinely different retrieval

Do not create several cards that differ only cosmetically.

Generate additional cards when they exercise a different useful retrieval path.

For example:

`used to`

may justify:

- recognition;
- production;
- transformation;
- contrast with `be used to`.

It does not justify five paraphrased versions of the same definition question.

## 12. Select cards according to learning value

Do not automatically generate every possible card type for every LearningItem.

Select useful forms based on:

- subject/profile;
- nature of the knowledge;
- learner difficulty;
- teacher emphasis;
- desired skill;
- available evidence;
- existing cards;
- redundancy.

The objective is effective practice, not maximum card count.

## 13. Resources are requirements of the card, not universal requirements

A card may require:

- image;
- audio;
- text;
- code;
- diagram;
- no media.

The selected template/activity determines which resources are required.

Do not globally require images or audio.

Examples:

English vocabulary production may require both.

A C# enumeration card may require neither.

A phonetics card may require audio.

A geography identification card may require a map.

## 14. Delegate visual resolution

When a selected card requires an image and that image is missing or needs review, use `learnkit-image-prompts`.

Do not duplicate visual-generation or visual-QA logic in this skill.

`learnkit-cards` decides **why the image is needed and what role it has**.

`learnkit-image-prompts` decides **whether the actual visual fulfils that role**.

## 15. Build through LearnKit

Use the supported LearnKit card-building workflow.

`cards build` is not limited to vocabulary: a `LearningItem` created from a grammar page, dialogue, or pronunciation minimal pair via `learn item promote` (see `learnkit-language`) and given content via `cards set` builds into a real card the same way a vocabulary item does — `cards set`/`CardSpec` are already generic, this is a clarification of scope, not new behaviour.

Do not modify persisted card state manually.

After building, inspect the structured result and card validation state.

Treat CLI validation as structural validation.

Do not assume that `complete` means pedagogically correct.

## 16. Perform whole-card quality review

A complete card must be coherent as a learning activity.

Check:

- the target matches the LearningItem;
- the question/stimulus matches the intended learning objective;
- the expected response actually answers that prompt;
- the response shape is appropriate;
- explanation/feedback is correct;
- examples correspond to the same concept or sense;
- image corresponds to the intended concept;
- audio corresponds to the intended target;
- pronunciation/notation is internally consistent when applicable;
- the answer is not accidentally exposed;
- the card is neither trivial nor unnecessarily overloaded;
- duplicated cards do not train the same retrieval without a reason.

A structurally complete card may still fail this review.

## 17. Preserve uncertainty

Do not create a definitive card from uncertain learning material.

If the source, answer, interpretation, rule, or expected response is uncertain:

- preserve the uncertainty;
- leave the card/item unresolved when necessary;
- do not manufacture a clean-looking answer.

## 18. Keep export separate

This skill designs, builds, and validates study cards.

Anki is an export target.

Do not make Anki-specific packaging rules the conceptual model for all cards.

When the user asks for Anki export, use the appropriate LearnKit export workflow after card readiness has been checked.

## Final principle

A useful card is not:

> some content placed on a front and a back.

It is:

> a deliberately designed retrieval activity for a specific learning objective.

The preferred reasoning is:

**LearningItem → learning objective → cognitive task → stimulus → expected response → feedback → required media → build → whole-card QA**

not:

**LearningItem → default template → card**

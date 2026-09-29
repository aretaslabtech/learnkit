---
name: learnkit-image-prompts
description: Resolve and review learning images for LearnKit cards. Prefer suitable existing imagery, generate unresolved concepts efficiently in image grids, visually inspect every candidate, and assign or reject images according to the intended learning objective. Subject-agnostic and provider-independent.
---

# LearnKit image prompts — visual resolution and quality review

Use this skill when a LearnKit session has learning items/cards that still need an image (`pending_image`), or when images automatically resolved from Wikimedia Commons need semantic and pedagogical review.

This skill is deliberately subject-agnostic.

It may be used for:

- language learning;
- geography;
- history;
- mathematics;
- science;
- phonetics;
- grammar;
- programming;
- or any other LearnKit profile.

Its job is not merely to find or generate a picture.

Its job is to ensure that the final visual actually serves the intended learning objective.

---

## Core principle

The preferred image-resolution strategy is:

```text
existing/supplied image
        ↓
Wikimedia Commons candidate
        ↓
actual visual review
    ├── suitable → keep
    └── unsuitable → reject
                          ↓
                   generated image
                          ↓
                        crop
                          ↓
                 actual visual review
                    ├── suitable → assign
                    └── unsuitable → leave pending
```

Reuse an existing suitable image before generating a new one.

Generate only what remains unresolved.

---

## The guard that matters most

**LearnKit's CLI does not judge whether an image is semantically or pedagogically appropriate.**

Commands such as:

- `cards image-batch`;
- `cards image-grid crop`;
- `cards image-grid assign`;
- `cards image-review`;
- `cards image-reject`;

are mechanical.

They list items, crop files, assign assets, or persist rejection decisions.

**Never accept or assign an image that you have not actually opened and visually inspected.**

Do not infer image quality from:

- filenames;
- Wikimedia titles;
- metadata;
- search-result descriptions;
- concept titles;
- the fact that LearnKit downloaded the image.

If you have not looked at the actual image, you do not yet have a judgement about it.

---

# Part A — Understand the visual objective

## 1. Establish what the image is supposed to teach

Before generating or judging an image, determine what the learner is expected to:

- recognise;
- recall;
- locate;
- distinguish;
- understand;
- compare;
- interpret;
- or infer.

Do not reason only from the item title when richer context is available.

Use all reliable context available from the learning item, session, card, template, examples, or profile.

Useful internal questions include:

```text
What is the target concept?
What specific meaning or sense is intended?
What is the learning objective?
What must the learner retrieve?
What information may be visible?
What information would reveal the answer?
What visual form best represents the concept?
```

This is reasoning guidance, not a required persisted schema.

---

## 2. Do not guess ambiguous meanings

A concept title may be ambiguous.

Examples:

```text
bank
charge
stress
cell
current
pitch
mean
```

If a stored sense, example, explanation, or session context establishes the intended meaning, use it.

For example:

```text
bank
sense: the land alongside a river
```

must be treated as a riverbank concept, not a financial institution.

If the intended meaning cannot be established reliably:

- do not guess;
- do not generate an arbitrary image;
- leave the item unresolved;
- report that semantic context is insufficient.

A visually attractive image of the wrong meaning is still incorrect.

---

## 3. Choose the appropriate visual form

Do not force every concept into a photograph or single-object illustration.

Depending on the learning objective, the best representation may be:

- a photograph;
- a simple object;
- a real-world scene;
- an action;
- a spatial relationship;
- a map;
- a diagram;
- a timeline;
- a process;
- a comparison;
- a labelled schematic;
- a chart;
- a sequence;
- an interface-like diagram;
- a symbolic or technical representation.

Prefer the simplest visual that communicates the intended concept clearly.

The goal is educational usefulness, not decorative complexity.

---

## 4. Text is context-dependent

Do not apply a universal "no text" rule.

Text, symbols, labels, dates, coordinates, IPA, formulas, annotations, place names, point names, captions, or code may be appropriate or necessary.

Examples:

### Language vocabulary

The target lexical item may need to remain hidden if the learner is expected to recall it from the image.

### Geography

A map may legitimately include labels, borders, coordinates, or neighbouring place names.

### Phonetics

IPA symbols and anatomical labels may be necessary.

### History

A timeline may require dates and event labels.

### Mathematics

A geometry diagram may require measurements, variables, or named points.

### Programming

A visual may require node names, identifiers, or short code fragments.

The governing rule is:

> Include textual or symbolic information when it supports the learning objective. Avoid only information that would unintentionally reveal what the learner is expected to retrieve.

Text is not automatically answer leakage.

---

# Part B — Review Wikimedia images

## 5. Prefer suitable Wikimedia imagery before generation

Existing licensed imagery should be reused when it serves the learning objective well.

Run:

```bash
learnkit cards image-review --session <session_id> --json
```

This lists cards/items whose image was resolved from Wikimedia Commons.

For each entry, use the actual image file at its returned path.

Do not judge it from metadata alone.

---

## 6. Inspect every Wikimedia image

Open and visually inspect each image.

Evaluate:

### Semantic correctness

Does it represent the intended concept or sense?

### Learning-objective fit

Does it support what the learner is supposed to understand or retrieve?

### Clarity

Will it remain understandable at study-card size?

### Ambiguity

Could it strongly suggest another concept or meaning?

### Text and notation

Are visible labels, symbols, dates, words, or annotations appropriate?

### Answer leakage

Does it reveal something the learner should be retrieving?

### Educational accuracy

For maps, diagrams, science, history, mathematics, phonetics, technical subjects, etc., is the information actually correct?

---

## 7. Keep suitable Wikimedia images

If the image passes review, leave it unchanged.

Do not regenerate it merely for aesthetic consistency.

Pedagogical usefulness matters more than making every card use the same visual style.

---

## 8. Reject unsuitable Wikimedia images

If the image fails review:

```bash
learnkit cards image-reject \
  --session <session_id> \
  --item <learning_item_id> \
  --reason "<specific reason>" \
  --json
```

Use a concrete reason.

Good examples:

```text
represents the financial meaning of "bank", but this item means riverbank
```

```text
target country is labelled directly and reveals what the learner is meant to identify
```

```text
diagram shows the wrong anatomical position
```

```text
image is related to the topic but too ambiguous to work as a retrieval cue
```

```text
timeline contains an incorrect event date
```

Avoid vague reasons such as:

```text
bad image
```

or:

```text
doesn't fit
```

The rejection reason should help future attempts.

After rejection, the item is offered again by `cards image-batch` — this holds
even when the template's front image is `Optional` rather than `Required`
(e.g. `expression-production-v1`). `image-reject` always clears the front
image's asset regardless of policy, and `image-batch` re-lists any card whose
template wants a front image (`Required` or `Optional`) and doesn't have one
assigned — it does **not** rely on the card's overall completeness, since an
`Optional` slot missing its asset never makes the card "incomplete" on its
own. A rejected image must never be left unreplaced just because its slot was
optional.

---

# Part C — Generate unresolved images efficiently

## 9. Get unresolved items

Run:

```bash
learnkit cards image-batch --session <session_id> --json
```

This returns every card whose template wants a front image and has none
assigned yet — whether that image is `Required` (a true `pending_image` card)
or merely `Optional` and was never resolved or was rejected. Don't assume
every item here is "incomplete" in `cards validate`'s sense; some are
complete-but-imageless `Optional` slots that still deserve an image.

Preserve:

- `learning_item_id`;
- `card_id`;
- title/concept;
- semantic/contextual information available for that item;
- returned order.

If the batch is empty, there is nothing to generate.

---

## 10. Batch generation is an optimisation

The purpose of image grids is to reduce the number of generation operations.

The efficient pattern is:

```text
N concepts
    ↓
1 generated composite image
    ↓
crop
    ↓
N individual images
```

Do **not** generate N independent images and then combine them into a grid.

That defeats the main purpose of batching.

---

## 11. Use the densest useful grid

Prefer the largest grid that preserves sufficient visual quality.

General guidance:

```text
simple visual concepts
→ 4×4
→ up to 16 concepts

medium-complexity visuals
→ 3×3
→ up to 9 concepts

maps / diagrams / labelled or detailed material
→ 2×2
→ up to 4 concepts

exceptionally detailed material
→ individual generation
```

This is a heuristic, not a rigid contract.

Use:

> the densest grid that still produces useful educational images.

Do not use 4×4 when doing so would make maps, diagrams, labels, technical detail, or visual distinctions too small or unreliable.

---

## 12. Group visually compatible items when useful

Do not fill a grid mechanically.

Whenever practical, group items that have similar visual requirements.

A 4×4 grid is well suited to:

- simple objects;
- actions;
- everyday scenes;
- visually straightforward concepts.

A single grid may be less suitable if it mixes:

- a detailed European map;
- a phonetics articulation diagram;
- a historical timeline;
- a cell diagram;
- a simple photograph.

Split incompatible visual requirements into separate batches when doing so materially improves generation quality.

Optimise generation count without sacrificing educational usefulness.

---

## 13. Write the grid prompt from meaning, not titles

The prompt must describe what each cell should visually show.

Do not merely list lexical or concept names.

Bad:

```text
1. bank
2. commute
3. stress
```

Better:

```text
1. A grassy riverbank beside flowing water, clearly showing the edge of the river.

2. A person making their regular morning journey from home to work by train.

3. A side-view educational mouth diagram showing the tongue position required for the target sound.
```

Use the intended sense and learning objective.

---

## 14. Keep grid mapping deterministic

Use strict reading order:

```text
row 1: left → right
row 2: left → right
row 3: left → right
...
```

Concept 1 corresponds to crop 1.

Concept 2 corresponds to crop 2.

And so on.

Never try to rematch concepts after generation when positional order already defines the mapping.

---

## 15. Unused cells

If the chosen grid has more cells than real concepts:

- leave unused cells blank or visually neutral;
- do not insert decorative concepts;
- do not add extra objects that might be confused with real learning items;
- do not assign unused crops.

Example:

```text
13 concepts in a 4×4 grid
→ 13 real cells
→ 3 neutral/blank cells
```

---

## 16. Text policy may differ between cells

Do not necessarily apply one text rule to the entire grid.

Examples:

```text
Cell 2: no written target word.

Cell 5: surrounding country labels allowed, target country unlabelled.

Cell 8: anatomical labels allowed.

Cell 10: dates required.

Cell 13: formula notation required.
```

The image-generation prompt should state these requirements explicitly when relevant.

---

## 17. Generate the composite image

This skill must remain independent of any specific image-generation provider.

If the current environment has an image-generation capability that can produce a usable file, it may be used.

Otherwise:

- provide the completed prompt to the user;
- ask them to run it using their chosen image generator;
- continue once the resulting grid image is available.

Do not pretend generation has occurred if only a prompt exists.

---

## 18. Crop using the actual geometry

Run:

```bash
learnkit cards image-grid crop \
  --file <grid_path> \
  --rows <rows> \
  --cols <cols> \
  --out-dir <dir> \
  --json
```

The crop geometry must exactly match the generated grid.

Examples:

```text
4×4 → --rows 4 --cols 4
3×3 → --rows 3 --cols 3
2×2 → --rows 2 --cols 2
```

Never request one geometry and crop using another.

---

# Part D — Review generated crops

## 19. Inspect every crop individually

Open each relevant crop.

Do not judge only the full composite grid.

Match each crop to the corresponding item using reading order.

For each crop, check:

- correct concept;
- correct sense;
- learning-objective fit;
- clarity;
- ambiguity;
- text/notation policy;
- answer leakage;
- educational accuracy;
- clean crop boundaries;
- sufficient visual quality at study size.

A crop can fail even when the overall grid looks good.

---

## 20. Assign only crops that pass

For a valid crop:

```bash
learnkit cards image-grid assign \
  --session <session_id> \
  --item <learning_item_id> \
  --file <crop_path> \
  --json
```

For an invalid crop:

- do not assign it;
- leave the item `pending_image`.

Do not lower the quality threshold simply because the rest of the grid succeeded.

Each item is judged independently.

---

## 21. Partial success is normal

A grid does not have to be entirely successful.

Example:

```text
16 generated cells

12 valid → assign
4 unsuitable → leave pending
```

This is a successful batch.

Do not discard good images merely because some cells failed.

---

## 22. Retry unresolved items intelligently

Items that remain `pending_image` may be:

- included in another grid;
- grouped differently;
- generated at lower grid density;
- given a more specific prompt;
- generated individually;
- supplied manually;
- resolved through another suitable source.

If a 4×4 crop failed because it lacked enough detail, retrying the same concept in another identical 4×4 without changing the prompt or resolution strategy is unlikely to help.

Adapt the next attempt.

---

# Part E — Reporting

## 23. Report Wikimedia review

Tell the user:

- which images were reviewed;
- which were kept;
- which were rejected;
- why rejected images failed;
- which items returned to `pending_image`.

---

## 24. Report generated-image results

Tell the user:

- grid dimensions used;
- which concepts received valid images;
- which crops were rejected or left unassigned;
- why they failed;
- which concepts still require another attempt.

Do not hide partial failures.

---

# Final principle

The image is not the learning objective.

It is a tool used to support the learning objective.

Therefore, never ask only:

> Does this image relate to the concept title?

Ask:

> Does this visual accurately and clearly support what the learner is supposed to understand, recognise, distinguish, locate, interpret, or retrieve?

The preferred workflow is:

```text
understand objective
→ reuse suitable existing imagery
→ validate visually
→ generate only unresolved items
→ batch efficiently
→ crop
→ inspect each result
→ assign only suitable images
```

Optimise generation cost, but never at the expense of sufficient educational quality.

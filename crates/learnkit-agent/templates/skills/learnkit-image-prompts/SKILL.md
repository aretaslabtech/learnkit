---
name: learnkit-image-prompts
description: Write 4x4 grid image-generation prompts for concepts pending an image, judge each cropped result (and each already-resolved Wikimedia image) for coherence, and assign or reject accordingly. Subject-agnostic — no language/domain-specific instructions.
---

# LearnKit image prompts — grid generation and coherence review

Use this skill when a session has cards missing a front image (`pending_image`),
or when you want to double-check images LearnKit already resolved
automatically from Wikimedia Commons.

This skill is deliberately generic: it never assumes a language-learning
session, and it never calls any image-generation API itself — LearnKit has no
direct integration with one. All it does is get a person to run a prompt in
an external tool (ChatGPT or similar) and hand back the result.

## The guard that matters most

**LearnKit's CLI never judges whether an image represents its concept.**
Every command in this skill (`image-batch`, `image-grid crop`, `image-grid
assign`, `image-review`, `image-reject`) is purely mechanical: it lists what's
missing, crops pixels, or persists a decision you already made. **You must
never assign or accept an image you have not actually looked at and judged
yourself.** If you have not opened and viewed a specific image file, you do
not yet have an opinion about it — do not assign it "to make progress".

## Part A — Generating images for concepts with no image yet

### 1. Get the batch of concepts needing an image

Run:

```
learnkit cards image-batch --session <session_id> --json
```

This returns up to 16 cards in `pending_image` state, each with a
`learning_item_id`, `card_id`, and `title` (the concept). The command already
caps the batch at 16 (a single 4x4 grid's worth) via `--limit` (default 16).
If the session has more than 16 pending, run it again after this batch is
resolved — you will handle multiple batches one at a time, one grid prompt
per batch.

If the batch is empty, there is nothing to do for this part.

### 2. Write one grid-generation prompt per batch

For the batch you just got, write a single text prompt to hand to an
external image-generation tool (ChatGPT or similar — you never call this API
yourself). The prompt must ask for:

- A single image laid out as a 4x4 grid (16 cells).
- One cell per concept in the batch, in **exactly the same reading order**
  the items came back from `image-batch` (row 1 left-to-right, then row 2,
  and so on — the first item in the batch is the top-left cell, the last is
  the bottom-right cell).
- Each cell: a clear, single-subject illustration of that one concept,
  suitable for a flashcard. Be concrete about what each cell should show —
  don't just list bare words, briefly describe the concept so the result is
  unambiguous (e.g. not just "bank", but "a riverbank with grass and water",
  or "a financial bank building with columns", whichever the concept
  actually means in this session).
- **No text or labels baked into the image itself** — the word is already
  printed on the flashcard, a label in the image would be redundant and can
  clash with the card layout.

If the batch has fewer than 16 items, ask for a grid sized to what you have
(you may still request 4x4 with a couple of cells intentionally left simple/
blank, or a smaller grid — whichever produces a cleaner result; either is
fine as long as the reading order for the real concepts is unambiguous).

### 3. Hand the prompt to the person

Tell the person, in the conversation, the prompt you wrote and ask them to:

1. Run it in ChatGPT (or their preferred image-generation tool).
2. Save the resulting grid image locally.
3. Tell you the file path once it's saved.

Wait for their reply before continuing — you cannot proceed without the
actual image file.

### 4. Crop the grid

Once you have the file path, run:

```
learnkit cards image-grid crop --file <path> --rows 4 --cols 4 --out-dir <dir> --json
```

This returns the list of 16 individual crop file paths, in the same reading
order as the grid (and therefore the same order as the batch from step 1).

### 5. Look at each crop and judge it

**Actually view each cropped image file** — you have vision, use it. For
each crop, match its position in the reading order to the concept at the
same position in the batch from step 1, and judge: does this image
coherently represent that concept?

- **Coherent**: assign it.

  ```
  learnkit cards image-grid assign --session <session_id> --item <learning_item_id> --file <crop_path> --json
  ```

- **Not coherent**: do **not** assign it. Do not force a mismatched image
  onto a card just to make the batch "done".

### 6. Report back

Tell the person, in the conversation:

- Which concepts got an image assigned.
- Which concepts did **not** get a good image, and that they still need a
  fresh attempt — either another grid batch/prompt, or a manually supplied
  image. Their cards stay `pending_image` until then.

## Part B — Reviewing images LearnKit already resolved from Wikimedia

`cards build` resolves front images automatically from Wikimedia Commons
without checking whether the result actually matches the concept — it only
checks the license. This part lets you catch a mismatch after the fact.

### 1. List what's there

```
learnkit cards image-review --session <session_id> --json
```

This lists every card whose front image came from Wikimedia (not a supplied
or grid-generated image), each with its `card_id`, `learning_item_id`,
`title`, `asset_id`, `path` (the actual image file — open this), and license/
attribution info.

### 2. Look at each one and judge it

Same principle as Part A step 5: **open and look at the image file at
`path`** for each entry, and judge whether it truly represents `title`.

- **Coherent**: nothing to do — leave it as is.
- **Not coherent**: reject it, explaining why.

  ```
  learnkit cards image-reject --session <session_id> --item <learning_item_id> --reason "<why it doesn't fit>" --json
  ```

  This clears the card's front image (it reverts to `pending_image`) and
  durably records the rejected asset and your reason, so the same bad
  candidate isn't silently proposed again without context. The card is now a
  good candidate for Part A (grid generation), a fresh `cards build` attempt,
  or a manually supplied image.

### 3. Report back

Tell the person which images you reviewed, which ones you rejected and why,
and which cards are now `pending_image` again as a result.

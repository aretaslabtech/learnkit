<!--
Sync Impact Report
- Version change: N/A (template placeholders) → 1.0.0
- Bump rationale: Initial concrete ratification of the constitution. All template
  placeholders replaced with LearnKit's actual non-negotiable principles, derived
  verbatim from learnkit-implementation-spec/README.md ("Principios no negociables")
  and cross-referenced against docs/02-architecture.md and docs/13-decisions.md
  (ADR-001 through ADR-013). Treated as MAJOR-equivalent initial adoption per
  governance rules (first ratified version is 1.0.0).
- Modified principles: none renamed (first fill from template slots).
- Added sections: all 12 Core Principles (I–XII), "Architecture & Technology
  Constraints", "Development Workflow & Quality Gates", "Governance".
- Removed sections: none.
- Follow-up TODOs:
  - TODO(RATIFICATION_DATE): the original date these principles were first agreed
    upon (README.md predates this constitution file) is not recorded anywhere in
    the spec; confirm with the project owner and replace this TODO.
-->

# LearnKit Constitution

## Core Principles

### I. Filesystem-First
Project files (YAML/JSON/Markdown/assets) are the source of truth. Any derived
index (e.g. SQLite) MUST be reconstructible from the filesystem and MUST NOT be
treated as authoritative.
**Rationale**: Filesystem-first artifacts survive tool churn, are diffable, and
remain auditable independent of LearnKit's own runtime (ADR-002).

### II. Git-Friendly
Canonical artifacts MUST be representable in formats that version and review well
under Git (text-based YAML/JSON/Markdown over opaque binaries wherever a choice
exists).
**Rationale**: Enables diff review, history, and collaboration without proprietary
tooling.

### III. Rust Owns Orchestration
The CLI, workflow engine, guards, validation, domain models, and exporters MUST
live in Rust. Non-Rust code (e.g. Python) MAY only run as an isolated worker
behind a typed provider interface producing typed drafts/results; it MUST NOT
mutate LearnKit's internal state directly or mark workflow phases as valid.
**Rationale**: A single orchestration language keeps guard enforcement,
concurrency, and distribution simple and avoids scattered scripts silently
mutating state (ADR-001, docs/02-architecture.md §5).

### IV. Hard Guards (NON-NEGOTIABLE)
No workflow phase is considered valid because an agent (Codex, Claude, or any
other) declares it so. Validators MUST re-run before any dependent phase
transition is permitted, and downstream phases MUST be invalidated when their
dependencies change.
**Rationale**: Agent prompts and Skills can be skipped, ignored, or hallucinated;
the binary is the only reliable enforcement point (ADR-003).

### V. Agent-Agnostic
Codex, Claude, and any other coding agent MUST consume the same workflow, the
same CLI contract, and the same artifacts. Agent-specific integration is limited
to thin Skills/instructions layered on top of the shared Rust core.
**Rationale**: Prevents lock-in to a single agent vendor and keeps enforcement
logic out of prompts (ADR-010).

### VI. Domain Profiles Extend, Never Contaminate
Domain-specific rules (languages, geography, history, Godot, etc.) MUST be added
via profiles/capabilities layered on top of the core, and MUST NOT introduce
domain-specific concepts (e.g. linguistic semantics) into core entities.
**Rationale**: Keeps the core (e.g. "Learning Item") generic enough to serve
non-linguistic subjects without special-casing (ADR-004, ADR-005).

### VII. Anki Is a Target, Not the Database
LearnKit MUST maintain its own canonical entities (e.g. Card Definition) and
treat Anki (`.apkg`) purely as an export target.
**Rationale**: Avoids lock-in and allows the same card model to feed other
destinations (ADR-006).

### VIII. Multimedia Is First-Class
Images, audio, and (in the future) video MUST be modeled as independent,
reusable, deduplicated assets referenced from cards/assessments — never embedded
as opaque blobs owned by a single feature.
**Rationale**: Enables reuse, deduplication, validation, and features like
independent front/back audio (ADR-007).

### IX. Assessment Is Distinct From Memorisation
Cards (Anki-style memorisation) and Assessments (exams/games) both derive from
Learning Items but MUST NOT derive from each other.
**Rationale**: Memorisation and evaluation measure different things and must
evolve independently (ADR-008).

### X. No Platform Lock-In
Kahoot, Blooket, Anki, Drive, and any future destination MUST be implemented as
substitutable exporters/adapters around LearnKit's own Question Bank and domain
model — never as the system of record.
**Rationale**: Preserves portability and avoids dependence on private platform
APIs (ADR-009).

### XI. Assessment Interoperability From Day One
The V1 domain model MUST be QTI-friendly (capable of a clean future QTI mapping)
even though QTI/TAO integration itself is deferred to V2. LearnKit's IDs and
question bank remain the source of truth; TAO/QTI are never authoritative over
questions.
**Rationale**: Avoids a costly V2 domain migration; V2 will adopt QTI 2.2 as
baseline with a path to QTI 3 (ADR-012, ADR-013, docs/02-architecture.md §7).

### XII. Transcription Is a Provider
LearnKit MUST define its own transcription contract (`TranscriptionProvider`)
independent of any specific engine. `whisper.cpp`, `faster-whisper`, Buzz
imports, or manual transcripts are interchangeable providers behind that
contract. Providers only produce a `TranscriptDraft`; they MUST NOT modify
workflow state or mark phases valid.
**Rationale**: Decouples domain knowledge from a specific ASR engine and keeps
Hard Guards (Principle IV) as the sole validator of state transitions (ADR-011).

## Architecture & Technology Constraints

- **Workspace shape**: Rust workspace with crates such as `learnkit-cli`,
  `learnkit-core`, `learnkit-store`, `learnkit-workflow`, `learnkit-profile`,
  `learnkit-cards`, `learnkit-media`, `learnkit-transcription`, `learnkit-anki`,
  `learnkit-assessment`, `learnkit-export`, `learnkit-agent`. V1 MAY start with a
  reduced subset (4-5 crates) and split further only when real boundaries emerge;
  crate boundaries are not fixed in this constitution and are refined in
  `docs/02-architecture.md`.
- **`learnkit-cli` contains no domain logic**: parsing, human/JSON output, and
  exit codes only.
- **Derived index is disposable**: SQLite (or equivalent) indexes MUST be
  reconstructible from the filesystem store and MUST NOT be the last remaining
  copy of any canonical fact.
- **Python usage is restricted**: permitted only as an isolated worker behind a
  typed interface (`Rust CLI -> typed provider interface -> external worker ->
  typed result`), never as scripts that reach into LearnKit's internal state
  directly.
- **Plugin model**: V1 uses internal Rust traits (e.g. `AudioProvider`,
  `TranscriptionProvider`) rather than a dynamic plugin/ABI system. Dynamic
  plugin loading is explicitly out of scope until a real need is demonstrated.
- **Licensing**: no AGPL (or similarly restrictive) dependency may be linked
  into the LearnKit binary without an explicit, documented license review;
  interoperate with such systems (e.g. TAO Community Edition) via external
  processes/formats instead.
- **TAO/QTI boundary (V2)**: TAO is never the source of truth for questions;
  LearnKit IDs MUST survive round-trips via metadata/fingerprints; the QTI
  exporter MUST be deterministic and validable; result import MUST produce
  `Attempt` events rather than writing progress metrics directly.

## Development Workflow & Quality Gates

- **Architecture Decision Records**: significant, hard-to-reverse design choices
  MUST be recorded as ADRs in `learnkit-implementation-spec/docs/13-decisions.md`
  (or successor) before or alongside implementation. Existing ADR-001 through
  ADR-013 are binding until explicitly superseded by a new ADR.
- **Guard revalidation is mandatory**: any change that touches workflow phases,
  validators, or exporters MUST be reviewed against Principle IV (Hard Guards) —
  a phase transition that trusts agent/user assertion instead of re-running
  validation is a defect, not a shortcut.
- **Cross-domain regression check**: since the core must serve both linguistic
  and non-linguistic domains (Principle VI), a change that special-cases one
  domain profile inside `learnkit-core` MUST be rejected or refactored into a
  profile before merge.
- **Deferred decisions stay deferred**: items explicitly listed as "diferidas" in
  `docs/13-decisions.md` (e.g. exact XLSX crate, TTS provider, ID format) MUST
  NOT be treated as settled architecture until a corresponding ADR is added.

## Governance

- **Supremacy**: This constitution supersedes conflicting guidance in other
  LearnKit documents (specs, plans, task lists, Skills). Where a document under
  `learnkit-implementation-spec/docs/` conflicts with a Core Principle here, the
  principle wins until either the constitution or the ADR is amended.
- **Amendment procedure**: amendments are proposed by editing this file via the
  `/speckit-constitution` workflow (or direct PR review for this repository),
  and MUST include an updated Sync Impact Report describing what changed and
  why. Adding or materially changing a Core Principle requires the same review
  rigor as an ADR (see Development Workflow above) and SHOULD reference the
  ADR it codifies or supersedes.
- **Versioning policy**: semantic versioning (MAJOR.MINOR.PATCH).
  - MAJOR: removal or backward-incompatible redefinition of a Core Principle.
  - MINOR: a new principle or section is added, or existing guidance is
    materially expanded.
  - PATCH: wording, typo, or non-semantic clarification.
- **Compliance review**: any PR/spec/plan generated through the Spec Kit
  workflow (`/speckit-specify`, `/speckit-plan`, `/speckit-tasks`,
  `/speckit-implement`) MUST be checked against the Core Principles above,
  especially Hard Guards (IV) and Agent-Agnostic (V), before being merged or
  executed. Complexity that appears to violate a principle MUST be explicitly
  justified in the plan or rejected.
- **Guidance file**: for day-to-day implementation conventions not rising to
  the level of a principle, use `learnkit-implementation-spec/docs/` and
  `.specify/memory/` as the working guidance set; this constitution governs
  when they conflict.

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): original agreement date not recorded in source spec | **Last Amended**: 2026-09-26

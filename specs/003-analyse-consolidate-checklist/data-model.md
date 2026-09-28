# Data Model: Checklist por fase y fases `analyse`/`consolidate` reales

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Research**: [research.md](./research.md)

Entidades derivadas de `spec.md` → Key Entities. `ChecklistItem` extiende el `PhaseManifest` ya existente (`crates/learnkit-workflow/src/engine.rs`, ver `research.md` §1); el resto son entidades nuevas dentro de `learnkit-workflow` (genéricas, sin conceptos de idiomas — Principio VI).

## PhaseManifest (existente, ampliado)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `phase` | string | Igual que hoy. | Sin cambios. |
| `input_fingerprint` / `output_fingerprint` | string | Igual que hoy. | Sin cambios; para una fase con checklist, es el fingerprint agregado de todos los elementos. |
| `validated_at` | string | Igual que hoy. | Sin cambios. |
| `result` | `PhaseResult` (`Valid` \| `Failed`) | Igual que hoy. | Para una fase con checklist, se deriva del agregado de sus elementos (ver `ChecklistItem.state` más abajo), nunca se fija manualmente. |
| `checklist` | `Option<Vec<ChecklistItemManifest>>` | **Nuevo.** `None` para fases sin checklist (`inventory`, `vocabulary`, `cards`, `anki`, `assessment`); `Some(items)` para `analyse`. | `#[serde(default)]` — manifests antiguos sin este campo se leen como `None`, compatible hacia atrás. |

## ChecklistItemManifest (nuevo)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `item_id` | string | Identificador estable del elemento dentro de la fase (`summary`, `mindmap`, `page-<n>`). | Único dentro del `checklist` de una fase; para `page-<n>`, `<n>` es secuencial por sesión. |
| `state` | enum | `Pending` \| `Done` \| `Blocked` \| `PendingUserDecision`. | `Blocked` si la fase de la que depende (`inventory`) no es `Valid`; `PendingUserDecision` cuando se marcó explícitamente con `flag-pending` (FR-007) y no se ha resuelto. |
| `input_fingerprint` | string | Huella de las fuentes concretas de las que depende este elemento (puede ser un subconjunto de las fuentes de la sesión). | Si cambia, el elemento pasa a `Pending` aunque estuviera `Done` (invalidación en cascada, FR-004). |
| `pending_reason` | string \| null | Motivo explicado cuando `state = PendingUserDecision`. | Obligatorio no vacío cuando el estado lo es; `null` en cualquier otro estado. |
| `resolution` | `{kind: "confirmed"} \| {kind: "skipped", reason} \| null` | Cómo se resolvió un elemento que pasó por `PendingUserDecision`. | `skipped` conserva el motivo dado en `analyse <item> skip --reason ...` (FR-008), auditable igual que el override de `docs/04-workflow-guards.md §10`. |

## ClassSummary (Resumen)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `session_id` | string | Sesión a la que pertenece. | — |
| `content` | markdown | Resumen redactado (qué se trabajó, conceptos importantes, reglas, ejemplos a conservar, errores a repasar, vocabulario a estudiar — spec §Guia_maestra §6). | No vacío para considerarse `Done` (FR-005). |
| `filled_gaps` | list<{concept, note}> | Huecos del material de origen que el agente completó (FR-006). | Puede estar vacía; cada entrada queda marcada como añadida por el sistema, nunca se mezcla sin distinción con el contenido derivado directamente del material original. |
| `source_fingerprint` | string | Huella de las fuentes (notas + transcripción activa) usadas para redactarlo. | Base del `input_fingerprint` del `ChecklistItemManifest` `summary`. |

## StudyMap (Mapa mental / esquema)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `session_id` | string | Sesión a la que pertenece. | — |
| `content` | markdown/estructura jerárquica simple | Mapa mental o esquema de repaso rápido. | Debe ser distinguible del resumen — no una repetición literal (FR-005, Acceptance Scenario 2 de US2); esto se valida por forma (estructura jerárquica de nodos cortos), no por calidad de la prosa. |
| `source_fingerprint` | string | Igual patrón que `ClassSummary`. | — |

## ConceptPage (Página de concepto)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `id` | string | Identificador estable dentro de la sesión (`page-<n>`). | Una sesión puede tener varias; cada una es un `ChecklistItemManifest` independiente (`item_id = "page-<n>"`), pueden completarse en cualquier orden (Edge Case de spec.md). |
| `session_id` | string | Sesión a la que pertenece. | — |
| `concept` | string | Concepto que concentra esta página. | No vacío. |
| `content` | markdown | Explicación concentrada del concepto. | No vacío para considerarse `Done`. |
| `source_fingerprint` | string | Igual patrón que `ClassSummary`/`StudyMap`, referido solo a las partes del material relevantes a este concepto. | — |

## ConsolidatedCandidate (Candidato consolidado, acotado a vocabulario en esta feature)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `id` | string | Identificador estable del candidato dentro de la sesión. | — |
| `session_id` | string | Sesión de la que proviene. | — |
| `candidate_type` | enum | `vocabulary` (único variante implementado en esta feature; ver `research.md` §4 y Assumptions de `spec.md`). | Reservado para extender con otros tipos en la feature de la plantilla de gramática, sin romper este esquema. |
| `text` | string | Palabra/expresión candidata. | No vacío. |
| `source_ref` | {origin: "summary" \| "mindmap" \| `"page-<n>"`, locator} | Trazabilidad hacia la parte del análisis de la que proviene (FR-012). | Obligatorio. |
| `already_exists` | bool | Si ya existe una `VocabularyEntry` equivalente (calculado por el handler CLI vía `learnkit-profile::language`, nunca por `learnkit-workflow` — ver `research.md` §4). | Cuando es `true`, el candidato se reporta pero no se propone como nuevo (FR-011/SC-005). |
| `existing_vocabulary_id` | string \| null | `VocabularyEntry.id` ya existente, si `already_exists = true`. | `null` en otro caso. |

## Relaciones (resumen)

```text
Session
  └─ inventory (Source: notas, transcripción)
       └─ analyse (checklist: summary, mindmap, page-1..N)
             ├─ ClassSummary
             ├─ StudyMap
             └─ ConceptPage (0..N)
                    └─ consolidate ──► ConsolidatedCandidate (candidate_type = vocabulary)
                                             │
                                             ▼ (ya existe? consultado vía learnkit-profile::language,
                                                fuera de learnkit-workflow)
                                       VocabularyEntry (feature 002, reutilizada tal cual)
```

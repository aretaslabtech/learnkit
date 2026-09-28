# Data Model: Flujo end-to-end de inglés (clase EOI — vocabulario)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Research**: [research.md](./research.md)

Entidades derivadas de `spec.md` → Key Entities, con la forma concreta ya fijada en `learnkit-implementation-spec/docs/03-domain-model.md` (no se reinventan campos que el domain model ya fija).

## Session

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `id` | string (ULID/UUID) | Identificador estable. | Generado al crear; nunca reasignado. |
| `title` | string | Título descriptivo de la clase. | Obligatorio, no vacío. |
| `profile` | string | Perfil activo de la sesión. | Para esta feature, `language-en` (o un perfil que lo incluya vía composición, `docs/06-profiles.md §7`). |
| `created_at` | timestamp | Momento de creación. | Inmutable. |
| `source_dir` | path | Directorio de fuentes de la sesión. | Relativo a `sessions/<id>/input/`. |
| `workflow` | string | Nombre del workflow/DAG de fases aplicado. | `default` en V1. |

## Source

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `id` | string | Identificador estable (`src-NNN`). | Único dentro de la sesión. |
| `kind` | enum | `text` \| `pdf` \| `image` \| `audio`. | Determinado al inventariar por tipo de fichero. |
| `path` | path | Ruta relativa dentro de la sesión. | — |
| `sha256` | string | Huella de integridad del contenido. | Recalculada en cada `inventory`; si cambia, invalida derivados (FR-003). |
| `status` | enum | `inventoried` \| `stale`. | `stale` cuando `sha256` ya no coincide con la última inventariada. |

## Transcript

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `transcript_id` | string | Identificador estable de esta versión de transcripción. | Cada reimportación/regeneración crea un nuevo `transcript_id`, nunca sobrescribe uno existente (FR-006). |
| `source_id` | string | `Source` de audio del que proviene. | Debe existir y ser `kind = audio`. |
| `engine` | {provider, model, config_fingerprint} \| {imported_from} | Procedencia: motor que la generó, o de dónde se importó. | Obligatorio; nunca vacío — es la base de FR-006 (conservar procedencia). |
| `source_fingerprint` | string | `sha256` del `Source` de audio en el momento de transcribir. | Si el audio cambia, todo transcript con un `source_fingerprint` distinto queda `stale` (FR-006/FR-007). |
| `segments` | list<{start_ms, end_ms, text}> | Texto segmentado con tiempos. | Al menos un segmento para considerarse válido. |
| `active` | bool | Si es la versión que consumen los pasos posteriores. | Exactamente una transcripción activa por `Source`; importar/regenerar puede cambiar cuál es la activa sin borrar las anteriores. |

## VocabularyEntry (perfil `language-en`, `learnkit-profile`)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `id` | string | Identificador estable (`vocab-en-<slug>`). | Único en el proyecto (sobrevive a la sesión, FR-011). |
| `language` / `variety` | string | `en` / `en-GB` por defecto. | Variedad configurable por proyecto/sesión (`docs/06-profiles.md §3`). |
| `lemma` | string | Forma canónica de la palabra/expresión. | Clave de deduplicación (FR-009): dos confirmaciones con el mismo `lemma` reutilizan la entrada existente. |
| `part_of_speech` | string | Categoría gramatical (`phrasal_verb`, `noun`, ...). | — |
| `ipa` | string \| null | Transcripción fonética. | Opcional; puede añadirse manualmente, no se genera automáticamente en esta feature. |
| `senses` | list<{gloss}> | Significado(s). | Al menos uno. |
| `examples` | list<{text}> | Ejemplos de uso. | Puede estar vacío. |
| `sources` | list<{source_id, locator}> | Trazabilidad hasta el origen exacto (nota o segmento de transcript). | Obligatorio al menos una entrada (FR-008). |
| `suggested_by` | enum \| null | `agent-suggested` \| `manual`. | Registra si vino de una sugerencia confirmada (FR-012b) o de creación manual directa (Acceptance Scenario 3 de US3); no afecta validación, solo trazabilidad/analítica. |

## LearningItem (genérico, vinculado 1:1 a una `VocabularyEntry` en este flujo)

Reutiliza la forma ya fijada en `docs/03-domain-model.md §3` (`kind`, `title`, `summary`, `tags`, `sources`, `mastery_dimensions`). Para esta feature: `kind = "vocabulary"`, `mastery_dimensions` incluye como mínimo `recognition`, `production`, `listening` (spec FR-021).

## CardDefinition

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `id` | string | Identificador estable (`card-NNN`). | — |
| `learning_item_ids` | list<string> | Elemento(s) de aprendizaje que representa. | Al menos uno. |
| `template` | string | Plantilla usada (`image-to-production-v1`, `sentence-listening-v1`, ...). | Debe existir en el perfil `language-en` (`docs/06-profiles.md §3`). |
| `front` / `back` | {blocks: list<Block>} | Contenido de cada lado. | Cada `Block` es `text` \| `image(asset_id)` \| `audio(asset_id)`. |
| `completeness` | derivado, no persistido | Se recalcula en cada `validate`/`run cards`/exportación. | `complete` solo si todo bloque `required` por la plantilla tiene un `asset_id` válido (FR-015); nunca se cachea como verdad (Hard Guards). |

## Asset

Reutiliza la forma de `docs/03-domain-model.md §6` (`id`, `type`, `path`, `sha256`, `mime`, `origin`, `metadata`, `validation`). Para esta feature, `origin.kind` toma uno de:

| `origin.kind` | Significado | Campos adicionales |
|----------------|--------------|---------------------|
| `supplied` | Suministrado por el usuario. | — |
| `generated` | Audio de pronunciación generado por voz (FR-017b). | `origin.provider` (p.ej. `piper`), `origin.voice_policy`. |
| `fetched` | Imagen obtenida de Wikimedia Commons (FR-017c). | `origin.provider = "wikimedia-commons"`, `origin.license` (`{name, url}`), `origin.author`, `origin.source_url`. |

**Regla de deduplicación** (FR-016): un `Asset` nuevo con la misma huella lógica (`hash(texto_normalizado + locale + voice_policy)` para audio generado; `hash(page_id + resolución)` para imágenes de Wikimedia; `sha256` de contenido para suministrados) reutiliza el `Asset` existente en vez de crear uno nuevo.

**Regla de licencia** (FR-017d/e): un `Asset` con `origin.kind = "fetched"` DEBE tener `origin.license` no vacío antes de poder asociarse a una tarjeta; si la búsqueda no devuelve ningún resultado con licencia reutilizable, no se crea `Asset` y la tarjeta queda en estado `pending_image` en lugar de vincular una imagen sin licencia válida.

## AssessmentItem / Assessment

Reutiliza la forma de `docs/03-domain-model.md §7-8`. Para esta feature: `skills` (campo `skills` del Assessment Item) toma valores de `{recognition, production, listening}` (FR-021/FR-022); para preguntas de `listening`, el `prompt` referencia un `asset_id` de audio.

## Attempt

Evento inmutable append-only (`attempts.jsonl`), forma de `docs/03-domain-model.md §9`, con `skill` copiado del `AssessmentItem` respondido (FR-024).

## ProgressView

No persistida — vista derivada calculada en el momento de la consulta a partir de `attempts.jsonl`, agregando por `learning_item_id` y por `skill` (FR-025/FR-026): `accuracy_all_time`, `accuracy_recent` (últimos N intentos), `attempt_count`, `last_attempt_at`.

## WorkflowPhase / PhaseManifest (US1, soporte transversal)

| Campo | Tipo | Descripción | Reglas |
|-------|------|--------------|--------|
| `phase` | enum | `inventory` \| `analyse` \| `learn` \| `vocabulary` \| `cards` \| `anki` \| `assessment`. | Definidas en `.learnkit/workflow.toml`. |
| `requires` | list<phase> | Dependencias declaradas. | DAG, no lista rígida (`docs/04-workflow-guards.md §3`). |
| `state` | enum | `not_started` \| `dirty` \| `valid` \| `blocked` \| `failed`. | **Siempre recalculado**, nunca leído como autoridad de un manifest cacheado (Hard Guards). |
| `input_fingerprint` / `output_fingerprint` | string | Huellas usadas para invalidación descendente. | Si cambian los inputs de una fase, la fase y todas las que dependen de ella pasan a `dirty` (`docs/04-workflow-guards.md §8`). |

## Relaciones (resumen)

```text
Session
  └─ Source (audio) ──► Transcript (1..N versiones, 1 activa)
       └─ (agente sugiere, humano confirma) ──► VocabularyEntry ──► LearningItem
                                                        │
                              ┌─────────────────────────┼─────────────────────────┐
                              ▼                                                   ▼
                        CardDefinition ──► Asset(s) ──► .apkg              AssessmentItem ──► Assessment
                                                                                   │
                                                                                   ▼
                                                                          Attempt ──► ProgressView (derivada)
```

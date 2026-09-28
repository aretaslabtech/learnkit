---

description: "Task list template for feature implementation"
---

# Tasks: Flujo end-to-end de inglés (clase EOI — vocabulario)

**Input**: Design documents from `/specs/002-english-eoi-flow/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-commands.md, contracts/provider-traits.md, quickstart.md

**Tests**: se incluyen tareas de test de integración, con el mismo criterio que en la feature 001: `research.md` fija los tests de integración como estrategia de validación. Donde una prueba dependa de una herramienta externa no instalable en CI (whisper.cpp, Piper, red hacia Wikimedia Commons), la tarea usa un provider *fake*/inyectado que respeta el mismo trait, dejando la validación con la herramienta real documentada en `quickstart.md` como paso manual.

**Organization**: tareas agrupadas por historia de usuario (US1-US7 de `spec.md`), en el mismo orden de prioridad que fija el roadmap del proyecto.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (ficheros distintos, sin dependencias pendientes)
- **[Story]**: historia de usuario a la que pertenece (US1-US7)
- Cada tarea incluye la ruta de fichero exacta

## Path Conventions

Workspace Rust ampliado, según `plan.md` → Project Structure:

```text
crates/
├── learnkit-cli/            (existente, ampliado)
├── learnkit-core/           (existente, ampliado)
├── learnkit-store/          (existente, ampliado)
├── learnkit-profile/        (existente, ampliado con módulo language)
├── learnkit-agent/          (existente, ampliado con Skill learnkit-language)
├── learnkit-workflow/       NUEVO
├── learnkit-transcription/  NUEVO
├── learnkit-cards/          NUEVO
├── learnkit-media/          NUEVO
├── learnkit-anki/           NUEVO
└── learnkit-assessment/     NUEVO
tests/fixtures/              fixtures de audio/imagen/notas de ejemplo
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: ampliar el workspace con los crates y dependencias nuevas.

- [X] T001 Añadir al `Cargo.toml` raíz los 6 crates nuevos como miembros del workspace (`crates/learnkit-workflow`, `crates/learnkit-transcription`, `crates/learnkit-cards`, `crates/learnkit-media`, `crates/learnkit-anki`, `crates/learnkit-assessment`), creando para cada uno un `Cargo.toml`/`src/lib.rs` mínimo, según `plan.md` → Project Structure.
- [X] T002 Añadir a `[workspace.dependencies]` en el `Cargo.toml` raíz las dependencias nuevas (`reqwest` con feature `json`, `rusqlite` con feature `bundled`, `zip`, `walkdir`, `serde_yaml`) según `research.md` §1-5, y `[lints] workspace = true` en cada `Cargo.toml` nuevo (mismo patrón que la feature 001).
- [X] T003 [P] Crear `tests/fixtures/eoi-sample/` con: un fichero de notas Markdown de ejemplo, un audio corto sintético (p.ej. un tono/silencio de 2-3s, no una grabación real) suficiente para probar la forma del pipeline sin depender de contenido real de clase, y una imagen de ejemplo; documentar en un `README.md` del propio directorio que la validación con audio/imagen reales de clase se hace siguiendo `quickstart.md`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: infraestructura compartida por 2 o más historias de usuario. Ninguna historia puede implementarse hasta completar esta fase.

**⚠️ CRITICAL**: no empezar Phase 3+ sin terminar esta fase.

- [X] T004 [P] Definir un tipo de error compartido para providers externos (`ProviderError`, con variantes para "herramienta no encontrada", "fallo de ejecución", "salida no parseable") en `crates/learnkit-core/src/provider_error.rs`, usado por `TranscriptionProvider`, `VoiceProvider` e `ImageProvider` (`contracts/provider-traits.md`).
- [X] T005 [P] Implementar los helpers de layout de sesión (`sessions/<id>/input/`, `sessions/<id>/knowledge/`, `sessions/<id>/cards/`, `sessions/<id>/assessments/`, `sessions/<id>/assets/`, `sessions/<id>/validation/`, `sessions/<id>/dist/`, per `docs/10-storage-git.md §2`) en `crates/learnkit-store/src/session_paths.rs` (depende de T001).
- [X] T006 Implementar el motor genérico de fases (`PhaseManifest` con fingerprints de entrada/salida, estados `not_started|dirty|valid|blocked|failed` siempre recalculados, orden topológico sobre las dependencias declaradas en `.learnkit/workflow.toml`) en `crates/learnkit-workflow/src/engine.rs`, según `data-model.md` → WorkflowPhase/PhaseManifest y `research.md` §6 (depende de T001, T004).
- [X] T007 [P] Implementar el registro de `Asset` (estructura, cálculo de huella lógica de deduplicación, validación determinista de existencia/decodificabilidad) en `crates/learnkit-media/src/asset.rs`, según `data-model.md` → Asset (usado por US4 y US5) (depende de T001, T004).
- [X] T008 Registrar en el CLI los esqueletos `clap` (`derive`, sin lógica todavía) de los subcomandos nuevos: `session {new,list,show}`, `ingest`, `inventory`, `transcribe {,import}`, `learn vocabulary add`, `cards {build,validate}`, `run <phase>`, `validate`, `export {anki,exam}`, `attempt import`, `progress`, ampliando `learnkit status` con `--session <id>`, en `crates/learnkit-cli/src/main.rs` y `crates/learnkit-cli/src/commands/mod.rs` (depende de T001; reutiliza el patrón de despacho de la feature 001).

**Checkpoint**: infraestructura lista — las historias de usuario pueden implementarse (en paralelo si hay más de una persona, respetando que US2 en adelante necesita que exista una sesión, ver Dependencies).

---

## Phase 3: User Story 1 - Registrar una sesión de clase con sus fuentes (Priority: P1) 🎯 MVP

**Goal**: `learnkit session new`, `learnkit ingest`, `learnkit inventory` crean y mantienen la sesión y sus fuentes de forma trazable e idempotente.

**Independent Test**: Escenario 1 de `quickstart.md`.

### Tests for User Story 1

- [X] T009 [P] [US1] Test de integración: `learnkit session new` crea la sesión con `id` estable y directorio propio en `crates/learnkit-cli/tests/session_test.rs`.
- [X] T010 [P] [US1] Test de integración: `ingest` + `inventory` registran notas/audio/imagen con tipo, tamaño y `sha256` (FR-002) en `crates/learnkit-cli/tests/inventory_test.rs`.
- [X] T011 [P] [US1] Test de integración: repetir `inventory` sin cambios no duplica fuentes (FR-003) en `crates/learnkit-cli/tests/inventory_test.rs`.
- [X] T012 [P] [US1] Test de integración: modificar el contenido de una fuente ya inventariada y volver a inventariar la marca `stale` con nueva huella (FR-003) en `crates/learnkit-cli/tests/inventory_test.rs`.

### Implementation for User Story 1

- [X] T013 [US1] Implementar el modelo `Session` (creación, listado, lectura) en `crates/learnkit-workflow/src/session.rs`, según `data-model.md` → Session (depende de T005).
- [X] T014 [US1] Implementar la ingesta/inventario de `Source` (copia dentro de la sesión, detección de tipo, `sha256`, transición `inventoried`/`stale`) en `crates/learnkit-workflow/src/inventory.rs`, según `data-model.md` → Source (depende de T005, T013).
- [X] T015 [US1] Registrar la fase `inventory` (`requires = []`) en el motor de fases de T006, en `crates/learnkit-workflow/src/phases.rs`.
- [X] T016 [US1] Implementar los handlers CLI `session new/list/show`, `ingest`, `inventory` en `crates/learnkit-cli/src/commands/session.rs` e `crates/learnkit-cli/src/commands/inventory.rs` (depende de T013, T014, T015, T008).
- [X] T017 [US1] Ampliar `learnkit status --session <id>` (de la feature 001) para incluir el estado de las fases del workflow de esa sesión, en `crates/learnkit-cli/src/commands/status.rs` (depende de T006, T015).

**Checkpoint**: User Story 1 completa y comprobable de forma independiente — MVP entregable (organización/trazabilidad de material de clase).

---

## Phase 4: User Story 2 - Obtener un transcript fiable del audio de clase (Priority: P2)

**Goal**: transcribir o importar una transcripción de una fuente de audio, con procedencia conservada e invalidación cuando el audio cambia.

**Independent Test**: Escenario 2 de `quickstart.md`.

### Tests for User Story 2

- [X] T018 [P] [US2] Test de integración: transcribir con un `TranscriptionProvider` *fake* inyectado produce un `Transcript` con segmentos y tiempos (FR-004) en `crates/learnkit-transcription/tests/transcribe_test.rs`.
- [X] T019 [P] [US2] Test de integración: importar un `.srt` de ejemplo para la misma fuente produce un `Transcript` con la misma forma sin invocar ningún provider de generación (FR-005) en `crates/learnkit-transcription/tests/import_test.rs`.
- [X] T020 [P] [US2] Test de integración: cambiar el `sha256` de la fuente de audio marca como `stale` la transcripción existente (FR-006) en `crates/learnkit-transcription/tests/invalidation_test.rs`.
- [X] T021 [P] [US2] Test de integración: `learnkit run <phase-que-requiere-transcripción>` sin transcripción disponible termina en `BLOCKED`/exit 20 (FR-007) en `crates/learnkit-cli/tests/transcribe_test.rs`.

### Implementation for User Story 2

- [X] T022 [US2] Definir el trait `TranscriptionProvider` y `TranscriptDraft` en `crates/learnkit-transcription/src/provider.rs`, según `contracts/provider-traits.md` (depende de T004).
- [X] T023 [US2] Implementar `WhisperCppProvider` (invocación como subproceso, parseo de su salida a `TranscriptDraft`) en `crates/learnkit-transcription/src/whisper_cpp.rs`, según `research.md` §1 (depende de T022).
- [X] T024 [P] [US2] Implementar `TranscriptImporter` para SRT/VTT/TXT/JSON en `crates/learnkit-transcription/src/import.rs` (depende de T022).
- [X] T025 [US2] Implementar el escritor canónico `transcript.json` (múltiples versiones por `Source`, versión activa, `source_fingerprint`) en `crates/learnkit-transcription/src/transcript.rs`, según `data-model.md` → Transcript (depende de T023, T024, T005).
- [X] T026 [US2] Registrar la fase `analyse` (`requires = ["inventory"]`, guard: toda fuente de audio requerida tiene transcripción activa y válida) en `crates/learnkit-workflow/src/phases.rs` (depende de T006, T025).
- [X] T027 [US2] Implementar los handlers CLI `transcribe` y `transcribe import` en `crates/learnkit-cli/src/commands/transcribe.rs` (depende de T025, T026, T008).

**Checkpoint**: User Stories 1 y 2 funcionan de forma independiente (sesión con transcripción fiable, sin vocabulario todavía).

---

## Phase 5: User Story 3 - Extraer vocabulario y contenido de aprendizaje de la clase (Priority: P3)

**Goal**: confirmar (manualmente o a partir de sugerencias del agente) entradas de vocabulario con trazabilidad, generando su `LearningItem` vinculado.

**Independent Test**: Escenario 3 de `quickstart.md`.

### Tests for User Story 3

- [X] T028 [P] [US3] Test de integración: `learn vocabulary add` crea `VocabularyEntry` + `LearningItem` vinculado con trazabilidad de origen (FR-008/010) en `crates/learnkit-cli/tests/vocabulary_test.rs`.
- [X] T029 [P] [US3] Test de integración: repetir el mismo `lemma` en otra sesión reutiliza la entrada existente en vez de duplicarla (FR-009) en `crates/learnkit-cli/tests/vocabulary_test.rs`.
- [X] T030 [P] [US3] Test de integración: añadir vocabulario manualmente sin ninguna sugerencia previa funciona igual (Acceptance Scenario 3 de US3) en `crates/learnkit-cli/tests/vocabulary_test.rs`.
- [X] T031 [P] [US3] Test de integración: `session show --json` incluye el texto de las notas y los segmentos de la transcripción activa (necesario para que la Skill del agente pueda sugerir candidatos, `research.md` §2) en `crates/learnkit-cli/tests/session_test.rs`.

### Implementation for User Story 3

- [X] T032 [US3] Implementar `VocabularyEntry` (creación, deduplicación por `lemma`) en `crates/learnkit-profile/src/language/vocabulary.rs`, según `data-model.md` → VocabularyEntry (depende de T001).
- [X] T033 [US3] Implementar la creación automática del `LearningItem` genérico vinculado a cada `VocabularyEntry` en `crates/learnkit-profile/src/language/learning_item.rs` (depende de T032).
- [X] T034 [US3] Ampliar `session show --json` para incluir el texto de las notas y los segmentos de la transcripción activa en `crates/learnkit-cli/src/commands/session.rs` (depende de T013, T025).
- [X] T035 [US3] Implementar el handler CLI `learn vocabulary add` en `crates/learnkit-cli/src/commands/learn.rs` (depende de T032, T033, T008).
- [X] T036 [US3] Registrar las fases `learn`/`vocabulary` (`requires = ["analyse"]`) en `crates/learnkit-workflow/src/phases.rs` (depende de T006, T035).
- [X] T037 [US3] Crear la Skill de agente `learnkit-language` (`SKILL.md`, instrucciones para leer `session show --json`, proponer candidatos de vocabulario conversacionalmente, y confirmar con `learn vocabulary add`) embebida vía el mecanismo de plantillas de agente ya existente (`crates/learnkit-agent/templates/skills/learnkit-language/SKILL.md` + registro en `crates/learnkit-agent/src/templates.rs`), según `research.md` §2 (depende del mecanismo de plantillas ya construido en la feature 001).

**Checkpoint**: User Stories 1-3 funcionan de forma independiente (vocabulario reutilizable, con o sin ayuda de un agente).

---

## Phase 6: User Story 4 - Crear tarjetas de estudio con imagen y audio (Priority: P4)

**Goal**: generar tarjetas completas a partir de vocabulario, con recursos suministrados, generados por voz, o buscados en Wikimedia Commons con atribución.

**Independent Test**: Escenario 4 de `quickstart.md`.

### Tests for User Story 4

- [X] T038 [P] [US4] Test de integración: generar una tarjeta de producción con imagen suministrada crea el bloque correcto en cada lado (FR-013) en `crates/learnkit-cards/tests/build_test.rs`.
- [X] T039 [P] [US4] Test de integración: una tarjeta de listening soporta audio distinto e independiente en frontal y reverso (FR-014) en `crates/learnkit-cards/tests/build_test.rs`.
- [X] T040 [P] [US4] Test de integración: falta un recurso obligatorio de la plantilla → la tarjeta no se considera completa (FR-015) en `crates/learnkit-cards/tests/validate_test.rs`.
- [X] T041 [P] [US4] Test de integración: dos tarjetas que necesitan el mismo recurso lo reutilizan sin duplicar el `Asset` (FR-016) en `crates/learnkit-media/tests/dedup_test.rs`.
- [X] T042 [P] [US4] Test de integración: con un `VoiceProvider` *fake* inyectado, una entrada sin audio propio recibe audio generado (FR-017b) en `crates/learnkit-media/tests/voice_test.rs`.
- [X] T043 [P] [US4] Test de integración: con un `ImageProvider` *fake* inyectado, una entrada sin imagen propia recibe una imagen con licencia registrada (FR-017c/d), y si el fake no devuelve candidatos la tarjeta queda `pending_image` (FR-017e) en `crates/learnkit-media/tests/image_test.rs`.

### Implementation for User Story 4

- [X] T044 [US4] Implementar `CardDefinition` y las plantillas de idiomas (`image-to-production-v1`, `audio-to-text-v1`, `sentence-listening-v1`, `expression-production-v1`, `word-to-meaning-v1`, per `docs/06-profiles.md §3`) en `crates/learnkit-cards/src/card.rs` y `crates/learnkit-cards/src/template.rs`, según `data-model.md` → CardDefinition (depende de T001).
- [X] T045 [US4] Implementar las políticas de media por plantilla y lado (`required`/`optional`/`disabled`) y el cálculo de `completeness` en `crates/learnkit-cards/src/template.rs` (depende de T044).
- [X] T046 [US4] Definir el trait `VoiceProvider` e implementar `PiperVoiceProvider` (subproceso) en `crates/learnkit-media/src/voice.rs`, según `contracts/provider-traits.md` y `research.md` §3 (depende de T004, T007).
- [X] T047 [US4] Definir el trait `ImageProvider` e implementar `WikimediaCommonsProvider` (HTTP vía `reqwest`, filtro de licencia) en `crates/learnkit-media/src/image.rs`, según `contracts/provider-traits.md` y `research.md` §4 (depende de T004, T007).
- [X] T048 [US4] Implementar el flujo de resolución de recursos por tarjeta (suministrado → generado/buscado → `pending_*`) y el registro de licencia/atribución del `Asset` en `crates/learnkit-media/src/resolve.rs` (depende de T007, T046, T047).
- [X] T049 [US4] Implementar los handlers CLI `cards build` y `cards validate` en `crates/learnkit-cli/src/commands/cards.rs` (depende de T044, T045, T048, T008).
- [X] T050 [US4] Registrar la fase `cards` (`requires = ["vocabulary"]`) en `crates/learnkit-workflow/src/phases.rs` (depende de T006, T049).

**Checkpoint**: User Stories 1-4 funcionan de forma independiente (tarjetas completas listas para exportar).

---

## Phase 7: User Story 5 - Exportar las tarjetas a un mazo importable en Anki (Priority: P5)

**Goal**: producir un `.apkg` autocontenido y válido a partir de tarjetas completas, sin duplicar en reexportaciones.

**Independent Test**: Escenario 5 de `quickstart.md` (con validación manual final en Anki Desktop).

### Tests for User Story 5

- [X] T051 [P] [US5] Test de integración: exportar un mazo produce un `.apkg` con estructura ZIP/SQLite válida (colección + al menos una nota con media) en `crates/learnkit-anki/tests/export_test.rs`.
- [X] T052 [P] [US5] Test de integración: exportar con una tarjeta incompleta falla explícitamente (tarjeta+recurso exactos) y no deja ningún `.apkg` parcial en disco (FR-019) en `crates/learnkit-cli/tests/export_anki_test.rs`.
- [X] T053 [P] [US5] Test de integración: el GUID de una nota es estable entre dos exportaciones sucesivas de las mismas tarjetas (FR-020) en `crates/learnkit-anki/tests/guid_test.rs`.

### Implementation for User Story 5

- [X] T054 [US5] **Spike técnico** (obligatorio per `docs/07-anki-media.md §10`): generar un `collection.anki2` mínimo con `rusqlite` (esquema de notas/cards/decks) y verificar su estructura en `crates/learnkit-anki/src/spike.rs`; documentar el resultado (éxito o necesidad del plan B de `research.md` §5) en `research.md` antes de continuar con T055-T057.
- [X] T055 [US5] Implementar la derivación determinista de GUID (`sha256(card_id)` truncado al rango esperado) en `crates/learnkit-anki/src/guid.rs` (depende de T054).
- [X] T056 [US5] Implementar el empaquetado del mazo (bundle de media + `media.json` + ZIP) en `crates/learnkit-anki/src/package.rs` (depende de T054, T055, T007).
- [X] T057 [US5] Implementar el handler CLI `export anki` con escritura atómica (temporal + rename solo si toda tarjeta es `complete`) en `crates/learnkit-cli/src/commands/export_anki.rs` (depende de T045, T056, T008).
- [X] T058 [US5] Registrar la fase `anki` (`requires = ["cards"]`) en `crates/learnkit-workflow/src/phases.rs` (depende de T006, T057).

**Checkpoint**: User Stories 1-5 funcionan de forma independiente (mazo Anki real e importable — cierra el requisito no negociable de interoperabilidad).

---

## Phase 8: User Story 6 - Evaluar el dominio del vocabulario (Priority: P6)

**Goal**: generar y completar un examen local que cubra recognition/production/listening, registrando cada respuesta como evento inmutable.

**Independent Test**: Escenario 6 de `quickstart.md`.

### Tests for User Story 6

- [X] T059 [P] [US6] Test de integración: el banco de preguntas generado cubre las tres destrezas y cada pregunta referencia el vocabulario que evalúa (FR-021/022) en `crates/learnkit-assessment/tests/build_test.rs`.
- [X] T060 [P] [US6] Test de integración: el HTML de examen generado es autocontenido (sin referencias externas) y calcula una puntuación en el propio fichero (FR-023) en `crates/learnkit-assessment/tests/exam_html_test.rs`.
- [X] T061 [P] [US6] Test de integración: `attempt import` es append-only — importar el mismo fichero de resultados dos veces no sobrescribe intentos previos (FR-024) en `crates/learnkit-cli/tests/attempt_test.rs`.

### Implementation for User Story 6

- [X] T062 [US6] Implementar `AssessmentItem`/`Assessment` (identidad estable, respuesta correcta explícita, scoring, feedback) en `crates/learnkit-assessment/src/item.rs`, según `data-model.md` → AssessmentItem/Assessment (depende de T001).
- [X] T063 [US6] Implementar la generación de preguntas por destreza (recognition/production/listening) a partir de `LearningItem`s de vocabulario en `crates/learnkit-assessment/src/build.rs` (depende de T062, T033).
- [X] T064 [US6] Implementar el renderer HTML autocontenido del examen (aleatorización, autocorrección de tipos objetivos, botón de descarga de `results.json`) en `crates/learnkit-assessment/src/exam_html.rs` (depende de T062).
- [X] T065 [US6] Implementar el escritor append-only de `attempts.jsonl` en `crates/learnkit-assessment/src/attempt.rs`, según `data-model.md` → Attempt (depende de T005).
- [X] T066 [US6] Implementar los handlers CLI `assessment build`, `export exam`, `attempt import` en `crates/learnkit-cli/src/commands/assessment.rs` (depende de T063, T064, T065, T008).
- [X] T067 [US6] Registrar la fase `assessment` (`requires = ["vocabulary"]`) en `crates/learnkit-workflow/src/phases.rs` (depende de T006, T066).

**Checkpoint**: User Stories 1-6 funcionan de forma independiente (evaluación real por destreza, no solo memorización).

---

## Phase 9: User Story 7 - Ver el progreso por destreza (Priority: P7)

**Goal**: agregar los intentos de evaluación por elemento de aprendizaje y por destreza, siempre recalculado.

**Independent Test**: Escenario 7 de `quickstart.md`.

### Tests for User Story 7

- [X] T068 [P] [US7] Test de integración: `progress` agrega precisión y número de intentos por `learning_item_id` y por `skill` (FR-025) en `crates/learnkit-assessment/tests/progress_test.rs`.
- [X] T069 [P] [US7] Test de integración: añadir un nuevo intento a `attempts.jsonl` y volver a consultar `progress` refleja el cambio inmediatamente, sin valor cacheado (FR-026) en `crates/learnkit-assessment/tests/progress_test.rs`.
- [X] T070 [P] [US7] Test de integración: `progress --skill listening` filtra correctamente la vista a esa destreza en `crates/learnkit-cli/tests/progress_test.rs`.

### Implementation for User Story 7

- [X] T071 [US7] Implementar la agregación `ProgressView` (`accuracy_all_time`, `accuracy_recent`, `attempt_count`, `last_attempt_at`, por `learning_item_id` y por `skill`) en `crates/learnkit-assessment/src/progress.rs` (depende de T065).
- [X] T072 [US7] Implementar el handler CLI `progress` (con `--session`/`--tag`/`--skill`) en `crates/learnkit-cli/src/commands/progress.rs` (depende de T071, T008).

**Checkpoint**: las 7 historias de usuario funcionan de forma independiente — recorrido completo cerrado.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: mejoras que afectan a varias historias de usuario.

- [X] T073 [P] Ejecutar los 7 escenarios de `quickstart.md` (los automatizables contra el binario compilado; el de importación en Anki Desktop, manualmente) y corregir cualquier discrepancia encontrada.
- [X] T074 [P] Verificar las licencias de las dependencias nuevas (`reqwest`, `rusqlite`, `zip`, `walkdir`, `serde_yaml`) con `cargo metadata` (mismo procedimiento que la feature 001) y documentar el resultado en `research.md` §8.
- [X] T075 Implementar el formateo de salida legible por humanos para los comandos nuevos (`session`, `transcribe`, `learn`, `cards`, `export anki`, `export exam`, `attempt`, `assessment`, `progress`) en cada handler correspondiente de `crates/learnkit-cli/src/commands/`.
- [X] T076 [P] Pasar `cargo fmt --all` y `cargo clippy --workspace --all-targets` sobre los 6 crates nuevos y corregir cualquier warning.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias.
- **Foundational (Phase 2)**: depende de Setup — bloquea todas las historias de usuario.
- **User Stories (Phase 3-9)**: dependen de Foundational. A diferencia de la feature 001, aquí las historias forman una **cadena secuencial de valor** (docs/11-v1-roadmap.md): cada historia usa entidades que la anterior crea (US2 necesita una `Source` de audio de US1; US3 necesita una `Transcript` de US2 o al menos notas de US1; US4 necesita `LearningItem`s de US3; US5 necesita `CardDefinition`s de US4; US6 necesita `LearningItem`s de US3; US7 necesita `Attempt`s de US6). Cada historia sigue siendo *independientemente implementable y probable* con fixtures propios (no hace falta completar la anterior para escribir/probar el código), pero el recorrido de demo end-to-end solo se completa en orden P1→P7.
- **Polish (Phase 10)**: depende de que las historias deseadas estén completas.

### Within Each User Story

- Tests antes de la implementación correspondiente.
- Modelos/entidades antes de los handlers CLI.
- Registro de la fase en el motor de workflow (T006) al final de cada historia, una vez su lógica de dominio existe.

### Parallel Opportunities

- T003 (fixtures) en paralelo con T001/T002.
- T004, T005, T007 (Foundational) en paralelo entre sí; T006 y T008 tienen dependencias directas indicadas.
- Todos los tests marcados [P] de una misma historia, en paralelo entre sí.
- Dentro de US4: T046 (voz) y T047 (imagen) en paralelo, ya que son providers independientes.
- Distintas personas pueden trabajar en historias distintas en paralelo una vez Foundational está completo, siempre que cada una use sus propios fixtures en lugar de depender del resultado real de la historia anterior.

---

## Parallel Example: User Story 4

```bash
# Lanzar juntos los tests de la historia 4:
Task: "Test de integración: tarjeta de producción con imagen suministrada en crates/learnkit-cards/tests/build_test.rs"
Task: "Test de integración: audio independiente frontal/reverso en crates/learnkit-cards/tests/build_test.rs"
Task: "Test de integración: recurso obligatorio faltante en crates/learnkit-cards/tests/validate_test.rs"
Task: "Test de integración: dedup de Asset en crates/learnkit-media/tests/dedup_test.rs"
Task: "Test de integración: VoiceProvider fake genera audio en crates/learnkit-media/tests/voice_test.rs"
Task: "Test de integración: ImageProvider fake devuelve licencia o pending_image en crates/learnkit-media/tests/image_test.rs"

# Lanzar juntos los dos providers independientes:
Task: "VoiceProvider + PiperVoiceProvider en crates/learnkit-media/src/voice.rs"
Task: "ImageProvider + WikimediaCommonsProvider en crates/learnkit-media/src/image.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Completar Phase 1: Setup.
2. Completar Phase 2: Foundational.
3. Completar Phase 3: User Story 1.
4. **Parar y validar**: Escenario 1 de `quickstart.md`.
5. Organización/trazabilidad de sesiones de clase ya es utilizable, incluso sin transcripción ni vocabulario todavía.

### Incremental Delivery

1. Setup + Foundational → base lista.
2. US1 → sesión y fuentes trazables → MVP.
3. US2 → transcripción fiable y sustituible → base para vocabulario.
4. US3 → vocabulario reutilizable (con o sin agente) → base para tarjetas y evaluación.
5. US4 → tarjetas completas con media resuelta automáticamente → base para exportar.
6. US5 → mazo Anki real → cierra el requisito no negociable de interoperabilidad.
7. US6 → evaluación real por destreza → distinto de memorizar con tarjetas.
8. US7 → progreso agregado → cierra el ciclo aprendizaje→evaluación→progreso.

### Parallel Team Strategy

Con varias personas, una vez completado Foundational: cada historia puede asignarse a una persona distinta si cada una usa fixtures propios (no el resultado real de la historia anterior) para no bloquearse; la demo end-to-end final sí requiere las 7 completas en orden.

---

## Phase 11: Sesión activa (FR-003b — añadida post-release tras uso real, 2026-09-27)

**Purpose**: evitar repetir `--session <id>` en cada comando, análogo al `.specify/feature.json` que ya usa este mismo repositorio. Ver `research.md` §6b.

- [X] T077 [P] [US1] Implementar `learnkit-store::active_session` (`set_active`/`get_active` sobre `.learnkit/active-session.json`, escritura atómica) en `crates/learnkit-store/src/active_session.rs`.
- [X] T078 [US1] Registrar `active_session` en `crates/learnkit-store/src/lib.rs` y crear el helper `resolve_session(root, explicit: Option<String>) -> Result<String, LearnKitError>` reutilizado por todos los comandos de sesión en `crates/learnkit-cli/src/commands/session.rs` (depende de T077).
- [X] T079 [US1] Añadir el subcomando `learnkit session use <id>` (valida que la sesión existe, fija la activa) en `crates/learnkit-cli/src/commands/session.rs` (depende de T078).
- [X] T080 [US1] Hacer que `session new` fije automáticamente la sesión creada como activa en `crates/learnkit-cli/src/commands/session.rs` (depende de T078).
- [X] T081 [US1] Hacer `--session`/`<id>` opcional (cae a la sesión activa si se omite) en `ingest`, `inventory`, `session show`, `transcribe`, `transcribe import`, `learn vocabulary add`, `cards build`, `cards validate`, `export anki`, `export exam`, `assessment build`, `attempt import` — todos los comandos de `crates/learnkit-cli/src/commands/*.rs` que hoy exigen `--session` (depende de T078).
- [X] T082 [P] [US1] Test de integración: `session new` fija la sesión activa y los comandos posteriores funcionan sin `--session` en `crates/learnkit-cli/tests/active_session_test.rs`.
- [X] T083 [P] [US1] Test de integración: `session use <id>` cambia la sesión activa; comandos sin `--session` a partir de ahí usan la nueva en `crates/learnkit-cli/tests/active_session_test.rs`.
- [X] T084 [P] [US1] Test de integración: sin sesión activa y sin `--session`, el comando falla explícitamente (exit code `2`) en `crates/learnkit-cli/tests/active_session_test.rs`.

---

## Phase 12: Idempotencia de `cards build` (FR-017f — añadida post-release tras uso real, 2026-09-27)

**Purpose**: evitar que un reintento de `cards build` reprocese tarjetas ya completas y las convierta en `pending_image`/`pending_audio` por la intermitencia del proveedor de imágenes (Wikimedia Commons). Ver FR-017f.

- [X] T085 [US4] Hacer que `run_build` en `crates/learnkit-cli/src/commands/cards.rs` cargue las tarjetas ya existentes de la sesión (`learnkit_cards::card::load_all`) y, para cada `learning_item`, si ya existe una tarjeta con `completeness() == Complete` para la plantilla solicitada, la reutilice sin llamar a `resolve_image`/`resolve_audio` de nuevo.
- [X] T086 [US4] Añadir el flag `--force` a `learnkit cards build` que, cuando se pasa, ignora las tarjetas existentes y regenera todo el conjunto (comportamiento previo) en `crates/learnkit-cli/src/commands/cards.rs` (depende de T085).
- [X] T087 [P] [US4] Test de integración: dos ejecuciones consecutivas de `cards build` no vuelven a invocar el proveedor de imágenes para una tarjeta ya completa (usando un fake que falla en la segunda llamada) en `crates/learnkit-cli/tests/cards_test.rs` (depende de T085).
- [X] T088 [P] [US4] Test de integración: `cards build --force` sí reprocesa tarjetas ya completas en `crates/learnkit-cli/tests/cards_test.rs` (depende de T086).

---

## Phase 13: Provider de voz REST por defecto (FR-017b revisado — añadida post-release tras uso real, 2026-09-27)

**Purpose**: sustituir `Piper` como provider de voz por defecto por un `RestVoiceProvider` que llama al servicio TTS REST ya operado por el usuario, porque Piper no está instalado en la máquina real de uso. Ver `research.md` §3 (revisión).

- [X] T089 [US4] Implementar `RestVoiceProvider` (implementa `VoiceProvider`) en `crates/learnkit-media/src/voice.rs`: `POST {base_url}/v1/audio/speech` con cuerpo `{model, voice, input, response_format: "mp3"}`, cabecera `Authorization: Bearer {api_key}`, `base_url`/`api_key`/`voice` configurables (constructor + valores por defecto `https://tts.davidpalazon.net` / env var `TTS_API_KEY` / `en-GB-SoniaNeural`).
- [X] T090 [US4] Test unitario: `RestVoiceProvider` con un servidor HTTP fake (usando el mismo patrón que `image_test.rs`/mocks del crate) — éxito devuelve `AudioAssetDraft` con los bytes del cuerpo; fallo HTTP o falta de API key devuelve `ProviderError` (`NetworkError`/`ExecutionFailed`) en `crates/learnkit-media/src/voice.rs`.
- [X] T091 [US4] Cambiar `crates/learnkit-cli/src/commands/cards.rs` para instanciar `RestVoiceProvider::default()` en vez de `PiperVoiceProvider` como provider de voz por defecto de `cards build` (depende de T089).
- [X] T092 [US4] Documentar en `README.md`/`docs` que `TTS_API_KEY` es una variable de entorno requerida para la generación de audio por defecto, y que `PiperVoiceProvider` sigue disponible como alternativa 100% offline (sin credencial ni red) para quien prefiera ese flujo.

---

## Phase 14: `learn vocabulary remove` (FR-012c — añadida post-release tras uso real, 2026-09-27)

**Purpose**: David detectó que no existe ninguna forma de eliminar una entrada de vocabulario (ni la tarjeta asociada) salvo editando `knowledge/vocabulary/*.yaml`/`knowledge/learning-items/*.yaml` a mano, lo cual contradice la convención del proyecto de mutar el estado solo a través del CLI (ver `crates/learnkit-agent/templates/CLAUDE.md`).

- [X] T093 [P] [US3] Añadir `pub fn remove(project_root, id) -> io::Result<bool>` en `crates/learnkit-profile/src/language/vocabulary.rs` (borra `knowledge/vocabulary/{id}.yaml`; `false` si no existía).
- [X] T094 [P] [US3] Hacer pública `find_by_vocabulary_entry` y añadir `pub fn remove(project_root, id) -> io::Result<bool>` en `crates/learnkit-profile/src/language/learning_item.rs` (borra `knowledge/learning-items/{id}.yaml`).
- [X] T095 [US4] Añadir `pub fn remove(session_root, id) -> io::Result<bool>` en `crates/learnkit-cards/src/card.rs` (borra `cards/{id}.yaml`) (depende de T085 ya existente en `card.rs`).
- [X] T096 [US3] Implementar `learnkit learn vocabulary remove --lemma <lemma>` en `crates/learnkit-cli/src/commands/learn.rs`: busca la entrada por lema (proyecto entero, FR-011), su elemento de aprendizaje asociado, recorre todas las sesiones bajo `sessions/*` buscando tarjetas cuyo `learning_item_ids` la referencien y las borra, y finalmente borra el elemento de aprendizaje y la entrada de vocabulario. Falla explícitamente si el lema no existe, reutilizando `LearnKitError::ExporterConstraint` (exit 40) como ya hace `cards build` para "unknown template" (depende de T093, T094, T095).
- [X] T097 [P] [US3] Test de integración: `learn vocabulary remove` elimina la entrada, el elemento de aprendizaje, y cualquier tarjeta que lo referenciara, y una segunda llamada al mismo lema falla en `crates/learnkit-cli/tests/vocabulary_test.rs` (depende de T096).

---

## Phase 15: Reintento de búsqueda de imágenes ante fallo transitorio (FR-017g — añadida post-release tras uso real, 2026-09-27)

**Purpose**: David observó que el conjunto de palabras `pending_image` cambiaba casi por completo entre dos ejecuciones consecutivas de `cards build` sobre el mismo vocabulario — señal de intermitencia de red, no de palabras no ilustrables. Causa raíz confirmada: `resolve_image` no distinguía un fallo transitorio del proveedor de un "sin candidatos" definitivo. Ver `research.md` §4 (revisión).

- [X] T098 [US4] En `crates/learnkit-media/src/resolve.rs`, hacer que `resolve_image` reintente `image_provider.search()` hasta 3 intentos en total (con una breve espera creciente entre intentos) cuando el proveedor devuelve `Err`, y solo entonces (agotados los reintentos) devuelva `ResolvedMedia::Pending`. Un `Ok([])` (sin candidatos con licencia válida) sigue sin reintentarse.
- [X] T099 [P] [US4] Test unitario: un `ImageProvider` fake que falla las dos primeras llamadas y tiene éxito a la tercera hace que `resolve_image` devuelva un `Asset` (no `Pending`) en `crates/learnkit-media/src/resolve.rs` (depende de T098).
- [X] T100 [P] [US4] Test unitario: un `ImageProvider` fake que siempre falla agota los reintentos y `resolve_image` devuelve `Pending` con un mensaje que menciona el número de intentos, en `crates/learnkit-media/src/resolve.rs` (depende de T098).

---

## Phase 16: Exportar excluyendo tarjetas incompletas (FR-019b — añadida post-release tras uso real, 2026-09-28)

**Purpose**: David tenía 126/127 tarjetas completas de una sesión real; la única incompleta (`ty numbers (30-90)`, sin imagen posible por ser un concepto abstracto) bloqueaba `export anki` por completo, sin forma de exportar el resto mientras se resolvía esa tarjeta suelta. Ver FR-019b.

- [X] T101 [US5] Añadir el flag `--skip-incomplete` a `learnkit export anki` en `crates/learnkit-cli/src/commands/export_anki.rs`: cuando se pasa, las tarjetas incompletas se excluyen del `.apkg` en vez de bloquear toda la exportación; el resultado (`--json` y humano) DEBE listar qué tarjetas se excluyeron y por qué (mismo shape que `incomplete_cards` ya usa hoy para el caso de fallo). Sin el flag, el comportamiento por defecto sigue siendo FR-019 (falla si hay alguna incompleta).
- [X] T102 [P] [US5] Test de integración: `export anki` sin `--skip-incomplete` sigue fallando (exit 40) si hay una tarjeta incompleta, exactamente como antes (regresión de FR-019) en `crates/learnkit-cli/tests/export_anki_test.rs`.
- [X] T103 [P] [US5] Test de integración: `export anki --skip-incomplete` con una tarjeta incompleta entre varias completas exporta un `.apkg` válido con solo las completas, y el resultado indica qué tarjeta se excluyó y por qué en `crates/learnkit-cli/tests/export_anki_test.rs`.

---

## Phase 17: Diagnóstico de recursos opcionales no resueltos (FR-017h — añadida post-release tras uso real, 2026-09-28)

**Purpose**: David tenía 127/127 tarjetas `complete` sin ninguna con audio — `TTS_API_KEY` no estaba configurada, pero al ser el audio opcional en la plantilla, `cards build`/`cards validate`/`status` nunca lo reportaron; la causa solo se pudo confirmar leyendo el YAML de una tarjeta a mano. Ver FR-017h.

- [X] T104 [US4] En `crates/learnkit-cli/src/commands/cards.rs`, hacer que `resolved_id` reciba la política del recurso y acumule un `MediaWarning { card_id, side, kind, reason }` cuando un recurso `Optional` no se resuelve (nunca para `Required`, que ya se refleja en `completeness()`); `cards build` incluye `media_warnings` en su salida `--json` y las imprime en modo humano.
- [X] T105 [P] [US4] Test unitario: un recurso `Optional` sin resolver registra un `MediaWarning`; uno `Required` no (ya cubierto por `completeness()`); un recurso resuelto no registra nada — en `crates/learnkit-cli/src/commands/cards.rs`.

---

## Phase 18: Formato correcto de tarjeta de vocabulario — IPA, ejemplo, audio en ambos lados (FR-013b — añadida post-release tras uso real, 2026-09-28)

**Purpose**: gap #2 de la Guía maestra (§10.1/10.2), identificado hace tiempo y ahora confirmado explícitamente por David: la plantilla `image-to-production-v1` no coincide con el formato pedagógico real — anverso sin audio, reverso sin IPA/ejemplo/palabra escrita, y el audio del reverso pronunciaba la traducción en castellano con voz inglesa. Ver FR-013b.

- [X] T106 [US4] Añadir `ipa: Option<String>` y `examples: Vec<Example>` (nuevo `struct Example { text: String }`) a `VocabularyEntry` en `crates/learnkit-profile/src/language/vocabulary.rs`; ambos opcionales, sin romper la deserialización de entradas ya persistidas sin estos campos (`#[serde(default)]`).
- [X] T107 [US4] Añadir `--ipa <texto>` y `--example <texto>` (repetible) a `learnkit learn vocabulary add` en `crates/learnkit-cli/src/commands/learn.rs`, persistidos en la `VocabularyEntry` (depende de T106).
- [X] T108 [US4] Cambiar `image-to-production-v1` en `crates/learnkit-cards/src/template.rs`: `front_audio` de `Disabled` a `Required`. `back_audio` se mantiene, pero su recurso se deriva del mismo asset ya resuelto en el anverso, nunca de una llamada de generación independiente (depende de T106 solo conceptualmente, no de código).
- [X] T109 [US4] En `crates/learnkit-cli/src/commands/cards.rs`, reescribir la construcción de bloques de `image-to-production-v1`: anverso = imagen + audio de pronunciación (generado a partir de `item.title`, la palabra en inglés — nunca de `item.summary`); reverso = texto con la palabra (`item.title`), texto con la traducción (`item.summary`), texto con el IPA (si `VocabularyEntry.ipa` existe — requiere pasar ese dato hasta `cards build`, no solo lo que ya expone `LearningItem`), texto con un ejemplo (si existe), y el mismo `asset_id` de audio ya resuelto en el anverso (sin volver a llamar a `resolve_audio`). Corrige de paso el bug encontrado: el audio de pronunciación nunca debe generarse a partir de la traducción en castellano (depende de T106, T108).
- [X] T110 [P] [US4] Test de integración/unitario: una `VocabularyEntry` con `ipa`/`example` produce una tarjeta cuyo reverso incluye ambos como bloques de texto separados; una sin ninguno de los dos genera una tarjeta igualmente completa, sin esos bloques — en `crates/learnkit-cli/tests/cards_test.rs` o `crates/learnkit-cards/src/card.rs`.
- [X] T111 [P] [US4] Test unitario: el texto pasado a `resolve_audio` para el audio de pronunciación es siempre la palabra en inglés (`item.title`), nunca la traducción — regresión del bug encontrado, en `crates/learnkit-cli/src/commands/cards.rs`.
- [X] T112 [P] [US4] Test unitario: el `asset_id` de audio del reverso coincide exactamente con el del anverso en la misma tarjeta (mismo recurso reutilizado, no dos llamadas independientes) — en `crates/learnkit-cli/src/commands/cards.rs`.

---

## Phase 19: Editar una entrada de vocabulario (FR-012d — añadida post-release tras uso real, 2026-09-28)

**Purpose**: `--ipa`/`--example` (Phase 18) solo se pueden fijar al confirmar una entrada nueva; David tiene 127 entradas ya creadas sin esos campos y no hay forma de añadírselos sin borrar y recrear la entrada (perdiendo su `id`/trazabilidad). Ver FR-012d.

- [X] T113 [US3] Añadir `learnkit learn vocabulary edit --lemma <lema> [--sense <s>] [--part-of-speech <p>] [--ipa <ipa>] [--example <e>]...` en `crates/learnkit-cli/src/commands/learn.rs`: busca la entrada por lema (`find_by_lemma`, ya existente), aplica solo los campos indicados (omitir un flag no borra el valor ya guardado), y la persiste con el mismo `id`. Falla explícitamente (mismo patrón que `remove`, exit 40) si el lema no existe. `--example` sin más argumentos no reemplaza los ejemplos existentes; para eso hace falta un flag explícito `--clear-examples` (evita que un `edit` parcial borre datos sin querer).
- [X] T114 [P] [US3] Test de integración: `learn vocabulary edit --lemma <l> --ipa <ipa>` actualiza el IPA de una entrada ya existente sin tocar sus demás campos (sense, examples, sources) ni cambiar su `id`, en `crates/learnkit-cli/tests/vocabulary_test.rs`.
- [X] T115 [P] [US3] Test de integración: `learn vocabulary edit` sobre un lema inexistente falla explícitamente (exit 40), igual que ya hace `remove`, en `crates/learnkit-cli/tests/vocabulary_test.rs`.

---

## Notes

- [P] = ficheros distintos, sin dependencias pendientes entre sí.
- La etiqueta [Story] mapea cada tarea a su historia de usuario para trazabilidad.
- La decisión de research.md §2 (sugerencia de vocabulario delegada al agente vía Skill, no un motor NLP en Rust) ya fue confirmada por el usuario durante `/speckit-plan`; T037 es su implementación.
- El spike de Anki (T054) es un punto de control real: si falla, `research.md` §5 ya documenta el plan B (helper externo aislado) antes de continuar con T055-T057.
- Confirmar cada checkpoint contra el escenario correspondiente de `quickstart.md` antes de avanzar a la siguiente fase.

---

description: "Task list template for feature implementation"
---

# Tasks: Checklist por fase y fases `analyse`/`consolidate` reales

**Input**: Design documents from `/specs/003-analyse-consolidate-checklist/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-commands.md, contracts/agent-skill.md, quickstart.md

**Tests**: se incluyen tareas de test de integración, mismo criterio que las features 001/002. Donde una prueba dependa de contenido redactado por un agente real (resumen/mapa mental/páginas), la tarea usa contenido de fixture fijo pasado directamente a los comandos de confirmación (`analyse summary set --file ...`), sin invocar ningún LLM real — igual que `vocabulary_test.rs` simula la confirmación de `learn vocabulary add` sin invocar ningún agente.

**Organization**: tareas agrupadas por historia de usuario (US1-US4 de `spec.md`), en el mismo orden de prioridad y dependencia real que fija la propia spec.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (ficheros distintos, sin dependencias pendientes)
- **[Story]**: historia de usuario a la que pertenece (US1-US4)
- Cada tarea incluye la ruta de fichero exacta

## Path Conventions

Workspace Rust ya existente, sin crates nuevos (ver `plan.md` → Project Structure):

```text
crates/
├── learnkit-workflow/    (existente, ampliado: engine.rs, phases.rs, analysis.rs NUEVO, consolidate.rs NUEVO)
├── learnkit-cli/         (existente, ampliado: commands/status.rs, commands/analyse.rs NUEVO, commands/consolidate.rs NUEVO)
├── learnkit-profile/     (existente, sin cambios de contrato — solo consumido desde el handler CLI de consolidate)
└── learnkit-agent/       (existente, ampliado con Skill `learnkit-analyse`)
tests/fixtures/            fixtures de material con huecos deliberados, para FR-006
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: fixture de material con un hueco deliberado, necesaria para probar la detección/relleno de huecos (FR-006) sin depender de contenido real de clase.

- [X] T001 [P] Crear `tests/fixtures/analyse-sample/` con un fichero de notas Markdown de ejemplo que mencione un concepto sin explicarlo (para US2/FR-006) y documentar en un `README.md` del propio directorio que la validación con contenido real de clase se hace siguiendo `quickstart.md`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: extender el motor de fases genérico para que soporte un checklist por fase, sin el cual ninguna historia de usuario puede implementarse.

**⚠️ CRITICAL**: no empezar Phase 3+ sin terminar esta fase.

- [X] T002 Añadir `ChecklistItemManifest` (`item_id`, `state: Pending|Done|Blocked|PendingUserDecision`, `input_fingerprint`, `pending_reason`, `resolution`) y el campo `checklist: Option<Vec<ChecklistItemManifest>>` (`#[serde(default)]`, compatible hacia atrás) a `PhaseManifest`, más la variante `PhaseState::NeedsUserInput`, en `crates/learnkit-workflow/src/engine.rs`, según `data-model.md` → PhaseManifest/ChecklistItemManifest y `research.md` §1 (sin dependencias).
- [X] T003 Implementar la derivación del estado agregado de una fase con checklist (`Valid` solo si todos los elementos son `Done`; `NeedsUserInput` si alguno es `PendingUserDecision` y ninguno `Failed`; `Blocked` si la fase de la que depende no es `Valid`) dentro de `recompute_state` en `crates/learnkit-workflow/src/engine.rs` (depende de T002).
- [X] T004 Implementar los helpers de lectura/escritura de un elemento de checklist individual (`write_checklist_item`, `read_checklist`, invalidación cuando su `input_fingerprint` cambia) en `crates/learnkit-workflow/src/engine.rs` (depende de T002).
- [X] T005 [P] Registrar la fase `analyse` con checklist vacío inicial (`requires = ["inventory"]`) en `crates/learnkit-workflow/src/phases.rs`, sustituyendo el nodo opaco actual que solo conecta con `vocabulary` (depende de T002).
- [X] T006 [P] Registrar la fase `consolidate` (`requires = ["analyse"]`, sin checklist propio) en `crates/learnkit-workflow/src/phases.rs` — hoy no existe en el código pese a estar nombrada en `docs/05-cli-spec.md` (depende de T002).

**Checkpoint**: el motor de fases soporta checklist de forma genérica; las historias de usuario pueden implementarse.

---

## Phase 3: User Story 1 - Ver de un vistazo en qué punto está una sesión (Priority: P1) 🎯 MVP

**Goal**: `learnkit status --session <id>` muestra el estado individual de cada elemento de una fase con checklist, no un único estado agregado.

**Independent Test**: Escenario 1 de `quickstart.md` — construyendo el checklist directamente vía los helpers de T004 (sin depender todavía de que `analyse summary set` exista de verdad, que es US2).

### Tests for User Story 1

- [X] T007 [P] [US1] Test de integración: una fase con checklist donde un elemento está `done` y otro `pending` se reportan por separado en `status --json`, no como un único estado de fase (FR-001/002) en `crates/learnkit-cli/tests/status_test.rs`.
- [X] T008 [P] [US1] Test de integración: cuando todos los elementos de una fase están `done`, la fase se reporta como completa y cada elemento aparece `done` (Acceptance Scenario 2 de US1) en `crates/learnkit-cli/tests/status_test.rs`.
- [X] T009 [P] [US1] Test de integración: cambiar la fuente de la que depende un elemento ya `done` lo marca como desactualizado (no `done`) en la siguiente consulta de estado (FR-004, Acceptance Scenario 3 de US1) en `crates/learnkit-cli/tests/status_test.rs`.
- [X] T010 [P] [US1] Test de regresión: una fase sin checklist (`inventory`) sigue reportando un único estado agregado exactamente como antes de esta feature (FR-003) en `crates/learnkit-cli/tests/status_test.rs`.

### Implementation for User Story 1

- [X] T011 [US1] Ampliar `learnkit status --session <id>` (salida humana y `--json`) para incluir, por cada fase con checklist, el array de sus elementos con `item_id`/`state`/`pending_reason`, en `crates/learnkit-cli/src/commands/status.rs` (depende de T004).

**Checkpoint**: User Story 1 completa y comprobable de forma independiente — visibilidad del checklist ya funciona para cualquier fase que lo adopte.

---

## Phase 4: User Story 2 - Obtener resumen, mapa mental y páginas de concepto de una sesión (Priority: P1)

**Goal**: `analyse summary/mindmap/page` producen contenido real, confirmado y trazable, visible a través del checklist de US1.

**Independent Test**: Escenario 2 de `quickstart.md`.

### Tests for User Story 2

- [ ] T012 [P] [US2] Test de integración: `analyse summary set` confirma el elemento `summary`, que pasa a `done` con su fingerprint de fuentes (FR-005) en `crates/learnkit-cli/tests/analyse_test.rs`.
- [ ] T013 [P] [US2] Test de integración: `analyse mindmap set` confirma el elemento `mindmap`; si su contenido es idéntico byte a byte al resumen, el comando falla explícitamente (Acceptance Scenario 2 de US2) en `crates/learnkit-cli/tests/analyse_test.rs`.
- [ ] T014 [P] [US2] Test de integración: `analyse page add` crea un elemento `page-<n>` independiente cada vez que se llama, y pueden confirmarse en cualquier orden entre sí y respecto a `summary`/`mindmap` (Edge Case de spec.md) en `crates/learnkit-cli/tests/analyse_test.rs`.
- [ ] T015 [P] [US2] Test de integración: usando el fixture de T001 (hueco deliberado), `analyse summary set --filled-gap "<concepto>:<nota>"` persiste ese relleno marcado como añadido por el sistema, distinguible del resto del contenido (FR-006, Acceptance Scenario 3 de US2) en `crates/learnkit-cli/tests/analyse_test.rs`.
- [ ] T016 [P] [US2] Test de integración: repetir `analyse summary set` sin cambios en las fuentes es un no-op (no regenera); `--force` sí lo hace (FR-009, Acceptance Scenario 4 de US2) en `crates/learnkit-cli/tests/analyse_test.rs`.

### Implementation for User Story 2

- [ ] T017 [US2] Implementar `ClassSummary`/`StudyMap`/`ConceptPage` (lectura/escritura en `sessions/<id>/analysis/`, ya reservado en `session_paths.rs`) en `crates/learnkit-workflow/src/analysis.rs`, según `data-model.md` (depende de T004).
- [ ] T018 [US2] Implementar el handler CLI `analyse summary set` (validación de fichero no vacío, `--filled-gap`, idempotencia/`--force`) en `crates/learnkit-cli/src/commands/analyse.rs` (nuevo) (depende de T017).
- [ ] T019 [US2] Implementar el handler CLI `analyse mindmap set` (misma validación + comprobación de no-repetición respecto al resumen) en `crates/learnkit-cli/src/commands/analyse.rs` (depende de T017).
- [ ] T020 [US2] Implementar el handler CLI `analyse page add` en `crates/learnkit-cli/src/commands/analyse.rs` (depende de T017).
- [ ] T021 [US2] Crear la Skill de agente genérica `learnkit-analyse` (`SKILL.md`, sin instrucciones específicas de idiomas: lee `session show --json` y `status --json`, redacta resumen/mapa/páginas, confirma vía los comandos de T018-T020, usa `flag-pending` cuando el material no basta) en `crates/learnkit-agent/templates/skills/learnkit-analyse/SKILL.md`, registrada en `crates/learnkit-agent/src/templates.rs` (mismo mecanismo que `learnkit-language`), según `contracts/agent-skill.md` (depende de T018, T019, T020).

**Checkpoint**: User Stories 1 y 2 funcionan de forma independiente — `analyse` produce contenido real y visible.

---

## Phase 5: User Story 3 - Ser consultado cuando falta información para completar un elemento (Priority: P2)

**Goal**: un elemento sin material suficiente se marca explícitamente como pendiente de decisión, con un motivo, y se resuelve sin volver a preguntarse.

**Independent Test**: Escenario 3 de `quickstart.md`.

### Tests for User Story 3

- [ ] T022 [P] [US3] Test de integración: `analyse mindmap flag-pending --reason "..."` marca el elemento como `pending_user_decision` con ese motivo, visible en `status --json`, mientras otros elementos siguen su curso normal (FR-007, Acceptance Scenario 1 de US3) en `crates/learnkit-cli/tests/analyse_test.rs`.
- [ ] T023 [P] [US3] Test de integración: aportar el material que faltaba y volver a llamar a `analyse mindmap set` limpia el estado pendiente sin repetir el trabajo de los demás elementos (FR-008, Acceptance Scenario 2 de US3) en `crates/learnkit-cli/tests/analyse_test.rs`.
- [ ] T024 [P] [US3] Test de integración: `analyse mindmap skip --reason "..."` resuelve el pendiente de forma auditada; una consulta de estado posterior no vuelve a mostrarlo como pendiente de decisión (FR-008, Acceptance Scenario 3 de US3) en `crates/learnkit-cli/tests/analyse_test.rs`.

### Implementation for User Story 3

- [ ] T025 [US3] Implementar el handler CLI `analyse <item> flag-pending --reason <motivo>` (escribe `state = PendingUserDecision`, `pending_reason`) en `crates/learnkit-cli/src/commands/analyse.rs` (depende de T004, T017).
- [ ] T026 [US3] Implementar el handler CLI `analyse <item> skip --reason <motivo>` (escribe `resolution = {kind: "skipped", reason}`, deja de bloquear fases dependientes para ese elemento) en `crates/learnkit-cli/src/commands/analyse.rs` (depende de T025).

**Checkpoint**: User Stories 1-3 funcionan de forma independiente — ningún hueco de material bloquea en seco ni se completa en silencio.

---

## Phase 6: User Story 4 - Consolidar el vocabulario/preguntas candidatas de una sesión sin duplicar lo ya existente (Priority: P3)

**Goal**: `consolidate` produce candidatos de vocabulario deduplicados contra el almacén persistente ya existente.

**Independent Test**: Escenario 4 de `quickstart.md`.

### Tests for User Story 4

- [ ] T027 [P] [US4] Test de integración: `consolidate` produce un listado de `ConsolidatedCandidate` derivado del resumen/páginas de una sesión analizada, cada uno con su trazabilidad de origen (FR-010/012, Acceptance Scenario 1 de US4) en `crates/learnkit-cli/tests/consolidate_test.rs`.
- [ ] T028 [P] [US4] Test de integración: consolidar dos sesiones que comparten un mismo concepto de vocabulario ya persistido marca el segundo candidato con `already_exists: true` y no lo duplica en el listado de candidatos nuevos (FR-011, Acceptance Scenario 2 de US4, SC-005) en `crates/learnkit-cli/tests/consolidate_test.rs`.
- [ ] T029 [P] [US4] Test de integración: `consolidate` sobre una sesión con un elemento de `analyse` todavía `pending_user_decision` bloquea explícitamente (exit `20`, `BLOCKED`) la parte afectada, sin consolidar con datos incompletos (FR-013, Edge Case de spec.md) en `crates/learnkit-cli/tests/consolidate_test.rs`.
- [ ] T030 [P] [US4] Test de integración: `consolidate list` muestra los candidatos ya consolidados de una sesión, con su origen y su estado de duplicado (Acceptance Scenario 3 de US4) en `crates/learnkit-cli/tests/consolidate_test.rs`.

### Implementation for User Story 4

- [ ] T031 [US4] Implementar `ConsolidatedCandidate` (creación, lectura/escritura en `sessions/<id>/consolidated/`, ya reservado en `session_paths.rs`; campo `candidate_type` con único variante `vocabulary` en esta feature) en `crates/learnkit-workflow/src/consolidate.rs`, según `data-model.md` (depende de T004).
- [ ] T032 [US4] Implementar el guard de entrada de `consolidate`: bloquear (exit `20`) la parte de la sesión afectada por cualquier elemento de `analyse` que siga `pending_user_decision`, dejando consolidar con normalidad el resto, en `crates/learnkit-workflow/src/consolidate.rs` (depende de T031, T025).
- [ ] T033 [US4] Implementar el handler CLI `consolidate`: derivar candidatos de vocabulario del `ClassSummary`/`ConceptPage` ya confirmados, comprobar cada uno contra `VocabularyEntry` (dedup por `lemma`, vía la API pública ya existente de `learnkit-profile::language`, **sin** que `learnkit-workflow` dependa de `learnkit-profile` — ver `research.md` §4), y persistir el resultado vía T031, en `crates/learnkit-cli/src/commands/consolidate.rs` (nuevo) (depende de T031, T032).
- [ ] T034 [US4] Implementar el handler CLI `consolidate list` en `crates/learnkit-cli/src/commands/consolidate.rs` (depende de T031).

**Checkpoint**: las 4 historias de usuario funcionan de forma independiente — pipeline `inventory → analyse → consolidate → vocabulary` cerrado para el caso de vocabulario.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: mejoras que afectan a varias historias de usuario.

- [ ] T035 [P] Ejecutar los 4 escenarios + el escenario de bloqueo de `quickstart.md` (con contenido de fixture, sin agente real) y corregir cualquier discrepancia encontrada.
- [ ] T036 Implementar el formateo de salida legible por humanos para los comandos nuevos (`analyse summary/mindmap/page/flag-pending/skip`, `consolidate`, `consolidate list`) en `crates/learnkit-cli/src/commands/analyse.rs` y `crates/learnkit-cli/src/commands/consolidate.rs`.
- [ ] T037 [P] Pasar `cargo fmt --all` y `cargo clippy --workspace --all-targets` sobre los ficheros tocados y corregir cualquier warning.
- [ ] T038 [P] Actualizar `docs/05-cli-spec.md` (comandos `analyse`/`consolidate` ya no son nombres sueltos) y `docs/04-workflow-guards.md` (checklist por fase, estado `NeedsUserInput`) para reflejar el diseño real, siguiendo la misma convención "spec antes que código" ya aplicada al resto del proyecto.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias.
- **Foundational (Phase 2)**: depende de Setup — bloquea todas las historias de usuario.
- **User Stories (Phase 3-6)**: dependen de Foundational. A diferencia de la feature 002 (historias que consumían fuentes/dominios externos independientes), aquí forman una **cadena de valor secuencial dentro del mismo motor de fases**: US2 (contenido real de `analyse`) necesita que el checklist de US1 ya exista para poder marcar sus elementos; US3 (pendiente de decisión) necesita elementos de checklist reales de US2 sobre los que aplicarse; US4 (`consolidate`) necesita que `analyse` produzca contenido real (US2) y respeta sus pendientes (US3). Cada historia sigue siendo *implementable y probable* con sus propios fixtures — US1 puede probarse escribiendo un checklist directamente con los helpers de T004, sin pasar por los comandos de US2 — pero el recorrido de demo end-to-end solo se completa en orden P1(US1)→P1(US2)→P2(US3)→P3(US4).
- **Polish (Phase 7)**: depende de que las historias deseadas estén completas.

### Within Each User Story

- Tests antes de la implementación correspondiente.
- Modelos/entidades (`analysis.rs`/`consolidate.rs`) antes de los handlers CLI.
- Los guards de entrada/salida de una fase se registran en `phases.rs` (Foundational) antes de que su historia añada contenido real.

### Parallel Opportunities

- T001 (fixture) en paralelo con el resto de Setup (no hay más tareas de Setup).
- T005, T006 (Foundational) en paralelo entre sí una vez completado T002; T003, T004 dependen directamente de T002.
- Todos los tests marcados [P] de una misma historia, en paralelo entre sí.
- Distintas personas pueden trabajar en historias distintas en paralelo una vez Foundational está completo, siempre que cada una use sus propios fixtures (checklist construido directamente, o contenido de fixture fijo) en lugar de depender del resultado real de la historia anterior.

---

## Parallel Example: User Story 2

```bash
# Lanzar juntos los tests de la historia 2:
Task: "Test de integración: analyse summary set confirma el elemento en crates/learnkit-cli/tests/analyse_test.rs"
Task: "Test de integración: analyse mindmap set rechaza contenido idéntico al resumen en crates/learnkit-cli/tests/analyse_test.rs"
Task: "Test de integración: analyse page add crea elementos independientes en crates/learnkit-cli/tests/analyse_test.rs"
Task: "Test de integración: --filled-gap queda trazable en crates/learnkit-cli/tests/analyse_test.rs"
Task: "Test de integración: idempotencia de analyse summary set en crates/learnkit-cli/tests/analyse_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Completar Phase 1: Setup.
2. Completar Phase 2: Foundational.
3. Completar Phase 3: User Story 1.
4. **Parar y validar**: Escenario 1 de `quickstart.md` (con un checklist construido directamente vía los helpers de T004).
5. El checklist por fase ya es visible y consultable, incluso sin contenido real de `analyse` todavía.

### Incremental Delivery

1. Setup + Foundational → checklist genérico listo.
2. US1 → checklist visible por sesión → MVP (aunque `analyse` todavía no tenga contenido real).
3. US2 → resumen/mapa mental/páginas reales, redactados por el agente → primer valor de estudio real.
4. US3 → ningún hueco de material bloquea en seco ni se completa en silencio.
5. US4 → `consolidate` cierra el puente hacia el vocabulario ya existente (feature 002), sin duplicar.

### Parallel Team Strategy

Con varias personas, una vez completado Foundational: cada historia puede asignarse a una persona distinta si cada una usa fixtures propios (checklist construido directamente para US1, contenido de fixture fijo para US2-US4) para no bloquearse; la demo end-to-end final sí requiere las 4 completas en orden.

---

## Notes

- [P] = ficheros distintos, sin dependencias pendientes entre sí.
- La etiqueta [Story] mapea cada tarea a su historia de usuario para trazabilidad.
- La decisión de research.md §2 (redacción delegada al agente vía Skill, no un motor de generación en Rust) ya fue confirmada por el usuario durante `/speckit-clarify`; T021 es su implementación.
- La decisión de research.md §4 (`learnkit-workflow` nunca depende de `learnkit-profile`; la orquestación de dedup vive en el handler CLI) es la que fija dónde va cada pieza de T031-T033 — no mover la comprobación de dedup a `learnkit-workflow` sin revisar esa decisión primero.
- Confirmar cada checkpoint contra el escenario correspondiente de `quickstart.md` antes de avanzar a la siguiente fase.

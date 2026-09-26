---

description: "Task list template for feature implementation"
---

# Tasks: Inicialización de proyecto e integración de agentes (`learnkit init`)

**Input**: Design documents from `/specs/001-cli-init-agent-hooks/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-commands.md, quickstart.md

**Tests**: se incluyen tareas de test de integración porque `plan.md`/`research.md` (§8) ya fijan los tests de integración sobre directorios temporales como la estrategia de validación elegida para esta feature — son la única forma de comprobar idempotencia, no-sobrescritura y los exit codes/`--json` end-to-end.

**Organization**: las tareas se agrupan por historia de usuario (US1/US2/US3 de `spec.md`) para permitir implementación y prueba independientes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (ficheros distintos, sin dependencias pendientes)
- **[Story]**: historia de usuario a la que pertenece (US1, US2, US3)
- Cada tarea incluye la ruta de fichero exacta

## Path Conventions

Single project (workspace Rust), según `plan.md` → Project Structure:

```text
Cargo.toml
crates/
├── learnkit-cli/
├── learnkit-core/
├── learnkit-store/
├── learnkit-profile/
└── learnkit-agent/
tests/integration/
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: inicialización del workspace Rust y herramientas base.

- [X] T001 Crear el workspace Rust con `Cargo.toml` raíz y los crates vacíos `crates/learnkit-cli`, `crates/learnkit-core`, `crates/learnkit-store`, `crates/learnkit-profile`, `crates/learnkit-agent` (cada uno con su `Cargo.toml`/`src/lib.rs` o `src/main.rs` mínimo), según `plan.md` → Project Structure.
- [X] T002 Añadir dependencias compartidas al workspace (`clap` con feature `derive`, `serde`+`derive`, `serde_json`, `toml`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`, `sha2`, `is-terminal`, `dialoguer`) y `tempfile` como dev-dependency, en `Cargo.toml` raíz, siguiendo `research.md` §1-6 y §9-10.
- [X] T003 [P] Configurar `rustfmt.toml` y la configuración de `clippy` (lints por workspace) en la raíz del repositorio.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: infraestructura compartida por las tres historias de usuario. Ninguna historia puede implementarse hasta completar esta fase.

**⚠️ CRITICAL**: no empezar Phase 3+ sin terminar esta fase.

- [X] T004 Definir los tipos de error de dominio (`thiserror`) compartidos en `crates/learnkit-core/src/error.rs`, cubriendo al menos: perfil no soportado, agente no soportado, proyecto no inicializado, fallo de filesystem.
- [X] T005 [P] Definir la envolvente de salida `--json` estable (`{"ok","code",...}`) reutilizable por los tres comandos en `crates/learnkit-core/src/output.rs`, según `contracts/cli-commands.md`.
- [X] T006 [P] Implementar la utilidad de escritura atómica (temp file + rename) en `crates/learnkit-store/src/atomic.rs`, según `research.md` §6 y `docs/10-storage-git.md` §5.
- [X] T007 Implementar la lectura/detección de un `Project` existente (presencia y validez de `learnkit.toml` + `.learnkit/`) en `crates/learnkit-store/src/project.rs`, según el modelo `Project` de `data-model.md` (depende de T004).
- [X] T008 [P] Definir el catálogo de perfiles soportados (`generic`, `language`, `geography`, `godot`) en `crates/learnkit-profile/src/catalog.rs`, según `docs/06-profiles.md`.
- [X] T009 [P] Definir el catálogo de agentes soportados (`codex`, `claude`) en `crates/learnkit-agent/src/catalog.rs`, según `docs/09-agent-integration.md`.
- [X] T010 [P] Embeber las plantillas fuente de integración de agentes (`AGENTS.md`, `CLAUDE.md`, `SKILL.md` por Skill V1 sugerido en `docs/09-agent-integration.md` §7) vía `include_str!` en `crates/learnkit-agent/src/templates.rs`, según decisión de `research.md` §4.
- [X] T011 Implementar el cálculo de hash SHA-256 y la derivación de estado de `AgentFile` (`missing`/`up_to_date`/`modified_by_user`/`stale_template`) en `crates/learnkit-agent/src/state.rs`, según el modelo `AgentFile` de `data-model.md` y `research.md` §5 (depende de T010).
- [X] T012 Implementar el motor de instalación de `AgentIntegration` (escribe/actualiza los ficheros de un agente respetando el estado de T011, nunca sobrescribe `modified_by_user`) en `crates/learnkit-agent/src/install.rs` (depende de T006, T010, T011).
- [X] T013 Crear el esqueleto `clap` (`derive`) con los subcomandos `init`, `agent install <agent_id>`, `status` (todavía sin lógica, solo parsing y despacho) en `crates/learnkit-cli/src/main.rs` y `crates/learnkit-cli/src/commands/mod.rs` (depende de T002).
- [X] T013b [P] Implementar la detección de ejecución interactiva vs no interactiva (TTY en stdin+stdout, `--json` siempre no interactivo) en `crates/learnkit-cli/src/interactive.rs`, según `research.md` §9 / FR-013 (depende de T002).
- [X] T013c Implementar el menú interactivo con pasos independientes (perfil con `generic` preseleccionado; agentes, selección múltiple con mínimo uno; shell con la detección automática de T013d preseleccionada) con soporte de cancelación sin efectos secundarios en cualquiera de los tres pasos, en `crates/learnkit-cli/src/interactive.rs`, según `research.md` §10 / FR-003, FR-004, FR-014 (depende de T008, T009, T013b, T013d).
- [X] T013d [P] Definir el catálogo de shells soportados (`sh`, `ps`) y la heurística de detección automática por sistema operativo en `crates/learnkit-core/src/shell.rs`, según `research.md` §11 / FR-015.

**Checkpoint**: infraestructura lista — las historias de usuario pueden implementarse (en paralelo si hay más de una persona).

---

## Phase 3: User Story 1 - Inicializar un proyecto LearnKit nuevo en una carpeta (Priority: P1) 🎯 MVP

**Goal**: `learnkit init [PATH] [--profile <id>] [--agents <ids>] [--json]` crea la estructura mínima de proyecto de forma idempotente, con al menos un agente siempre instalado (interactivamente vía menú, o mediante el fallback no interactivo `generic`+`claude`), sin tocar ficheros ajenos, y rechaza perfiles/agentes no soportados sin dejar estructura parcial.

**Independent Test**: Escenarios 1, 1b, 2, 4 y 6 de `quickstart.md` — inicializar una carpeta vacía sin opciones (no interactivo: fallback `generic`+`claude`), repetir la inicialización (idempotencia), inicializar con perfil+agentes en una sola ejecución, el menú interactivo con cancelación, y rechazar un perfil inexistente sin crear nada.

### Tests for User Story 1

- [X] T014 [P] [US1] Test de integración: `learnkit init` no interactivo (sin TTY, sin `--profile`/`--agents`) sobre carpeta vacía crea `learnkit.toml`, `.learnkit/profiles/generic/`, y la integración del agente por defecto `claude` (FR-003/FR-004 fallback) en `tests/integration/init_test.rs`.
- [X] T015 [P] [US1] Test de integración: ejecutar `learnkit init` dos veces con las mismas opciones (o el mismo fallback) produce el mismo árbol de ficheros (idempotencia, FR-002) en `tests/integration/init_test.rs`.
- [X] T016 [P] [US1] Test de integración: `learnkit init --profile no-existe` termina en exit code 2 y no crea `learnkit.toml` ni `.learnkit/` (FR-011, sin estructura parcial) en `tests/integration/init_test.rs`.
- [X] T017 [P] [US1] Test de integración: `learnkit init` sobre una carpeta con ficheros ajenos (p.ej. `README.md` de otro proyecto) no los modifica (FR-007) en `tests/integration/init_test.rs`.
- [X] T018 [P] [US1] Test de integración: `learnkit init --profile geography --agents codex,claude` crea la estructura de perfil y las integraciones de ambos agentes en la misma ejecución, sin mostrar menú aunque se simule TTY (Escenario 2) en `tests/integration/init_test.rs`.
- [X] T018b [P] [US1] Test de integración: `learnkit init --agents` con lista vacía termina en exit code 2 sin instalar ningún agente ni crear estructura parcial (edge case "mínimo un agente") en `tests/integration/init_test.rs`.
- [X] T018c [P] [US1] Test unitario: simulando una entrada de menú cancelada (Ctrl+C) en el selector de perfil/agente, `interactive.rs` propaga un error de cancelación sin haber invocado ninguna escritura (FR-014) en `crates/learnkit-cli/src/interactive.rs` (test unitario junto al módulo, no de integración, porque no se puede simular un TTY real en CI).
- [X] T018d [P] [US1] Test de integración: `learnkit init` no interactivo guarda en `learnkit.toml` una `shell_preference` detectada automáticamente del sistema operativo (FR-015), visible en `learnkit status --json` en `tests/integration/init_test.rs`.
- [X] T018e [P] [US1] Test de integración: `learnkit init --shell ps` (o `sh`) guarda exactamente el valor pasado, sin mostrar menú para ese paso aunque falten `--profile`/`--agents` en `tests/integration/init_test.rs`.

### Implementation for User Story 1

- [X] T019 [US1] Implementar la resolución/validación de perfil (`Profile`) contra el catálogo de T008 en `crates/learnkit-profile/src/profile.rs` (depende de T008).
- [X] T019b [US1] Añadir `shell_preference` a `ProjectConfig`/`learnkit.toml` en `crates/learnkit-store/src/project.rs` (depende de T004, T013d).
- [X] T020 [US1] Implementar el escritor de scaffold de proyecto (`learnkit.toml` + `.learnkit/workflow.toml` + `.learnkit/profiles/<id>/`) usando escritura atómica, incluyendo `shell_preference`, en `crates/learnkit-store/src/init.rs` (depende de T006, T007, T019, T019b).
- [X] T021 [US1] Implementar el handler de `learnkit init` (parsear `PATH`/`--profile`/`--agents`/`--shell`/`--json`; resolver perfil, agentes y shell de forma independiente cada uno — explícito por flag, si no interactivo mostrar el paso de menú correspondiente, si no aplicar el fallback no interactivo; invocar el scaffold de T020 y el motor de instalación de T012 para el/los agente/s resultantes; construir la envolvente `--json` de T005) en `crates/learnkit-cli/src/commands/init.rs` (depende de T020, T012, T013b, T013c, T013d, T005, T013).
- [X] T022 [US1] Añadir el manejo de error para perfil/agente no soportado, lista de agentes vacía explícita, y cancelación del menú interactivo en `init` (códigos `PROFILE_UNSUPPORTED`/agente no soportado/cancelación, exit code 2, sin ninguna escritura parcial) en `crates/learnkit-cli/src/commands/init.rs` (depende de T021).
- [X] T023 [US1] Añadir logging estructurado (`tracing`) de las operaciones de `init` (modo interactivo/no interactivo, perfil resuelto, agentes instalados, resultado) en `crates/learnkit-cli/src/commands/init.rs`.

**Checkpoint**: User Story 1 completa y comprobable de forma independiente — MVP entregable.

---

## Phase 4: User Story 2 - Añadir integración de un agente a un proyecto ya inicializado (Priority: P2)

**Goal**: `learnkit agent install <agent_id> [--path <PATH>] [--json]` instala/reinstala la integración de un agente sobre un proyecto existente sin repetir `init`, siendo idempotente y protegiendo ficheros editados manualmente.

**Independent Test**: Escenarios 3 y 5 de `quickstart.md` — instalar un agente sobre un proyecto sin agentes, repetir la instalación (idempotencia), e intentar reinstalar sobre un fichero editado manualmente (debe bloquear sin sobrescribir).

### Tests for User Story 2

- [X] T024 [P] [US2] Test de integración: `learnkit agent install claude` sobre un proyecto inicializado sin agentes instala los ficheros esperados en `tests/integration/agent_install_test.rs`.
- [X] T025 [P] [US2] Test de integración: repetir `learnkit agent install claude` sin cambios produce el mismo resultado sin duplicar ficheros (FR-002) en `tests/integration/agent_install_test.rs`.
- [X] T026 [P] [US2] Test de integración: modificar manualmente un fichero de integración instalado y reinstalar el agente termina en `AGENT_FILES_MODIFIED`/exit code 20 sin sobrescribir el contenido editado (FR-008) en `tests/integration/agent_install_test.rs`.
- [X] T027 [P] [US2] Test de integración: `learnkit agent install <agente-inventado>` y `learnkit agent install claude` sobre una carpeta sin proyecto inicializado terminan en exit code 2 sin crear ficheros en `tests/integration/agent_install_test.rs`.

### Implementation for User Story 2

- [X] T028 [US2] Implementar el handler de `learnkit agent install <agent_id>` (validar catálogo de T009, comprobar que `PATH` es un `Project` válido vía T007, invocar el motor de instalación de T012, construir la envolvente `--json`) en `crates/learnkit-cli/src/commands/agent.rs` (depende de T012, T007, T009, T005, T013).
- [X] T029 [US2] Añadir el manejo de las respuestas de error específicas (`AGENT_FILES_MODIFIED` con exit code 20, agente no soportado y proyecto no inicializado con exit code 2) en `crates/learnkit-cli/src/commands/agent.rs` (depende de T028).

**Checkpoint**: User Stories 1 y 2 funcionan de forma independiente.

---

## Phase 5: User Story 3 - Verificar que un proyecto está correctamente inicializado (Priority: P3)

**Goal**: `learnkit status [--path <PATH>] [--json]` recalcula siempre desde el filesystem si el proyecto es válido, su perfil y sus agentes instalados, en un formato apto para humanos y para agentes automatizados.

**Independent Test**: Escenario 1 (primera y última comprobación de `status --json`) y Escenario 3 de `quickstart.md` — comprobar el estado antes de inicializar, después de inicializar, y después de instalar un agente.

### Tests for User Story 3

- [X] T030 [P] [US3] Test de integración: `learnkit status --json` sobre una carpeta sin proyecto devuelve `{"ok": false, "code": "PROJECT_NOT_INITIALIZED", ...}` en `tests/integration/status_test.rs`.
- [X] T031 [P] [US3] Test de integración: `learnkit status --json` sobre un proyecto válido con agentes instalados devuelve `profile_id` e `installed_agents` correctos, recalculados desde el filesystem (no cacheados) en `tests/integration/status_test.rs`.

### Implementation for User Story 3

- [X] T032 [US3] Implementar la construcción de `StatusReport` (validez del proyecto, perfil activo, agentes instalados derivados de `AgentFile`/T011, `shell_preference` leída del `Project`, lista de `issues`) en `crates/learnkit-store/src/status.rs` (depende de T007, T011, T019b).
- [X] T033 [US3] Implementar el handler de `learnkit status` (salida humana y `--json`, exit codes según `contracts/cli-commands.md`) en `crates/learnkit-cli/src/commands/status.rs` (depende de T032, T005, T013).

**Checkpoint**: las tres historias de usuario funcionan de forma independiente.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: mejoras que afectan a las tres historias de usuario.

- [X] T034 [P] Implementar el formateo de salida legible por humanos (no `--json`) compartido para `init`/`agent install`/`status` en `crates/learnkit-cli/src/output/human.rs`.
- [X] T035 Ejecutar manualmente los 7 escenarios (incluyendo 1b, interactivo) de `quickstart.md` contra el binario compilado (`cargo build`) y corregir cualquier discrepancia encontrada.
- [X] T036 [P] Verificar las licencias de las dependencias añadidas en T002 (confirmar ausencia de licencias AGPL, según la restricción de arquitectura de `constitution.md`) y documentar el resultado en `research.md` §7 (nota final).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias — puede empezar de inmediato.
- **Foundational (Phase 2)**: depende de Setup — bloquea todas las historias de usuario.
- **User Stories (Phase 3-5)**: todas dependen de Foundational; pueden avanzar en paralelo o en orden de prioridad (US1 → US2 → US3).
- **Polish (Phase 6)**: depende de que las historias de usuario deseadas estén completas.

### User Story Dependencies

- **User Story 1 (P1)**: puede empezar tras Foundational; no depende de US2/US3.
- **User Story 2 (P2)**: puede empezar tras Foundational; reutiliza el motor de instalación de agentes (T012, Foundational) pero no depende de que US1 se haya implementado primero, aunque en la práctica normalmente se entrega después por prioridad.
- **User Story 3 (P3)**: puede empezar tras Foundational; solo lee estado (T007/T011), no depende de US1/US2 para funcionar, aunque su valor observable aumenta una vez existen proyectos/agentes que inspeccionar.

### Within Each User Story

- Tests antes de la implementación correspondiente.
- Modelos/validaciones antes de los escritores de storage.
- Escritores de storage antes del handler del comando CLI.
- Manejo de errores y logging al final de cada historia.

### Parallel Opportunities

- T003 puede ejecutarse en paralelo con T001/T002.
- T005, T006, T008, T009, T010 (Foundational) pueden ejecutarse en paralelo entre sí; T007, T011, T012, T013 tienen dependencias directas indicadas.
- Todos los tests marcados [P] de una misma historia pueden ejecutarse en paralelo entre sí.
- Una vez completada Foundational, US1/US2/US3 pueden repartirse entre distintas personas en paralelo.

---

## Parallel Example: User Story 1

```bash
# Lanzar juntos todos los tests de la historia 1:
Task: "Test de integración: init crea learnkit.toml + .learnkit/profiles/generic en tests/integration/init_test.rs"
Task: "Test de integración: init es idempotente en tests/integration/init_test.rs"
Task: "Test de integración: init rechaza perfil no soportado sin estructura parcial en tests/integration/init_test.rs"
Task: "Test de integración: init no toca ficheros ajenos en tests/integration/init_test.rs"
Task: "Test de integración: init --profile geography --agents codex,claude en tests/integration/init_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Completar Phase 1: Setup.
2. Completar Phase 2: Foundational (crítico — bloquea todas las historias).
3. Completar Phase 3: User Story 1.
4. **Parar y validar**: ejecutar los escenarios 1, 2, 4 y 6 de `quickstart.md` de forma independiente.
5. `learnkit init` es ya utilizable como MVP (inicialización de proyecto, con o sin agentes en la misma llamada).

### Incremental Delivery

1. Setup + Foundational → base lista.
2. Añadir User Story 1 → validar con `quickstart.md` → MVP.
3. Añadir User Story 2 → validar escenarios 3 y 5 → agentes instalables a posteriori.
4. Añadir User Story 3 → validar escenario de `status` → diagnóstico completo para humanos y agentes automatizados.
5. Cada historia añade valor sin romper las anteriores.

---

## Notes

- [P] = ficheros distintos, sin dependencias pendientes entre sí.
- La etiqueta [Story] mapea cada tarea a su historia de usuario para trazabilidad.
- Cada historia de usuario es completable y comprobable de forma independiente.
- Los tests de integración deben fallar antes de implementar el código que los hace pasar.
- Confirmar cada checkpoint contra los escenarios correspondientes de `quickstart.md` antes de avanzar a la siguiente fase.

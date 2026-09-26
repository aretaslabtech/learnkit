# Implementation Plan: Inicialización de proyecto e integración de agentes (`learnkit init`)

**Branch**: `001-cli-init-agent-hooks` | **Date**: 2026-09-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-cli-init-agent-hooks/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Implementar el comando `learnkit init` (y el subcomando `learnkit agent install <agent>`) como parte del binario CLI en Rust: crean/actualizan de forma idempotente la estructura mínima de un proyecto LearnKit en una carpeta (config `learnkit.toml`, `.learnkit/` con perfil y workflow, y opcionalmente `AGENTS.md`/`CLAUDE.md` + Skills por agente), sin tocar ficheros ajenos al proyecto ni duplicar contenido en reinstalaciones. Se añade además `learnkit status --json` para que humanos y agentes puedan comprobar si una carpeta ya es un proyecto válido y qué agentes tiene instalados, sin analizar texto libre. Todo el enforcement (idempotencia, detección de ficheros modificados, validación de perfil/agente soportado) vive en el CLI Rust, nunca en los ficheros de guía generados para los agentes (constitución, Principios III/IV/V).

## Technical Context

**Language/Version**: Rust (edición estable más reciente disponible en el toolchain del repo; sin fijar versión exacta en la especificación — la fija `Cargo.lock`, per `docs/02-architecture.md §4`).

**Primary Dependencies**: `clap` (parsing de comandos), `serde` + `serde_json` + `toml` (config y salida `--json`), `thiserror` (errores de dominio) + `anyhow` solo en el borde del CLI, `tracing`/`tracing-subscriber` (observabilidad), `sha2` (hashing para detectar ficheros de agente modificados, FR-008), crate de plantillas embebidas (`include_str!`/`rust-embed` o equivalente) para los ficheros `AGENTS.md`/`CLAUDE.md`/`SKILL.md`, `is-terminal` (detección de TTY en stdin/stdout, FR-013) y `dialoguer` (menú de selección de perfil/agente en terminal, FR-003/FR-004/FR-014).

**Storage**: filesystem local únicamente (YAML/TOML/Markdown); no se crea ni se depende de `.learnkit/cache/index.sqlite` en esta feature — esa cache es responsabilidad de `learnkit index rebuild` (fuera de alcance) y su ausencia nunca debe bloquear `init`/`status` (Principio I).

**Testing**: `cargo test` con tests de integración sobre directorios temporales (`tempfile` crate) que invocan el binario/los comandos de la CLI y verifican el árbol de ficheros resultante, la idempotencia (dos ejecuciones ⇒ mismo estado) y los exit codes.

**Target Platform**: binario multiplataforma (Linux, macOS, Windows) ejecutado localmente por el usuario o por un agente (Codex/Claude) desde su propio entorno; sin componente de servidor.

**Project Type**: CLI de proyecto único (single project) dentro de un workspace Rust ya existente/previsto (`docs/02-architecture.md`).

**Performance Goals**: `learnkit init` y `learnkit agent install <agent>` deben completarse en el orden de milisegundos a bajos segundos sobre una carpeta local típica (sin operaciones de red); esto respalda SC-001 ("menos de un minuto con un único comando", que aquí es una cota muy holgada frente al coste real de E/S local).

**Constraints**: operación idempotente (FR-002); nunca sobrescribe/elimina ficheros ajenos a la estructura conocida de LearnKit (FR-007); nunca sobrescribe silenciosamente ficheros de integración de agente detectados como modificados manualmente (FR-008); sin escritura parcial visible si el proceso falla, se cancela a mitad, o el usuario aborta el menú interactivo (FR-014) — implica escritura atómica (temp + rename) siguiendo `docs/10-storage-git.md §5`; el menú interactivo nunca se muestra en modo `--json` ni cuando stdin/stdout no son TTY (FR-013), para no bloquear invocaciones automatizadas; toda ejecución exitosa deja instalado como mínimo un agente (FR-004, SC-006).

**Scale/Scope**: una carpeta de proyecto local por invocación; número de ficheros generados es pequeño (decenas), no miles; no hay concurrencia multiusuario sobre la misma carpeta en el alcance de esta feature.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Evaluación |
|-----------|------------|
| I. Filesystem-First | PASS — `init`/`agent install` solo escriben ficheros de texto (`learnkit.toml`, `.learnkit/*.toml`, `AGENTS.md`/`CLAUDE.md`, `SKILL.md`); no se depende de SQLite. |
| II. Git-Friendly | PASS — todos los artefactos generados son texto versionable (TOML/Markdown), sin binarios. |
| III. Rust Owns Orchestration | PASS — toda la lógica de creación/idempotencia/detección de cambios vive en el CLI Rust; no hay workers Python en esta feature. |
| IV. Hard Guards (NON-NEGOTIABLE) | PASS — `status --json` reporta el estado real re-comprobado por el CLI; ningún fichero de guía de agente puede declarar por sí mismo que el proyecto está inicializado. |
| V. Agent-Agnostic | PASS — el mismo mecanismo de instalación de integración (FR-005/FR-006) se reutiliza para Codex y Claude sin lógica duplicada por agente más allá de sus plantillas. |
| VI. Domain Profiles Extend, Never Contaminate | PASS — la selección de perfil (FR-003) solo determina qué reglas/plantillas de perfil se copian a `.learnkit/profiles/`; el núcleo de `init` no conoce reglas específicas de idiomas/geografía/Godot. |
| VII–XII (Anki/Cards/Assessment/No lock-in/QTI/Transcripción) | N/A — esta feature no crea sesiones, tarjetas, assessments ni transcripciones; no aplica. |
| Architecture constraint: Python solo como worker aislado | PASS — no se introduce Python en esta feature. |
| Architecture constraint: sin AGPL en el binario | PASS — dependencias previstas (`clap`, `serde`, `toml`, `thiserror`, `tracing`, `sha2`) son MIT/Apache-2.0 estándar del ecosistema Rust; se confirmará licencia exacta al fijar versiones en `Cargo.lock`. |

Sin violaciones → no se requiere tabla de Complexity Tracking.

**Re-check post Phase 1 (2026-09-26)**: `data-model.md` y `contracts/cli-commands.md` no introducen SQLite como fuente de verdad, no añaden Python, no filtran reglas de perfil dentro del núcleo, y mantienen el mismo shape `--json` transversal (`{"ok","code",...}`). Constitution Check se mantiene PASS/N/A sin cambios.

## Project Structure

### Documentation (this feature)

```text
specs/001-cli-init-agent-hooks/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

Workspace Rust ya previsto en `docs/02-architecture.md §2`; esta feature implementa el subconjunto necesario para `init`/`agent install`/`status`, sin crear todavía los crates de dominio no relacionados (cards, media, transcripción, assessment, export):

```text
learnkit/                      # workspace root (Cargo.toml)
├── Cargo.toml
├── crates/
│   ├── learnkit-cli/          # parsing de comandos (clap), --json, exit codes; sin lógica de dominio
│   │   └── src/
│   │       ├── main.rs
│   │       ├── interactive.rs      # detección de TTY (FR-013) + menú de selección perfil/agente (FR-003/004/014)
│   │       └── commands/
│   │           ├── init.rs
│   │           ├── agent.rs        # `learnkit agent install <agent>`
│   │           └── status.rs
│   ├── learnkit-core/         # IDs, errores de dominio (thiserror), contratos compartidos
│   ├── learnkit-store/        # creación/lectura idempotente de learnkit.toml, .learnkit/*, escritura atómica
│   ├── learnkit-profile/      # carga/validación de perfiles soportados (genérico + los de docs/06-profiles.md)
│   └── learnkit-agent/        # plantillas embebidas AGENTS.md/CLAUDE.md/SKILL.md, hashing para detectar ediciones manuales (FR-008)
├── profiles/                  # plantillas de perfil fuente (embebidas en build o leídas en dev)
├── skills/                    # plantillas fuente de Skills (learnkit-session, learnkit-process, ... por ahora solo placeholders no usados por init)
└── tests/
    └── integration/
        ├── init_test.rs
        ├── agent_install_test.rs
        └── status_test.rs
```

**Structure Decision**: Single project (workspace Rust) — Opción 1 de la plantilla. Se limita el alcance de esta feature a los crates `learnkit-cli`, `learnkit-core`, `learnkit-store`, `learnkit-profile` y `learnkit-agent`; el resto de crates listados en `docs/02-architecture.md` (`learnkit-workflow`, `learnkit-cards`, `learnkit-media`, `learnkit-transcription`, `learnkit-anki`, `learnkit-assessment`, `learnkit-export`) quedan fuera de esta feature y se añadirán cuando se especifiquen sus features correspondientes.

## Complexity Tracking

*No violations — table intentionally omitted (Constitution Check above is all PASS/N/A).*

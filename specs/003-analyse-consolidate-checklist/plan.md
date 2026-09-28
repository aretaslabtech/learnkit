# Implementation Plan: Checklist por fase y fases `analyse`/`consolidate` reales

**Branch**: `003-analyse-consolidate-checklist` | **Date**: 2026-09-28 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-analyse-consolidate-checklist/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Sustituir el estado único por fase del motor de workflow (`crates/learnkit-workflow/src/engine.rs`) por un checklist de elementos individuales cuando una fase lo necesita, empezando por `analyse`. Implementar `analyse` de verdad (hoy es un nodo vacío que solo conecta `inventory` con `vocabulary`) para que produzca tres elementos por sesión — resumen, mapa mental y páginas de concepto — redactados por un agente vía una Skill genérica nueva (mismo patrón que `learnkit-language` con el vocabulario de la feature 002: el agente lee el material vía CLI, redacta, y confirma con un comando CLI que valida estructura/trazabilidad, nunca la prosa). Cuando el agente no pueda completar un elemento por falta de material, el CLI debe permitir marcarlo explícitamente como pendiente de decisión del usuario en vez de fallar o dar por bueno contenido pobre. Implementar la fase `consolidate` (`requires = ["analyse"]`, hoy inexistente en el código pese a estar nombrada en `docs/05-cli-spec.md`) para producir candidatos de vocabulario deduplicados contra el almacén ya existente (`VocabularyEntry`, feature 002), reutilizando ese mecanismo tal cual. Todo el pipeline de `analyse` (resumen/mapa/páginas, checklist, detección de huecos) debe funcionar igual para sesiones de cualquier asignatura, no solo para el perfil de idiomas (Principio VI); `consolidate` en sí queda acotada a candidatos de vocabulario en esta feature.

## Technical Context

**Language/Version**: Rust (mismo workspace/toolchain de las features 001/002; versión fijada por `Cargo.lock`, no aquí).

**Primary Dependencies**:
- Ya presentes en el workspace: `clap`, `serde`/`serde_json`/`serde_yaml`, `thiserror`/`anyhow`, `sha2` (fingerprints), `tempfile` (test).
- No se añade ninguna dependencia externa nueva de red/generación: la redacción de resumen/mapa mental/páginas la hace un agente vía Skill (confirmado en `/speckit-clarify`), no un provider Rust; el CLI solo valida y persiste texto ya redactado. No hace falta ningún cliente HTTP nuevo ni motor de generación en el binario.

**Storage**: filesystem local, mismo patrón que la feature 002 (YAML/JSON versionables por entidad). Las rutas de sesión para `analysis`/`consolidated` ya están reservadas en `crates/learnkit-store/src/session_paths.rs` (`SessionPaths::analysis()` → `sessions/<id>/analysis`, `SessionPaths::consolidated()` → `sessions/<id>/consolidated`) pero sin ningún escritor/lector todavía — esta feature es la primera en usarlas de verdad.

**Testing**: `cargo test` con tests de integración sobre directorios temporales (mismo patrón que features 001/002), más un *fake* del contenido que "redactaría" el agente (los tests de integración confirman elementos vía CLI con contenido de fixture fijo, igual que `vocabulary_test.rs` simula la confirmación de `learn vocabulary add` sin invocar ningún LLM real).

**Target Platform**: mismo binario CLI multiplataforma ya existente. Esta feature no añade ningún requisito nuevo de red ni de binario externo — es la primera fase del pipeline que no depende de ninguna herramienta de terceros (ni whisper.cpp, ni Piper, ni Wikimedia).

**Project Type**: CLI de proyecto único (mismo workspace, ampliando `learnkit-workflow` y `learnkit-cli`; una Skill nueva en `learnkit-agent`).

**Performance Goals**: sin requisitos de throughput (uso interactivo de una persona); consultar el checklist de una sesión (`status`) y confirmar un elemento vía CLI deben responder en el orden de milisegundos, igual que el resto de comandos de estado hoy.

**Constraints**: hard guards de fase (Principio IV) — ningún elemento de checklist se marca `hecho` porque el agente lo declare; el CLI valida estructura/trazabilidad antes de persistir. Invalidación en cascada: un elemento de checklist ya completo debe pasar a desactualizado si cambia la fuente de la que depende, con el mismo mecanismo de fingerprint que ya usa `PhaseManifest`. El core (`learnkit-workflow`) no puede introducir ningún concepto específico de idiomas (Principio VI): `ClassSummary`/`StudyMap`/`ConceptPage` y la detección de huecos son genéricos; solo `consolidate` (acotado a candidatos de vocabulario en esta feature) toca el perfil de idiomas, y lo hace consumiendo la API pública ya existente de `learnkit-profile::language`, sin que `learnkit-workflow` conozca el concepto de "lema".

**Scale/Scope**: una sesión típica: 1 resumen, 1 mapa mental, 2-5 páginas de concepto, 5-15 candidatos de vocabulario consolidados. Mismo orden de magnitud que la feature 002, sin necesidad de rediseño para volumen.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Evaluación |
|-----------|------------|
| I. Filesystem-First | PASS — `ClassSummary`/`StudyMap`/`ConceptPage`/`ChecklistItem`/`ConsolidatedCandidate` son ficheros de texto versionables (YAML/Markdown) bajo `sessions/<id>/analysis` y `sessions/<id>/consolidated`, ya reservados. |
| II. Git-Friendly | PASS — mismo patrón YAML/JSON/Markdown que el resto del proyecto. |
| III. Rust Owns Orchestration | PASS — el agente (worker externo, no Rust) solo produce el texto; el CLI Rust es quien valida estructura/trazabilidad y decide si un elemento pasa a `hecho`. El agente nunca escribe el estado del checklist directamente (eso es un comando CLI). |
| IV. Hard Guards (NON-NEGOTIABLE) | PASS — un elemento de checklist no se considera válido porque el agente lo declare completo; el comando de confirmación revalida estructura/trazabilidad, y el fingerprint de las fuentes de las que depende se recalcula siempre, igual que hoy hace `recompute_state` para el estado de fase único. |
| V. Agent-Agnostic | PASS — la Skill nueva (`learnkit-analyse`, genérica, sin instrucciones específicas de idiomas) es agnóstica de qué agente (Codex/Claude) la ejecute; toda la lógica de validación vive en el CLI compartido. |
| VI. Domain Profiles Extend, Never Contaminate | PASS — `ClassSummary`/`StudyMap`/`ConceptPage` y la detección de huecos viven en `learnkit-workflow` (core), sin ningún concepto de idiomas; solo `consolidate` toca `learnkit-profile::language` para producir candidatos de vocabulario, y lo hace desde fuera del core (ver Constraints arriba). |
| VII. Anki Is a Target, Not the Database | N/A — esta feature no toca exportación. |
| VIII. Multimedia Is First-Class | N/A — esta feature es puramente textual (resumen/mapa/páginas/candidatos); no introduce ni modifica ningún `Asset`. |
| IX. Assessment Is Distinct From Memorisation | N/A — no se toca `learnkit-assessment`. |
| X. No Platform Lock-In | N/A — no se introduce ningún exporter nuevo. |
| XI. Assessment Interoperability From Day One | N/A — no se toca el Question Bank. |
| XII. Transcription Is a Provider | PASS (sin cambios) — `analyse` consume la transcripción activa ya existente vía `session show --json` (feature 002, T034), sin volver a invocar ningún `TranscriptionProvider`. |
| Arquitectura: Python solo como worker aislado | PASS — no se introduce Python; el agente que redacta es un proceso externo ya existente (Codex/Claude vía Skill), no un worker nuevo del proyecto. |
| Arquitectura: sin AGPL en el binario | PASS — no se añade ninguna dependencia de compilación nueva. |

Sin violaciones → no se requiere tabla de Complexity Tracking.

**Re-check post Phase 1 (2026-09-28)**: `data-model.md` y `contracts/` mantienen `ClassSummary`/`StudyMap`/`ConceptPage`/`ChecklistItem` genéricos dentro de `learnkit-workflow`, y confirman que `ConsolidatedCandidate` en esta feature es un tipo con un único variante implementado (`vocabulary`), consumiendo `learnkit-profile::language` desde fuera del core sin que este último conozca "lema"/idiomas. Constitution Check se mantiene PASS/N/A sin cambios.

## Project Structure

### Documentation (this feature)

```text
specs/003-analyse-consolidate-checklist/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md         # Phase 1 output (/speckit-plan command)
├── contracts/            # Phase 1 output (/speckit-plan command)
└── tasks.md              # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

Se amplía el workspace Rust ya existente. Ningún crate nuevo: el checklist y las fases `analyse`/`consolidate` son responsabilidad del ya existente `learnkit-workflow` (mismo crate que ya posee `Session`, `Source`, el DAG de fases y `PhaseManifest`), la CLI vive en `learnkit-cli`, y la Skill nueva se registra en `learnkit-agent` junto a `learnkit-language`.

```text
learnkit/
├── crates/
│   ├── learnkit-workflow/
│   │   ├── src/engine.rs        # (existente) + soporte de ChecklistItem junto al PhaseState de fase única
│   │   ├── src/phases.rs        # (existente) + registro de `analyse` (con checklist) y `consolidate`
│   │   ├── src/analysis.rs      # NUEVO — ClassSummary, StudyMap, ConceptPage: lectura/escritura,
│   │   │                        #   detección de huecos expuesta al agente, invalidación en cascada
│   │   └── src/consolidate.rs   # NUEVO — ConsolidatedCandidate (variante vocabulary), dedup delegado
│   │                            #   a learnkit-profile::language
│   ├── learnkit-cli/
│   │   └── src/commands/analyse.rs   # NUEVO — handlers `analyse summary set`, `analyse mindmap set`,
│   │                                 #   `analyse page add`, `analyse skip`, `consolidate`; y
│   │                                 #   `status.rs` ampliado para mostrar el checklist por elemento
│   ├── learnkit-profile/        # (existente, sin cambios de contrato) — `consolidate` consume su
│   │                            #   API pública de dedup por lema, ya implementada en la feature 002
│   └── learnkit-agent/
│       └── templates/skills/learnkit-analyse/SKILL.md   # NUEVA Skill genérica (sin instrucciones
│                                                          #   específicas de idiomas): lee
│                                                          #   `session show --json`, redacta resumen/
│                                                          #   mapa/páginas, confirma vía CLI
└── tests/                        # mismo patrón: tests de integración en
                                   #  crates/learnkit-cli/tests/ y crates/learnkit-workflow/tests/
```

**Structure Decision**: Single project (workspace Rust ampliado, sin crates nuevos) — el checklist y las fases `analyse`/`consolidate` se añaden al ya existente `learnkit-workflow`, que es precisamente el crate que ya posee el DAG de fases y `PhaseManifest`; separarlas en un crate propio no aportaría ningún límite real nuevo (a diferencia de la feature 002, donde cada historia introducía un dominio externo distinto — transcripción, tarjetas, media, Anki, assessment — `analyse`/`consolidate` son una extensión directa del propio motor de workflow). La única pieza nueva fuera de `learnkit-workflow`/`learnkit-cli` es la Skill de agente, que sigue exactamente el mecanismo de plantillas ya usado por `learnkit-language`.

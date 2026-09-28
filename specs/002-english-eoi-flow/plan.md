# Implementation Plan: Flujo end-to-end de inglés (clase EOI — vocabulario)

**Branch**: `002-english-eoi-flow` | **Date**: 2026-09-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-english-eoi-flow/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Implementar, dentro del mismo workspace Rust ya existente, el recorrido completo de una clase de inglés: crear una sesión con sus fuentes (notas/audio/imagen) → transcribir el audio mediante un provider sustituible → sugerir y confirmar vocabulario objetivo → generar tarjetas con imagen y audio (suministrados, generados por voz sintética, o buscados en Wikimedia Commons con atribución) → exportar un mazo Anki → generar y completar un examen local por destreza (recognition/production/listening) → consultar el progreso agregado. Todo el enforcement (guards de fase, validación de assets, protección de licencias) vive en Rust; los providers externos (motor de transcripción, motor de voz, búsqueda de imágenes) son adapters sustituibles detrás de traits, nunca fuente de verdad del estado del workflow (constitución, Principios III/IV/XII).

## Technical Context

**Language/Version**: Rust (misma edición/toolchain del workspace ya creado en la feature 001; sin fijar versión exacta aquí — la fija `Cargo.lock`).

**Primary Dependencies**:
- Ya presentes en el workspace: `clap`, `serde`/`serde_json`/`toml`, `thiserror`/`anyhow`, `tracing`/`tracing-subscriber`, `sha2`, `tempfile` (test).
- Nuevas para esta feature: `serde_yaml` (entidades de dominio en YAML per `docs/03-domain-model.md`), `reqwest` (cliente HTTP para el provider de Wikimedia Commons — único uso de red en toda la feature, siempre opcional/best-effort), `rusqlite` (escritura del `collection.anki2` del `.apkg`), `zip` (empaquetado `.apkg`), `walkdir` (recorrido de directorios de sesión).
- Herramientas externas invocadas como subproceso (nunca como dependencia de compilación ni como Python obligatorio — Principio III/ADR-001): el binario `whisper.cpp` (provider de transcripción por defecto) y el binario `piper` (provider de voz por defecto). Ambas son opcionales en tiempo de compilación: su ausencia en el sistema del usuario se reporta como error de provider, no como fallo de compilación del CLI.

**Storage**: filesystem local, ficheros de texto versionables por entidad (YAML para `session.yaml`, Learning Items, Vocabulary Entries, Card Definitions, Assessment/Assessment Items; JSON para `transcript.json` y manifests de fase; JSONL append-only para `attempts.jsonl`), siguiendo el layout de `docs/10-storage-git.md §2`. Sin índice SQLite derivado en esta feature (se reconstruye por escaneo de directorio; añadirlo es una optimización futura, no requerida por ningún criterio de aceptación de esta feature).

**Testing**: `cargo test` con tests de integración sobre directorios temporales (patrón ya establecido en la feature 001), más fixtures de audio/imagen reales pequeños en `tests/fixtures/` para los tests de transcripción/media. La generación real de `.apkg` se valida con un test de integración que verifica la estructura del ZIP/SQLite, y adicionalmente con una comprobación manual de importación en Anki Desktop documentada en `quickstart.md` (no automatizable en CI sin Anki instalado).

**Target Platform**: mismo binario CLI multiplataforma de la feature 001. Los providers de transcripción/voz requieren que el usuario tenga instalado el binario externo correspondiente (`whisper.cpp`, `piper`) en su sistema; el provider de imágenes (Wikimedia Commons) requiere conectividad de red saliente y es el único punto de la feature que no funciona 100% offline.

**Project Type**: CLI de proyecto único (mismo workspace, ampliando los crates existentes con nuevos crates de dominio).

**Performance Goals**: sin requisitos de throughput (uso interactivo de una persona); la generación de un `.apkg` de una clase típica (10-20 tarjetas) y la construcción de un examen deben completarse en segundos en hardware de portátil estándar, sin contar el tiempo de la transcripción de audio en sí (dominado por el motor externo, fuera del control de LearnKit).

**Constraints**: hard guards de fase (ninguna fase se marca válida sin revalidación real, Principio IV); ninguna sobrescritura de contenido corregido a mano (transcripciones importadas, vocabulario confirmado); toda imagen de Wikimedia Commons debe conservar su atribución donde se use (FR-017d); ningún exporter (Anki) debe producir un artefacto parcial o silenciosamente incompleto (FR-019); los providers externos nunca pueden marcar una fase como válida por sí mismos (Principio III/XII).

**Scale/Scope**: una sesión de clase típica: 1-3 fuentes, 1 audio de 30-90 min, 10-30 expresiones de vocabulario candidatas, 10-30 tarjetas, un examen de 15-30 preguntas. No se diseña para volúmenes masivos (cientos de sesiones concurrentes) en esta feature.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Evaluación |
|-----------|------------|
| I. Filesystem-First | PASS — toda entidad nueva (sesión, fuentes, transcript, vocabulario, tarjetas, assessment, attempts) es un fichero de texto versionable; no se introduce ninguna base de datos como fuente de verdad. |
| II. Git-Friendly | PASS — YAML/JSON/JSONL por entidad, coherente con `docs/10-storage-git.md`. |
| III. Rust Owns Orchestration | PASS — whisper.cpp y Piper se invocan como binarios externos detrás de traits Rust (`TranscriptionProvider`, `VoiceProvider`); ningún worker externo escribe estado interno directamente. Wikimedia Commons es un cliente HTTP síncrono desde Rust, no un worker con estado. |
| IV. Hard Guards (NON-NEGOTIABLE) | PASS — el DAG de fases (session→inventory→analyse→learn→vocabulary→cards→anki/assessment) revalida dependencias, fingerprints y assets antes de cada fase dependiente (`docs/04-workflow-guards.md`), coherente con FR-006/007/015/019 de la spec. |
| V. Agent-Agnostic | PASS — la sugerencia de vocabulario (FR-012) se apoya en que el agente (Codex/Claude) lee el material vía comandos CLI existentes y propone candidatos a través de una Skill; el CLI persiste solo lo confirmado. Ningún comportamiento depende de qué agente se use. |
| VI. Domain Profiles Extend, Never Contaminate | PASS — Vocabulary Entry, IPA, variedad `en-GB`, y las plantillas de tarjeta de idiomas viven en el perfil `language-en` (`learnkit-profile`), nunca en `learnkit-core`/`learnkit-workflow`. |
| VII. Anki Is a Target, Not the Database | PASS — Card Definition (`learnkit-cards`) es independiente del formato Anki; `learnkit-anki` es un adapter de exportación separado. |
| VIII. Multimedia Is First-Class | PASS — imágenes/audios son `Asset` independientes y reutilizables (`learnkit-media`), con deduplicación por fingerprint, sea el origen suministrado, generado por voz, o de Wikimedia. |
| IX. Assessment Is Distinct From Memorisation | PASS — `learnkit-assessment` no depende de `learnkit-cards`; ambos derivan de Learning Item de forma independiente. |
| X. No Platform Lock-In | N/A en esta feature — Kahoot/Blooket quedan explícitamente fuera de alcance (spec → Assumptions); no se introduce ningún lock-in nuevo. |
| XI. Assessment Interoperability From Day One | PASS — Assessment Item conserva ID estable, choices con identidad propia, correct response explícita, scoring y feedback, per `docs/03-domain-model.md §7` (FR-022). |
| XII. Transcription Is a Provider | PASS — `TranscriptionProvider` con `whisper.cpp` como default V1 e importadores SRT/VTT/JSON como alternativa; ningún artefacto posterior conoce el formato nativo del engine (FR-004/005/006). |
| Arquitectura: Python solo como worker aislado | PASS — no se introduce Python; whisper.cpp y Piper son binarios nativos invocados como subproceso. |
| Arquitectura: sin AGPL en el binario | PASS — `reqwest`, `rusqlite`, `zip`, `walkdir`, `serde_yaml` son MIT/Apache-2.0 estándar; se reconfirmará con `cargo metadata` en la fase de implementación, como en la feature 001. |

Sin violaciones → no se requiere tabla de Complexity Tracking (ver nota más abajo sobre el número de crates nuevos, que es una expansión de alcance ya prevista por `docs/02-architecture.md`, no una violación de simplicidad).

**Re-check post Phase 1 (2026-09-26)**: `data-model.md` y `contracts/` no introducen ninguna dependencia nueva no listada aquí, no filtran conceptos de idioma dentro de `learnkit-core`/`learnkit-workflow`, y mantienen el mismo patrón de provider-detrás-de-trait para los tres servicios externos (transcripción, voz, imágenes). Constitution Check se mantiene PASS/N/A sin cambios.

## Project Structure

### Documentation (this feature)

```text
specs/002-english-eoi-flow/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

Se amplía el workspace Rust ya existente (`crates/learnkit-cli`, `crates/learnkit-core`, `crates/learnkit-store`, `crates/learnkit-profile`, `crates/learnkit-agent`) con los crates de dominio previstos en `docs/02-architecture.md §2` que esta feature necesita. Cada historia de usuario introduce/amplía un crate concreto:

```text
learnkit/
├── Cargo.toml                      # workspace ampliado con los crates nuevos
├── crates/
│   ├── learnkit-cli/               # (existente) + nuevos comandos: session, ingest, inventory,
│   │                                #   transcribe, learn, cards build, assessment build,
│   │                                #   validate, run, export anki, attempt import, progress
│   ├── learnkit-core/              # (existente, sin cambios de contrato)
│   ├── learnkit-store/             # (existente) + helpers de layout de sesión (sessions/<id>/...)
│   ├── learnkit-profile/           # (existente) + módulo `language` (Vocabulary Entry, IPA,
│   │                                #   plantillas de perfil language-en) — US3
│   ├── learnkit-agent/             # (existente, sin cambios de contrato en esta feature)
│   ├── learnkit-workflow/          # NUEVO — Session, Source, DAG de fases, guards, manifests,
│   │                                #   invalidación descendente — US1
│   ├── learnkit-transcription/     # NUEVO — TranscriptionProvider, adapter whisper.cpp,
│   │                                #   importadores SRT/VTT/TXT/JSON, transcript.json — US2
│   ├── learnkit-cards/             # NUEVO — Card Definition, templates, políticas de media
│   │                                #   required/optional/disabled por lado — US4
│   ├── learnkit-media/             # NUEVO — Asset registry, validación, VoiceProvider (Piper),
│   │                                #   ImageProvider (Wikimedia Commons), deduplicación — US4
│   ├── learnkit-anki/              # NUEVO — exporter .apkg (SQLite + ZIP), GUIDs deterministas — US5
│   └── learnkit-assessment/        # NUEVO — Assessment Item/Assessment, examen HTML local,
│                                    #   attempts.jsonl, vistas de Progress — US6/US7
├── profiles/
│   └── language-en/                # plantillas fuente del perfil de idiomas (cards, dimensions)
└── tests/                          # (mismo patrón: tests de integración dentro de cada crate/
                                     #  crates/learnkit-cli/tests/, más fixtures en tests/fixtures/)
```

**Structure Decision**: Single project (workspace Rust ampliado) — se añaden 6 crates nuevos (`learnkit-workflow`, `learnkit-transcription`, `learnkit-cards`, `learnkit-media`, `learnkit-anki`, `learnkit-assessment`) sobre los 5 ya existentes, cada uno con una responsabilidad única alineada 1:1 con una historia de usuario, siguiendo exactamente los límites de crate ya previstos (no inventados) en `docs/02-architecture.md §2`. `learnkit-export` (Kahoot/Blooket/HTML general) y `learnkit-workflow`'s alcance completo de perfiles no-idioma quedan fuera: el examen HTML de esta feature se implementa dentro de `learnkit-assessment` por ser mínimo y específico de esta feature, no un exporter de plataforma externa.

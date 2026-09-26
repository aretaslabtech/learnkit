# 11 — Roadmap de implementación V1

## Fase 0 — Skeleton

Objetivo: binario Rust instalable y proyecto inicializable.

Entregables:
- Cargo workspace;
- `learnkit init`;
- `learnkit doctor`;
- config discovery;
- IDs y errores;
- fixture project.

Criterio de salida: crear proyecto, reejecutar `init` sin romperlo, `doctor` verde.

## Fase 1 — Store + sessions + guards

- filesystem store;
- session model;
- source inventory;
- manifests/fingerprints;
- workflow DAG;
- `run`, `validate`, `status --json`;
- invalidación descendente.

Esta fase es la base de fiabilidad y debe terminar antes de integrar IA profundamente.

## Fase 2 — Transcripción e ingestión de audio

- `TranscriptionProvider`;
- formato canónico `transcript.json` + renderer `transcript.md`;
- provider `whisper.cpp`;
- importadores de SRT/VTT/TXT/JSON para Buzz/manual;
- fingerprints de engine/model/config;
- invalidación cuando cambia audio o configuración;
- benchmark reproducible sobre 2-3 audios reales de clase.

Criterio de salida: el mismo audio puede procesarse con al menos dos providers/importadores y producir el mismo contrato interno; el workflow no depende del motor elegido.

La adopción de `faster-whisper` como provider adicional se decide por benchmark, no por preferencia previa.

## Fase 3 — Learning Items

- schemas genéricos;
- source traceability;
- knowledge store;
- profile loading;
- generic profile;
- geography fixture.

## Fase 4 — Card Engine + media

- Card Definition;
- templates;
- asset registry;
- image/audio validation;
- front/back audio;
- media policies.

No necesita aún generar media mediante IA: primero soportar assets suministrados.

## Fase 5 — Anki spike y exporter

Gate técnico temprano porque es requisito no negociable.

- `.apkg` mínimo;
- media;
- front/back audio;
- deterministic GUID;
- smoke import en Anki.

**Si este spike falla, no avanzar dando por hecho Anki.** Resolver arquitectura antes.

## Fase 6 — Assessment

- Assessment Item;
- question bank;
- assessment definitions;
- HTML exam;
- attempts JSONL;
- progress views;
- weak-area selection simple.

## Fase 7 — Game exporters

- Kahoot XLSX;
- Blooket CSV;
- exporter constraints;
- media manifests.

## Fase 8 — Language profile

- Vocabulary Entry;
- British English settings;
- IPA field;
- templates de recognition/production/listening;
- TTS provider adapter;
- image provider adapter;
- deduplicación básica de vocabulary entries.

## Fase 9 — Agent adapters

- Codex Skills;
- AGENTS.md;
- Claude Skills/CLAUDE.md;
- agent evals;
- repair loop basado en validación.

Se puede prototipar antes, pero no debe reemplazar el core determinista.

## Fase 10 — End-to-end release gate

Ejecutar escenarios de `12-test-scenarios.md` desde repositorios limpios.

## Qué NO entra antes de V1

- vídeo generado;
- LMS;
- QTI/TAO runtime;
- cloud sync propio;
- plugin ABI dinámico;
- dashboard web grande;
- algoritmo adaptativo sofisticado.

## Handoff preparado para V2

V1 se considera preparada para V2 cuando el modelo de Assessment pueda mapearse sin pérdida estructural básica a:
- item identity;
- choices/responses;
- correct responses;
- scoring;
- feedback;
- media;
- metadata.

No se exige generar QTI en V1. La primera fase de V2 será un spike contra TAO Community Edition con QTI 2.2. Ver `15-v2-tao-qti.md`.

# LearnKit — Implementation Specification

Status: **V1 design baseline**  
Language of documentation: Spanish  
Primary implementation language: **Rust**

LearnKit es un framework local-first para convertir materiales de estudio en artefactos de aprendizaje verificables: apuntes consolidados, learning items, tarjetas multimedia, bancos de preguntas, exámenes, juegos y seguimiento de progreso.

La idea central es simple:

```text
sources -> session -> learning items -> cards / assessments -> validation -> export -> practice
```

El sistema debe servir tanto para idiomas como para materias no lingüísticas (geografía, historia, Godot, etc.) sin forzar conceptos específicos de un dominio dentro del core.

## Principios no negociables

1. **Filesystem-first**: los ficheros del proyecto son la fuente de verdad.
2. **Git-friendly**: los artefactos canónicos deben poder versionarse y revisarse.
3. **Rust owns orchestration**: el CLI, workflow, guards, validación, modelos y exporters viven en Rust.
4. **Hard guards**: ninguna fase se considera válida porque un agente lo declare; los validadores se ejecutan de nuevo antes de permitir transiciones dependientes.
5. **Agent-agnostic**: Codex, Claude u otros agentes consumen el mismo workflow y los mismos artefactos.
6. **Domain profiles**: idiomas, geografía, historia o Godot añaden reglas sin contaminar el core.
7. **Anki is a target, not the database**: LearnKit mantiene sus propias entidades y exporta a Anki.
8. **Multimedia first-class**: imagen, audio y, en el futuro, vídeo son assets independientes y reutilizables.
9. **Assessment is distinct from memorisation**: Anki y exámenes/juegos comparten conocimiento, pero miden cosas distintas.
10. **No platform lock-in**: Kahoot, Blooket, Anki o Drive son exporters/adapters sustituibles.
11. **Assessment interoperability from day one**: el modelo V1 debe ser QTI-friendly aunque QTI/TAO se implementen en V2.
12. **Transcription is a provider**: LearnKit conserva un contrato propio de transcripción y puede usar `whisper.cpp`, `faster-whisper`, Buzz/manual u otros motores sin acoplar el dominio al motor.

## Documentos

- `docs/01-product-spec.md`: alcance, casos de uso y no-objetivos.
- `docs/02-architecture.md`: arquitectura Rust, workspace y componentes.
- `docs/03-domain-model.md`: entidades y relaciones.
- `docs/04-workflow-guards.md`: fases, máquina de estados y hard guards.
- `docs/05-cli-spec.md`: contrato del CLI Rust.
- `docs/06-profiles.md`: perfiles genérico, idiomas, geografía y Godot.
- `docs/07-anki-media.md`: tarjetas, imágenes, audio frontal/reverso y `.apkg`.
- `docs/08-assessment-games-progress.md`: exámenes, Kahoot, Blooket y progreso.
- `docs/09-agent-integration.md`: integración con Codex/Claude mediante Skills e instrucciones ligeras.
- `docs/10-storage-git.md`: layout del filesystem, índices y política Git.
- `docs/11-v1-roadmap.md`: orden de implementación y criterios de aceptación.
- `docs/12-test-scenarios.md`: escenarios end-to-end para geografía, inglés y Godot.
- `docs/13-decisions.md`: ADRs iniciales y decisiones diferidas.
- `docs/14-transcription.md`: contrato de transcripción, providers, Buzz y benchmark de calidad.
- `docs/15-v2-tao-qti.md`: V2 de evaluación formal con TAO Community Edition y QTI.

## Resultado esperado de V1

V1 debe poder demostrar, como mínimo, estos dos recorridos:

### Geografía

```text
material sobre océanos/continentes
  -> learning items
  -> tarjetas con imagen
  -> .apkg
  -> Kahoot XLSX
  -> Blooket CSV
  -> examen local
  -> resultados/progreso
```

### Inglés

```text
notas + audio de clase
  -> inventario/transcripción
  -> vocabulary learning items
  -> tarjetas con imagen
  -> audio en frontal y/o reverso
  -> .apkg
  -> assessment
  -> progreso por recognition / production / listening
```

Si ambos funcionan sin añadir excepciones al core, la arquitectura está cumpliendo su objetivo.

## Dirección V2 ya acordada

V2 incorporará **TAO Community Edition** como runtime self-hosted para exámenes formales. LearnKit seguirá siendo dueño del Question Bank, Assessment model, trazabilidad y progreso. La interoperabilidad se realizará mediante QTI, con **QTI 2.2 como baseline inicial** y una vía de migración a QTI 3 cuando el soporte estable de TAO CE lo justifique. TCExam queda explícitamente fuera del roadmap.

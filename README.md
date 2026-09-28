# LearnKit

LearnKit es un framework local-first para convertir material de estudio (notas, audio de clase, imágenes) en artefactos de aprendizaje verificables: vocabulario, tarjetas de repaso (Anki), exámenes y seguimiento de progreso.

```text
sources -> session -> learning items -> cards / assessments -> validation -> export -> practice
```

La especificación de diseño completa vive en [`learnkit-implementation-spec/`](learnkit-implementation-spec/README.md). Este repositorio contiene la implementación real en Rust, construida siguiendo un flujo Spec Kit (`specs/`).

## Estado actual

| Feature | Estado |
|---------|--------|
| `001-cli-init-agent-hooks` — `learnkit init`, integración de agentes (Codex/Claude) | ✅ Implementada y verificada |
| `002-english-eoi-flow` — sesión → transcripción → vocabulario → tarjetas → Anki → evaluación → progreso | ✅ Implementada y verificada (incluye importación real en Anki Desktop) |

Ver `specs/<feature>/spec.md` y `specs/<feature>/tasks.md` para el detalle de cada una.

## Empezar

```bash
cargo install --path crates/learnkit-cli
```

Instala el binario `learnkit` en `~/.cargo/bin` (ya en el `PATH` si usas `rustup`), para poder llamarlo como `learnkit` desde cualquier carpeta. Si solo quieres compilarlo sin instalarlo, `cargo build --release` deja el binario en `target/release/learnkit` (`.exe` en Windows).

**Guía de uso completa**: [`docs/manual.md`](docs/manual.md) (también disponible en [`docs/manual.html`](docs/manual.html) para abrir en el navegador).

## Estructura del workspace

```text
crates/
├── learnkit-cli            # binario `learnkit`, comandos
├── learnkit-core           # errores, salida --json, utilidades compartidas
├── learnkit-store          # proyecto, layout de sesión
├── learnkit-profile        # perfiles (genérico, language-en con Vocabulary Entry)
├── learnkit-agent          # plantillas de integración de agentes (AGENTS.md/CLAUDE.md/Skills)
├── learnkit-workflow       # sesión, inventario de fuentes, motor de fases/guards
├── learnkit-transcription  # TranscriptionProvider (whisper.cpp), import SRT/VTT/TXT/JSON
├── learnkit-cards          # Card Definition, plantillas, completeness
├── learnkit-media          # Asset registry, VoiceProvider (REST TTS por defecto, Piper como alternativa offline), ImageProvider (Wikimedia Commons)
├── learnkit-anki           # exporter .apkg (SQLite + ZIP)
└── learnkit-assessment     # banco de preguntas, examen HTML, attempts, progreso
```

## Requisitos previos según lo que quieras usar

- **Siempre**: toolchain de Rust para compilar.
- **Transcripción generada** (`learnkit transcribe`): binario `whisper-cli` de [whisper.cpp](https://github.com/ggerganov/whisper.cpp) + un modelo GGML. Sin él, puedes importar transcripciones ya hechas (`.srt`/`.vtt`/`.txt`) con `learnkit transcribe import`.
- **Audio de pronunciación generado** (`cards build`): por defecto se usa un servicio REST de TTS compatible con `POST /v1/audio/speech` (estilo OpenAI; referencia: [`travisvn/openai-edge-tts`](https://github.com/travisvn/openai-edge-tts)). Requiere la variable de entorno **`TTS_API_KEY`** con la credencial del servicio; sin ella, esas tarjetas quedan sin audio si tampoco suministras uno propio (no bloquea el resto). Como alternativa 100% offline y sin credencial, sigue disponible `PiperVoiceProvider` (binario `piper` de [Piper TTS](https://github.com/rhasspy/piper) + una voz `en-GB`) para quien prefiera ese flujo.
- **Imágenes automáticas** (`cards build`): conectividad de red hacia `commons.wikimedia.org`.

## Gobernanza y contribución

Ver [`.specify/memory/constitution.md`](.specify/memory/constitution.md) para los principios no negociables del proyecto antes de proponer cambios de arquitectura.

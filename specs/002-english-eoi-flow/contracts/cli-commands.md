# Contract: comandos CLI de esta feature

**Feature**: [spec.md](./spec.md) | **Data model**: [data-model.md](./data-model.md)

Cubre los comandos que esta feature introduce o de los que depende, tomados de `learnkit-implementation-spec/docs/05-cli-spec.md`. Todos siguen el mismo contrato transversal `{"ok", "code", ...}` en `--json` ya establecido por la feature 001 (`contracts/cli-commands.md` de `specs/001-cli-init-agent-hooks/`).

## `learnkit session new "<título>" [--profile <id>] [--json]`

Crea una `Session` (US1). `--profile` por defecto usa el perfil activo del proyecto (feature 001); para esta feature se espera `language-en`. Devuelve `session_id` y la ruta de la sesión. **Efecto lateral (FR-003b)**: marca automáticamente la sesión recién creada como sesión activa del proyecto.

## `learnkit session list [--json]` / `learnkit session show [<id>] [--json]`

Lista sesiones o muestra el detalle de una (fuentes inventariadas, transcripciones, vocabulario, tarjetas asociadas). Si se omite `<id>`, usa la sesión activa (FR-003b). `session show --json` es lo que la Skill de agente usa para leer notas/transcript y proponer candidatos de vocabulario (research.md §2) — el JSON incluye el texto de las notas y los segmentos de transcript activa, no solo metadatos.

## `learnkit session use <id>`

Marca `<id>` como sesión activa del proyecto (FR-003b), sin crear ni modificar ninguna sesión. Falla si `<id>` no existe.

## `learnkit ingest <path...> [--session <id>]` / `learnkit inventory [--session <id>]`

Copia/referencia fuentes dentro de la sesión y las inventaría (US1, FR-002/003). `inventory` es idempotente y re-detecta cambios de huella (FR-003). `--session` es opcional en todos los comandos de esta feature que operan sobre una sesión (FR-003b): si se omite, se usa la sesión activa; si no hay ninguna sesión activa y tampoco se indica `--session`, el comando falla explícitamente (exit code `2`) pidiendo una de las dos.

## `learnkit transcribe --session <id> [--source <id>] [--provider whisper-cpp]` / `learnkit transcribe import <file> --source <id>`

Genera o importa una `Transcript` (US2, FR-004/005). Salida `--json` incluye `transcript_id`, `engine`, número de segmentos. Fallos por audio ausente/no soportado devuelven código `TRANSCRIPTION_FAILED`, exit `30` (provider externo falló), coherente con `docs/05-cli-spec.md §14`.

## `learnkit learn vocabulary add --session <id> --lemma "<texto>" --sense "<significado>" --source <source_id> [--locator <punto>] [--suggested-by agent|manual]`

Persiste una `VocabularyEntry` ya confirmada por el usuario (US3, FR-008/009/012b). Si `--lemma` coincide con una entrada existente (deduplicación FR-009), la reutiliza y la vincula a la nueva sesión/fuente en lugar de duplicar. Crea automáticamente el `LearningItem` vinculado (FR-010).

## `learnkit cards build --session <id> [--template <id>]`

Genera `CardDefinition`s a partir de los `LearningItem`s de vocabulario de la sesión (US4, FR-013/014). Para cada lado con recurso faltante, intenta primero un recurso suministrado por el usuario ya registrado; si no existe, invoca `VoiceProvider` (audio, FR-017b) o `ImageProvider` (imagen, FR-017c) según el tipo de bloque requerido por la plantilla.

## `learnkit cards validate --session <id> [--json]`

Guard de salida (US4, FR-015): reporta qué tarjetas están `complete` y cuáles `pending_image`/`pending_audio` con el motivo exacto.

## `learnkit run <phase>` / `learnkit run --until <phase>` / `learnkit validate --session <id>`

Motor de guards genérico (US1, `docs/04-workflow-guards.md`): ejecuta/valida fases en orden topológico, se detiene en el primer guard fallido con salida `BLOCKED: <phase>` y exit code `20`.

## `learnkit export anki --session <id> --out <file.apkg> [--json]`

Exporta el mazo (US5, FR-018/019/020). Falla con exit `40` (constraint de exporter) y detalle exacto de tarjeta/recurso si alguna tarjeta incluida no está `complete`; nunca escribe un `.apkg` parcial (escritura atómica: genera en temporal y renombra solo si toda la validación pasa).

## `learnkit assessment build --session <id> [--json]`

Genera el banco de `AssessmentItem`s y el `Assessment` local cubriendo recognition/production/listening (US6, FR-021/022).

## `learnkit export exam --assessment <id> --format html --out <file.html>`

Genera el examen HTML autocontenido (US6, FR-023). El HTML incluye un botón que descarga un `results.json` con las respuestas dadas, en el formato que consume `learnkit attempt import`.

## `learnkit attempt import <results.json> --assessment <id>`

Ingiere los resultados de un examen completado como eventos `Attempt` en `attempts.jsonl` (US6, FR-024). Operación append-only: nunca sobrescribe intentos previos, incluso si el fichero de resultados se reimporta.

## `learnkit progress [--session <id>] [--tag <t>] [--skill <s>] [--json]`

Vista de progreso agregada bajo demanda (US7, FR-025/026); nunca lee un valor cacheado — siempre recalcula desde `attempts.jsonl` en el momento de la consulta.

---

**Exit codes reutilizados de la feature 001** (`docs/05-cli-spec.md §14`): `0` éxito; `2` uso/config inválida; `10` validación fallida; `20` guard bloqueado; `30` provider/herramienta externa falló (whisper.cpp, Piper, o la búsqueda en Wikimedia Commons no disponible); `40` constraint de exporter; `50` fallo de filesystem.

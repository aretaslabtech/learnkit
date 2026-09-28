# Contract: comandos CLI de esta feature

**Feature**: [spec.md](./spec.md) | **Data model**: [data-model.md](./data-model.md)

Sigue el mismo contrato transversal `{"ok", "code", ...}` en `--json` ya establecido por las features 001/002. Todos los comandos de sesión son opcionales en `--session` (caen a la sesión activa, feature 002 FR-003b).

## `learnkit status --session <id> [--json]` (ampliado)

Además del estado agregado por fase (feature 002), para una fase con checklist (`analyse`) muestra el estado individual de cada elemento (US1, FR-002): `item_id`, `state` (`pending`/`done`/`blocked`/`pending_user_decision`), y `pending_reason` cuando aplica. En JSON, cada fase con checklist incluye un array `checklist` junto a su `state` agregado; una fase sin checklist no incluye ese campo (compatibilidad con clientes que ya parsean el JSON de la feature 002).

## `learnkit analyse summary set --session <id> --file <resumen.md> [--filled-gap <concepto>:<nota>]... [--json]`

Confirma el elemento `summary` de `analyse` (US2, FR-005). Valida que el fichero exista y no esté vacío; cada `--filled-gap` declara un hueco rellenado (FR-006), quedando marcado como añadido por el sistema en el `ClassSummary` persistido, nunca mezclado sin distinción con el contenido derivado directamente del material. Idempotente (FR-009): si el elemento ya está `done` con el mismo `input_fingerprint` de fuentes, es un no-op salvo `--force`.

## `learnkit analyse mindmap set --session <id> --file <mapa.md> [--json]`

Confirma el elemento `mindmap` de `analyse` (US2, FR-005). Misma validación/idempotencia que `summary set`.

## `learnkit analyse page add --session <id> --concept "<texto>" --file <pagina.md> [--json]`

Añade y confirma una `ConceptPage` (US2, FR-005) — el elemento `page-<n>` correspondiente. Puede llamarse varias veces por sesión; cada llamada crea un elemento de checklist independiente.

## `learnkit analyse <summary|mindmap|page-<n>> flag-pending --session <id> --reason "<motivo>" [--json]`

Marca un elemento como `pending_user_decision` (US3, FR-007) en vez de confirmarlo, con el motivo explicado. No falla el comando de `run analyse`/`validate` en seco — el elemento queda visible en `status` con su motivo, y bloquea únicamente la parte de `consolidate` que dependería de él (FR-013), no el resto del checklist.

## `learnkit analyse <summary|mindmap|page-<n>> skip --session <id> --reason "<motivo>" [--json]`

Resuelve un elemento `pending_user_decision` omitiéndolo explícitamente (US3, FR-008), de forma auditada (mismo patrón que el override `--accept semantic-check-<id> --reason` de `docs/04-workflow-guards.md §10`). Deja de bloquear `consolidate` para ese elemento; no vuelve a preguntarse en ejecuciones posteriores mientras el material del que dependía no cambie.

## `learnkit analyse` (registro de fase, sin subcomando propio de "ejecutar")

`analyse` no tiene un paso "todo o nada" como las fases anteriores — su checklist se va completando elemento a elemento vía los comandos de arriba (normalmente invocados por el agente a través de la Skill `learnkit-analyse`). `learnkit run analyse`/`learnkit validate --session <id>` (motor de guards genérico, feature 002) simplemente recalculan y reportan el estado agregado de la fase a partir de su checklist — nunca marcan un elemento como hecho por sí mismos.

## `learnkit consolidate --session <id> [--json]`

Ejecuta la fase `consolidate` (`requires = ["analyse"]`, US4, FR-010/011/012/013): deriva candidatos de vocabulario del resumen/páginas ya confirmados, comprueba cada uno contra `VocabularyEntry` ya persistida (dedup por `lemma`, reutilizando tal cual el mecanismo de la feature 002 — la comprobación la hace este handler CLI, no `learnkit-workflow`, ver `research.md` §4) y persiste el listado de `ConsolidatedCandidate`. Si algún elemento de `analyse` sigue `pending_user_decision`, la parte de la consolidación que dependería de él queda explícitamente bloqueada (`code: "BLOCKED"`, exit `20`) en vez de consolidar con datos incompletos; el resto de la sesión que no depende de ese elemento se consolida con normalidad.

## `learnkit consolidate list --session <id> [--json]`

Lista los `ConsolidatedCandidate` de una sesión ya consolidada, incluyendo `already_exists`/`existing_vocabulary_id` para los que ya tenían una `VocabularyEntry` equivalente (SC-005).

---

**Exit codes reutilizados de las features 001/002** (`docs/05-cli-spec.md §14`): `0` éxito; `2` uso/config inválida; `10` validación fallida; `20` guard bloqueado (incluye un elemento de checklist `pending_user_decision` sin resolver que bloquea `consolidate`); `50` fallo de filesystem. Esta feature no introduce ningún nuevo código de provider externo (`30`) ni de exporter (`40`) — no llama a ninguna herramienta de terceros (ver `plan.md` → Technical Context).

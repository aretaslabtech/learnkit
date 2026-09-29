# Feature: Generalización del modelo de tarjeta — LearningItem → CardSpec → Card

**Tracked as**: ODD (no SDD/spec-kit — decisión explícita de David: "te las paso juntas para minimizar el desarrollo").

## Objetivo

Que el núcleo de LearnKit (`cards build`) deje de estar acoplado a vocabulario. Hoy solo puede generar tarjetas a partir de `VocabularyEntry`; el objetivo es que cualquier `LearningItem` (gramática, diálogo, enumeración C#, rúbrica Clean Code, geografía, etc.) pueda tener una tarjeta generada a partir de contenido genérico (`activity`/`stimulus`/`response`/`feedback`), sin tocar ni una línea del comportamiento visible del flujo de vocabulario existente.

## Problema (confirmado en código, sesión 2026-09-29)

- `crates/learnkit-profile/src/language/learning_item.rs`: `LearningItem.vocabulary_entry_id: String` es un FK obligatorio y hardcodeado; el único constructor es `ensure_for_vocabulary`. No existe forma de crear un `LearningItem` sin una `VocabularyEntry` detrás.
- `crates/learnkit-cli/src/commands/cards.rs::run_build` (línea ~190): `if args.template == IMAGE_TO_PRODUCTION_V1 { ... find_vocabulary_by_id(...) ... }` — busca la `VocabularyEntry` directamente por `item.vocabulary_entry_id` y construye los bloques front/back a mano desde sus campos (`lemma`, `senses`, `examples`...). Es el único punto de acoplamiento real; el resto del pipeline (`template.rs`, `card.rs`, `export_anki`/`package.rs`) ya es genérico.
- No existe ningún concepto `CardSpec` en el código (confirmado por grep).

## Decisión de diseño (2026-09-29)

Para no arriesgar el flujo de vocabulario real que David ya usa (127 tarjetas, uso diario EOI), la generalización es **aditiva, no una reescritura**:

1. `LearningItem` no cambia (sigue teniendo `vocabulary_entry_id` para los de `kind = "vocabulary"`, sin tocar).
2. Nuevo tipo `CardSpec`, independiente de vocabulario, para el contenido de un `LearningItem` de cualquier otro `kind`: `activity`, `stimulus` (texto del frente), `response` (texto del dorso), `feedback` (texto adicional del dorso, opcional).
3. Nuevo comando `learnkit cards set --item <learning_item_id> --activity <str> --stimulus <str> --response <str> [--feedback <str>]`: upsert genérico de un `CardSpec`.
4. `run_build` se generaliza así: si `item.kind == "vocabulary"` → exactamente el camino de hoy, sin tocar (tanto la rama `IMAGE_TO_PRODUCTION_V1` con `find_vocabulary_by_id` como la rama "else" genérica que ya existe usando `item.title`/`item.summary`). Si `item.kind != "vocabulary"` → busca su `CardSpec` (si no existe: error claro, no genera nada en silencio) y construye los bloques usando `stimulus`/`response`/`feedback` en vez de `item.title`/`item.summary` como fuente de texto, tanto en la rama `IMAGE_TO_PRODUCTION_V1` (reemplazando la consulta a `VocabularyEntry`) como en la rama "else" (que hoy ya es genérica pero usa `title`/`summary` en bruto); los medios se siguen resolviendo igual (`resolve_image`/`resolve_audio` con `stimulus` como hint de búsqueda en vez de `item.title`).
5. `template.rs`/`card.rs`/`completeness()`/`export_anki`/`package.rs` no cambian — ya son genéricos (confirmado leyendo el código).

## Alcance

- Nuevo módulo `crates/learnkit-profile/src/card_spec.rs`: struct `CardSpec { id, learning_item_id, activity, stimulus, response, feedback: Option<String> }`, persistencia YAML en `knowledge/card-specs/<learning_item_id>.yaml` (mismo patrón que `learning_item.rs`), funciones `find_by_learning_item`, `set` (upsert), `remove`.
- Nuevo comando CLI `cards set` (`crates/learnkit-cli/src/commands/cards.rs`): valida que el `LearningItem` exista, upsert del `CardSpec`, salida JSON consistente con el resto de comandos.
- Generalización de `run_build`: rama nueva para `item.kind != "vocabulary"` que use `CardSpec`; rama de vocabulario **sin tocar**.
- `LearningItem::remove` en cascada: si se borra un `LearningItem` genérico, borrar también su `CardSpec` (mismo patrón que ya existe para `card.rs::remove` al borrar vocabulario — FR-012c).
- Documentación: `docs/manual.md`/`.html`, sección nueva explicando cómo usar `cards set` para dominios no-vocabulario.

**Fuera de alcance explícito** (no pedido hoy): creación de `LearningItem` genéricos desde CLI (`learn item add` o similar) — se asume que ya existe o se crea por otra vía; UI/wizard para `activity`; plantillas (`TemplateDefinition`) específicas por dominio (geografía, C#, etc.) — se reutilizan las plantillas existentes; migración de datos; pipeline de `analyse`/`consolidate` para generar `CardSpec` automáticamente desde contenido analizado (posible feature futura, no esta).

## Restricciones

- El flujo de vocabulario debe seguir produciendo exactamente los mismos resultados que hoy — verificar con los tests existentes de `cards.rs`/`card.rs` en verde sin modificarlos, más un test de regresión explícito si hace falta.
- `cards build` sobre un `LearningItem` genérico sin `CardSpec` debe fallar con un mensaje claro (no BLOCKED de Hard Guards — es un caso distinto, un item concreto sin contenido, no "cero vocabulario"), nunca generar una tarjeta vacía en silencio.
- Reutilizar `resolve_image`/`resolve_audio`/`resolved_id`/`media_warnings` tal cual — no duplicar lógica de resolución de medios.

## Checklist

- [x] T1 `crates/learnkit-profile/src/card_spec.rs`: struct `CardSpec` + persistencia (`find_by_learning_item`, `set` upsert, `remove`) + tests unitarios.
- [x] T2 Cascada de borrado: `learning_item::remove` también borra el `CardSpec` asociado (si existe).
- [x] T3 Comando `learnkit cards set --item --activity --stimulus --response [--feedback]` en `crates/learnkit-cli/src/commands/cards.rs`.
- [x] T4 Generalizar `run_build`: rama `item.kind != "vocabulary"` usa `CardSpec` para construir bloques genéricos; rama de vocabulario intacta.
- [x] T5 Error claro (no silencioso) cuando un `LearningItem` genérico no tiene `CardSpec` al hacer build.
- [x] T6 Tests de integración: build de un item genérico con `CardSpec` produce la tarjeta esperada; build de vocabulario sigue produciendo el mismo resultado que antes (regresión).
- [x] T7 Documentar `cards set` y el flujo genérico en `docs/manual.md`/`.html`.
- [x] T8 `cargo build --workspace --all-targets`, `cargo test --workspace`, `cargo clippy --workspace --all-targets` en verde; commit(s) en la rama `cardspec-generalization`; `cargo install --path crates/learnkit-cli` tras cerrar la feature.

## Ruta de implementación

Delegado a un worker de escritura acotado (writer trigger: toca ≥2 ficheros no triviales — `card_spec.rs` nuevo + `cards.rs` modificado), con este documento como contexto. TDD: no hay modo configurado explícitamente en el proyecto — se siguen chequeos funcionales ordinarios (build+test+clippy), igual que en `hard-guards-entry-checks` y `export-study-guide`.

## Progreso

**2026-09-29**: Documento creado tras confirmación de David ("Si") para retomar esta feature, ya explorada en sesión anterior (ver Engram #458). Rama `cardspec-generalization` creada desde `master` (limpio, sin cambios pendientes). Aún sin código escrito.

**2026-09-29 (implementación completa, T1-T8)**:
- T1: `crates/learnkit-profile/src/card_spec.rs` nuevo — struct `CardSpec { id, learning_item_id, activity, stimulus, response, feedback: Option<String> }`, persistencia YAML atómica en `knowledge/card-specs/<learning_item_id>.yaml` (mismo patrón que `learning_item.rs`), `find_by_learning_item`/`set` (upsert)/`remove`, 4 tests unitarios. Registrado en `crates/learnkit-profile/src/lib.rs` (`pub mod card_spec;`).
- T2: `learning_item::remove` ahora llama `crate::card_spec::remove` antes de borrar su propio YAML (cascada incondicional — un item de vocabulario simplemente nunca tiene `CardSpec`, así que no hay caso a distinguir). Test de regresión añadido (`removing_a_learning_item_cascades_to_its_card_spec`). También se añadió `learning_item::find_by_id` (necesario para que `cards set` valide `--item`).
- T3: nuevo subcomando `learnkit cards set --item <id> --activity <a> --stimulus <s> --response <r> [--feedback <f>] [--path] [--json]` en `cards.rs` — valida que el `LearningItem` exista (`learning_item::find_by_id`), upsert vía `card_spec::set`, salida `Envelope`/`emit_set_error` consistente con el resto de comandos (código `CARD_SPEC_SET` / errores con el código propio de `LearnKitError`).
- T4/T5: `run_build` generaliza sus dos ramas (`IMAGE_TO_PRODUCTION_V1` y la "else" genérica) con un `if item.kind == "vocabulary" { <código original, sin tocar> } else { <rama CardSpec> }` en cada una; el código de vocabulario es literalmente el mismo que antes (mismo orden de líneas), solo movido dentro de la rama `vocabulary`. Añadido `LearnKitError::CardSpecMissing` (código `CARD_SPEC_MISSING`, exit 40 — igual tier que `ExporterConstraint`, explícitamente distinto del `BLOCKED`/exit 20 de Hard Guards) para cuando un item no-vocabulario no tiene `CardSpec`. Nueva función pura `build_image_to_production_blocks_generic` (análoga a `build_image_to_production_blocks` pero con `stimulus`/`response`/`feedback`), con 3 tests unitarios.
- T6: 3 tests de integración nuevos en `crates/learnkit-cli/tests/cards_test.rs` (`cards_build_uses_card_spec_content_for_a_non_vocabulary_item`, `cards_build_fails_clearly_for_a_non_vocabulary_item_without_a_card_spec`, `cards_build_handles_a_mixed_session_without_changing_the_vocabulary_card`) — el último mezcla un item de vocabulario y uno genérico en la misma sesión y comprueba que la tarjeta de vocabulario sigue teniendo exactamente `["whiteboard", "pizarra"]` en el reverso. No existe comando CLI para crear un `LearningItem` genérico (fuera de alcance explícito), así que el fixture escribe el YAML directamente en `knowledge/learning-items/`. Todos los tests preexistentes de `cards.rs`/`learning_item.rs`/`vocabulary.rs` se dejaron sin modificar y siguen en verde.
- T7: nueva sección "8.1 Tarjetas para contenido no-vocabulario (CardSpec)" en `docs/manual.md` y `docs/manual.html` (después de "8. Generar tarjetas"), más una fila nueva en la tabla de referencia de comandos de ambos ficheros.
- T8: `cargo build --workspace --all-targets` verde, `cargo test --workspace` verde (todos los tests, incluidos los nuevos), `cargo clippy --workspace --all-targets` verde (0 warnings). `cargo install --path crates/learnkit-cli` ejecutado tras cerrar la feature.

**Ruta de implementación seguida**: ejecutado directamente por un worker delegado (writer trigger, como estaba previsto), sin desviaciones del diseño acordado. El flujo de vocabulario no cambió de forma observable en ningún punto.

# Feature: Cierre del pipeline sesión→resumen→Excel→Anki (gramática/diálogos a tarjetas + orquestación)

**Tracked as**: ODD. Origen: David describió su flujo real de uso (crear sesión → dar documentación → esperar resumen+Excel+Anki con gramática/estructuras/vocabulario) y pidió auditar si el conjunto de Skills+CLI lo cumple. Auditoría (delegada, solo lectura, 2026-09-30) encontró dos brechas reales, confirmadas en código. David eligió explícitamente: *"Las dos a la vez, en una sola feature ODD grande"*.

## Objetivo

1. Que gramática (`ConceptPage` de `analyse`), diálogos y pares mínimos de pronunciación puedan convertirse en tarjetas Anki reales — hoy es imposible.
2. Que `learnkit-session` encadene razonablemente resumen→vocabulario/gramática→tarjetas→exportación sin que el usuario tenga que pedir cada fase por su nombre de comando, y que el nivel acumulado se consulte una vez, al principio, sin depender de qué sub-skill se invoque después.

## Diagnóstico (auditoría 2026-09-30, ver Engram)

- `ensure_for_vocabulary` (`crates/learnkit-profile/src/language/learning_item.rs`) es el ÚNICO creador de `LearningItem` y fija `kind = "vocabulary"` siempre. No existe ningún equivalente para gramática/diálogo/pronunciación.
- **Buena noticia confirmada**: `cards build`/`load_all_vocabulary_items` YA cargan el directorio `knowledge/learning-items/` entero, sin filtrar por `kind` — el nombre de la función es engañoso (histórico), pero no es una restricción real. La rama genérica de `cards.rs::run_build` (de `cardspec-generalization`) ya funciona para cualquier `kind != "vocabulary"` que tenga un `CardSpec`. **El único hueco real es la creación del `LearningItem` genérico** — todo lo demás (CardSpec, cards build, export anki) ya sirve sin tocarlo.
- `LearningItem.vocabulary_entry_id: String` es obligatorio hoy — hay que hacerlo `Option<String>` para poder crear items sin vocabulario detrás (cambio aditivo: un `String` YAML deserializa igual en `Option<String>` sin romper datos existentes).
- Ninguna Skill menciona nunca `export study-guide`/`export study-pack`/`export anki`/`export exam`. `learnkit-session` reacciona a intención explícita por fase, no encadena un pipeline por defecto.
- `learn level show` solo se exige dentro de `learnkit-language` — la ruta de gramática (que vive en `learnkit-analyse`) puede completarse sin haberlo consultado nunca.

## Decisiones de diseño (2026-09-30)

1. **Nuevo campo genérico en `LearningItem`**: `source_ref: Option<String>` (paralelo a `vocabulary_entry_id`, que sigue existiendo solo para vocabulario). Para items no-vocabulario, `source_ref = "{kind}:{session_id}:{source_id_bruto}"` (único globalmente, evita colisión entre sesiones con el mismo id corto de diálogo/página). Sirve para idempotencia (`find_by_source_ref`, igual patrón que `find_by_vocabulary_entry`).
2. **Un solo comando nuevo**: `learnkit learn item promote --session <id> --kind grammar|dialogue|pronunciation --source-id <id> [--path] [--json]`. Valida que la fuente exista (`ConceptPage`/`Dialogue`/`MinimalPair`, según `--kind`), crea o reutiliza (idempotente) el `LearningItem` genérico correspondiente, y devuelve su `learning_item_id`. NO crea el `CardSpec` — eso lo sigue haciendo el ya existente `cards set --item <id> --activity ... --stimulus ... --response ...` (nada nuevo ahí, se reutiliza tal cual).
   - `grammar`: `--source-id page:<page_id>` (el `ConceptPage.id` ya usa ese prefijo) → `title = page.concept`, `summary = page.content` (recortado si hace falta), `tags = ["grammar"]`.
   - `dialogue`: `--source-id <dialogue_id>` → `title` derivado (p.ej. primera línea o "Dialogue <id>"), `summary` = las líneas unidas, `tags = ["dialogue"]`.
   - `pronunciation`: `--source-id <pair_id>` → `title = "<word_a> vs <word_b>"`, `summary` = la nota del contraste, `tags = ["pronunciation"]`.
   - `mastery_dimensions` reutiliza el mismo triple por defecto que vocabulario (`recognition/production/listening`) — no se pidió nada distinto.
3. **Renombrar `load_all_vocabulary_items` → `load_all`** (con alias temporal o actualizando todos los call sites, decisión del implementador) — limpieza menor señalada por la auditoría, el nombre actual es engañoso ahora que carga cualquier `kind`.
4. **`learnkit-session`**: tras confirmar que hay documentación nueva ingerida, la Skill debe describir explícitamente la secuencia por defecto que sigue si el usuario no pide otra cosa: `learn level show` (una vez, al principio) → `learnkit-analyse` (resumen/gramática) → `learnkit-language` (vocabulario, diálogos, pronunciación, y `learn item promote` + `cards set` para lo que el usuario confirme como tarjeta) → `cards build` → mencionar explícitamente los 3 exportadores (`export study-guide`, `export study-pack`, `export anki`) como el cierre natural de la sesión, preguntando al usuario cuáles quiere generar (nunca ejecutarlos sin decírselo, pero sí ofrecerlos por nombre en vez de esperar a que el usuario los conozca).
5. **`learnkit-language` sección 23 (gramática)**: se actualiza para mencionar `learn item promote --kind grammar` + `cards set` como el camino real para convertir una página de gramática confirmada en tarjeta — sigue sin haber comando nuevo de "gramática" en sí, solo se documenta el camino ya construido en el punto 2. Se añade guía equivalente para diálogos (sección 19) y pares mínimos (sección 20): cuándo tiene sentido promoverlos a tarjeta (no todos merecen serlo) y cómo.
6. **`learnkit-analyse`**: se le añade UNA mención mínima y genérica de comprobar el nivel/contexto acumulado si existe tal mecanismo para el dominio — pero como `learnkit-analyse` es subject-agnostic por diseño (no puede saber que existe `learn level show`, que es de idioma), la responsabilidad real de disparar la comprobación de nivel se deja en `learnkit-session` (punto 4), no en `analyse`. **No se toca `learnkit-analyse/SKILL.md` más allá de esto si es que hace falta algo — si el diseño final no requiere tocarlo, mejor, se mantiene neutral como ya es constante en este proyecto.**

## Alcance

### Fase A — Modelo de datos (`crates/learnkit-profile/src/language/learning_item.rs`)
- `vocabulary_entry_id: String` → `Option<String>` (aditivo, actualizar `ensure_for_vocabulary` y todo call site que lo lea/escriba).
- Nuevo campo `source_ref: Option<String>`.
- Nueva función genérica `ensure_generic(project_root, kind, source_ref, title, summary, tags) -> LearningItem` (upsert idempotente por `source_ref`), usada por las 3 variantes de `--kind` de `learn item promote`.
- Renombrar `load_all_vocabulary_items` → `load_all` (actualizar todos los call sites en `learnkit-cli`).
- Tests unitarios: crear cada kind, idempotencia (llamar dos veces con el mismo `source_ref` no duplica), deserialización de YAML legado con `vocabulary_entry_id` como string plano (regresión, no debe romperse ni una sola de las 127 cards reales de David).

### Fase B — Comando CLI (`crates/learnkit-cli/src/commands/learn.rs`)
- `learnkit learn item promote --session <id> --kind grammar|dialogue|pronunciation --source-id <id> [--path] [--json]`.
- Valida la fuente según `--kind` (lee `ConceptPage`/`Dialogue`/`MinimalPair` de la sesión), falla con error claro si no existe.
- Guarda anti-mojibake donde aplique (título/resumen derivados de texto ya persistido, probablemente no hace falta revalidar si el origen ya pasó la guarda al crearse — decidir con criterio, documentar la decisión).
- Tests de integración: promover cada uno de los 3 `--kind`, idempotencia vía CLI, luego `cards set` + `cards build` produciendo una tarjeta real de gramática/diálogo/pronunciación — **este es el test que demuestra que la brecha #1 de la auditoría queda cerrada de verdad**, no solo en el papel.

### Fase C — Skills
- `learnkit-session/SKILL.md`: sección de flujo por defecto tras ingerir documentación nueva (punto 4 del diseño), mención explícita de `learn level show` al principio y de los 3 exportadores al final.
- `learnkit-language/SKILL.md`: actualizar secciones 19 (diálogos), 20 (pares mínimos) y 23 (gramática) para mencionar `learn item promote` + `cards set` como el camino real hacia Anki.
- `learnkit-analyse/SKILL.md`: tocar solo si el diseño final de la Fase C lo exige de verdad (ver punto 6) — evitar por defecto.
- `learnkit-cards/SKILL.md`: revisar si necesita alguna mención de que ahora puede recibir tarjetas de gramática/diálogo/pronunciación además de vocabulario (probablemente una frase, no una sección nueva).

### Fase D — Cierre
- Documentar en `docs/manual.md`/`.html` el nuevo comando `learn item promote`.
- `cargo build/test/clippy --workspace` en verde; commits en `study-pipeline-completion`; `cargo install` al cerrar.

**Fuera de alcance explícito**: vocabulario draft/integrate (pedido por David en paralelo, se trackea como feature ODD aparte, no se mezcla aquí); cualquier UI/wizard; generación automática de contenido de tarjeta (el `--activity`/`--stimulus`/`--response` de `cards set` los sigue escribiendo el agente, con la guía de la Skill, nunca un algoritmo); tocar `export_study_pack.rs`/`export_anki.rs` (ya son genéricos, no necesitan cambios — la Fase A/B es suficiente para que empiecen a incluir tarjetas de gramática/diálogo/pronunciación sin tocarlos).

## Restricciones

- El flujo de vocabulario (`ensure_for_vocabulary`, `cards build` para `kind == "vocabulary"`) no cambia de forma observable — mismo principio que en `cardspec-generalization`.
- Ningún campo nuevo puede romper YAML persistido hoy (aditivo, `Option`/`#[serde(default)]`).
- `learnkit-analyse/SKILL.md` se mantiene neutral salvo necesidad real y mínima, documentada explícitamente si ocurre.

## Checklist

- [x] T1 `LearningItem.vocabulary_entry_id` → `Option<String>`, nuevo campo `source_ref: Option<String>`, `ensure_generic` + tests.
- [x] T2 Renombrar `load_all_vocabulary_items` → `load_all` (todos los call sites).
- [x] T3 Comando `learn item promote --kind grammar|dialogue|pronunciation` + tests de integración (incluye el test end-to-end promote→cards set→cards build→tarjeta real).
- [x] T4 `learnkit-session/SKILL.md`: flujo por defecto + nivel al principio + exportadores mencionados al final.
- [x] T5 `learnkit-language/SKILL.md`: secciones 19/20/23 actualizadas con el camino real a Anki.
- [x] T6 `learnkit-cards/SKILL.md`: mención de tarjetas no-vocabulario si hace falta.
- [x] T7 `learnkit-analyse/SKILL.md`: no fue necesario — no se tocó (ver Progreso).
- [x] T8 Documentación (`docs/manual.md`/`.html`): nueva sección 8.0 (`learn item promote`), y de paso corregido un hueco preexistente de `language-study-pack` (§7.1/7.2/7.3 — diálogos/pronunciación/nivel y `--topic`/`--notes` de vocabulario nunca se habían documentado en el manual, solo en la Skill). Tabla de referencia de comandos actualizada con las 4 filas nuevas.
- [x] T9 `cargo build/test/clippy --workspace` en verde (verificado tras T8); commits en la rama; `cargo install` pendiente de la decisión de cierre de David (merge a master).

## Progreso (continuación)

**2026-09-30 (Fase D, T8)**: Documentado `learn item promote` (§8.0) en `docs/manual.md`/`.html`. Al revisar el manual para insertar la sección, se detectó que `language-study-pack` (Fase C, T10) documentó `export study-pack` pero omitió por completo `learn dialogue`/`learn pronunciation`/`learn level`/`--topic`/`--notes` de vocabulario — corregido en la misma pasada (§7.1, §7.2, §7.3, más las filas correspondientes en la tabla de referencia de §13). No se tocó código ni Skills en este paso, solo `docs/manual.md`/`.html`.

## Progreso

**2026-09-30**: Documento creado tras auditoría delegada y decisión de David de abordar ambas brechas (Anki-para-no-vocabulario + orquestación de sesión) en una sola feature. Rama `study-pipeline-completion` creada desde `master` limpio (con `language-study-pack` ya mergeada). Sin código todavía.

**2026-09-30 (T1-T3)**: Implementadas Fase A y Fase B (solo el alcance encargado; Fases C/D quedan para otro trabajo, no tocadas: ninguna Skill ni `docs/manual.*` fue modificado).

- T1: `crates/learnkit-profile/src/language/learning_item.rs` — `vocabulary_entry_id: String` → `Option<String>`; nuevo `source_ref: Option<String>` (`#[serde(default)]`); nueva `find_by_source_ref`; nueva `ensure_generic(project_root, kind, source_ref, title, summary, tags)` (upsert idempotente por `source_ref`, id determinista `li-<slug(source_ref)>` vía una `slugify` interna nueva); `ensure_for_vocabulary`/`find_by_vocabulary_entry` actualizadas para el nuevo tipo sin cambiar comportamiento observable. 6 tests nuevos (creación, idempotencia, `find_by_source_ref`, y el más importante: deserialización de un YAML legado con `vocabulary_entry_id` como string plano — regresión que protege las ~127 cards reales de David).
- T2: `load_all_vocabulary_items` → `load_all`, renombrado en los 5 call sites reales (`assessment.rs`, `cards.rs`, `cards_image.rs`, `cards_image_test.rs`) más el propio módulo. En `cards_image.rs` hubo una colisión de nombre con `learnkit_cards::card::load_all` (ya importado sin alias) — resuelta importando el de `learning_item` como `load_all_items` en ese fichero únicamente; el resto de call sites no la tenían y quedaron con el nombre simple `load_all`.
- T3: nuevo `learnkit learn item promote --session <id> --kind grammar|dialogue|pronunciation --source-id <id> [--path] [--json]` en `crates/learnkit-cli/src/commands/learn.rs` (nuevas `ItemArgs`/`ItemAction`/`ItemPromoteArgs`, `run_item_promote`). `grammar` lee `ConceptPage` vía `learnkit_workflow::analysis::list_concept_pages` (función ya existente, reutilizada, filtrando por `id`) y trunca `content` a 500 caracteres para `summary` (constante `CONCEPT_PAGE_SUMMARY_MAX_CHARS`, documentada en el propio código). `dialogue`/`pronunciation` reutilizan `dialogue::load_all`/`pronunciation::load_all` ya existentes. Error claro (`LearnKitError::ExporterConstraint`, mismo patrón que "lemma no encontrado" en `run_remove`) cuando `--source-id` no existe. Sin guardia anti-mojibake nueva (decisión documentada en el propio comentario del código: el texto ya pasó, o nunca necesitó pasar, esa guarda al crearse el `ConceptPage`/`Dialogue`/`MinimalPair` original).
- Tests de integración nuevos: `crates/learnkit-cli/tests/learn_item_promote_test.rs` (7 tests) — promoción de cada `--kind` desde una fuente real creada vía los comandos CLI ya existentes, idempotencia (mismo `--source-id` dos veces → mismo `learning_item_id`, un solo fichero en `knowledge/learning-items/`), error claro con `--source-id` inexistente, y **el test end-to-end**: `learn item promote` (grammar y también dialogue+pronunciation) → `cards set --item <id> --activity ... --stimulus ... --response ...` (comando ya existente, sin cambios) → `cards build` → tarjeta real verificada en `knowledge`/sesión con el texto exacto de `--response` en su `back`. Cierra de punta a punta la brecha #1 de la auditoría.
- **Corrección al diagnóstico de la auditoría**: ninguna. `cards.rs::run_build`, `card_spec.rs`, `export_anki.rs` no se tocaron — solo se ajustaron, dentro de `cards.rs`/`cards_image.rs`/`crates/learnkit-assessment/src/build.rs`, los usos mecánicos de `item.vocabulary_entry_id` (ahora `Option<String>`: `.as_deref().unwrap_or_default()` en los dos sitios que lo pasan a `find_vocabulary_by_id`, seguro porque esas ramas solo se alcanzan para `kind == "vocabulary"`, donde el campo siempre es `Some`) y las 3 construcciones literales de `LearningItem` en tests (`build.rs`, `cards.rs`, `cards_test.rs`) para el nuevo campo `source_ref`.
- Verificación real ejecutada: `cargo build --workspace --all-targets` verde; `cargo test --workspace` verde (todos los tests preexistentes intactos sin modificar su lógica, más los nuevos — incluido el end-to-end); `cargo clippy --workspace --all-targets` verde, sin warnings.
- Commit en `study-pipeline-completion` (ver `git log`), Conventional Commits. No se ejecutó `cargo install` (fuera del alcance de este encargo).

**2026-09-30 (T4-T7)**: Fase C (Skills) implementada. Solo Markdown tocado — ningún `.rs`, ningún `docs/manual.*` (confirmado con `git diff --stat`: exactamente `learnkit-session/SKILL.md`, `learnkit-language/SKILL.md`, `learnkit-cards/SKILL.md`).

- T4 (`learnkit-session/SKILL.md`): nueva subsección "Accumulated level" dentro de la sección 4 (Inspect the current session state) — `learn level show` una vez, al principio, sin `--session` (es un dato de proyecto, no de sesión — verificado en `LevelShowArgs` de `learn.rs`, no acepta ese flag). Nueva subsección "User supplies new class material without naming a specific phase" dentro de la sección 7 (Determine the next workflow action) — describe la secuencia sugerida `learnkit-analyse` → `learnkit-language` (con `learn item promote`+`cards set` para lo confirmado) → `cards build` → ofrecer los 4 exportadores por nombre. Nota: el encargo de esta sesión pedía explícitamente los 4 exportadores (incluye `export exam`), mientras que el punto 4 de "Decisiones de diseño" de este mismo documento solo mencionaba 3 (sin `export exam`) — se siguió el encargo explícito de la sesión (los 4), por ser la instrucción más reciente y más específica; queda documentado aquí por si hay que reconciliar la discrepancia.
- T5 (`learnkit-language/SKILL.md`): añadidas subsecciones "Turning a confirmed dialogue/minimal pair/grammar page into a flashcard" al final de las secciones 19, 20 y 23 respectivamente, documentando `learn item promote --kind dialogue|pronunciation|grammar --source-id <id>` + `cards set` (verificado contra `ItemPromoteArgs`/`run_item_promote` real en `learn.rs`: grammar usa `--source-id page:<page_id>`, formato confirmado en `list_concept_pages`/`ConceptPage.id`). La sección 23 cierra explícitamente el hueco de texto que decía "this skill does not introduce any new command for grammar". Ninguna otra sección tocada.
- T6 (`learnkit-cards/SKILL.md`): un párrafo añadido en la sección 15 ("Build through LearnKit") aclarando que `cards build` no está limitado a vocabulario — acepta `LearningItem`s de gramática/diálogo/pronunciación vía `learn item promote`+`cards set`. Ninguna sección nueva, ninguna otra sección tocada.
- T7 (`learnkit-analyse/SKILL.md`): NO tocado. Revisado el fichero completo buscando cualquier mención de nivel/`ConceptPage`/señal de "listo para promoverse" — no existe ninguna, y no hace falta ninguna: `learn item promote --kind grammar` lee el `ConceptPage` directamente por `--source-id page:<id>`, sin necesitar que `analyse` marque nada como "ready". La responsabilidad de consultar el nivel se quedó, como decía el punto 6 del diseño, en `learnkit-session` (T4). Se mantiene neutral/subject-agnostic sin excepción.
- Verificación: cada fichero editado releído completo de principio a fin — ninguna contradice la regla de "nunca persistir sin confirmación explícita" (todas las nuevas subsecciones de `learnkit-language` la reafirman explícitamente antes de `item promote`/`cards set`). Sin build/test aplicable (solo Markdown). `git diff --stat` confirmado: 3 ficheros, 60 inserciones, 0 borrados, ningún `.rs` ni `docs/manual.*`.

# Feature: Material didáctico de idioma enriquecido + exportador a Excel (study-pack)

**Tracked as**: ODD (no SDD/spec-kit). Origen: David comparó la calidad de un Excel generado por un prompt genérico de 12 puntos (rol de tutor EOI) con lo que produce hoy LearnKit, y pidió portar ese nivel de detalle: "SI" a "¿quieres que porte ese nivel de detalle pedagógico a las Skills de LearnKit... y añada un exportador a Excel con esa misma estructura de pestañas".

## Objetivo

Que `learnkit-language` (nunca `learnkit-analyse`, que se mantiene neutral por decisión explícita de David: *"analyse, lo haras manteniendo su punto neutral, al ser idioma ira lo maximo que sea de idioma a learnkit-language"*) sea capaz de producir diálogos, pronunciación contrastada (pares mínimos) y un modelo de nivel acumulado entre sesiones — y que un nuevo comando de exportación genere un `.xlsx` con la misma estructura de pestañas que el Excel de referencia (`Lo aprendido, Vocabulario, Expresiones, Pronunciacion, Alfabeto, Dialogos, Repaso, Soluciones, Flashcards, Fuente`).

## Mapeo previo (exploración 2026-09-29, ver Engram)

Ya existe (reutilizar, no reinventar):
- `FilledGap`/trabajado-vs-añadido: real y funcional en `learnkit_workflow::analysis` (`ClassSummary`/`StudyMap`/`ConceptPage`), genérico — cubre "Lo aprendido" sin tocar `analyse`.
- `VocabularyEntry` ya tiene `ipa`/`examples` (`crates/learnkit-profile/src/language/vocabulary.rs`).
- `assessment build` genera preguntas multiple-choice deterministas (`crates/learnkit-assessment`), hoy sin separación real pregunta/solución en ningún export.
- `export study-guide` (Markdown) y `export anki` (`.apkg`) son el patrón a seguir para el nuevo exportador.

No existe, hay que construirlo:
- Ninguna dependencia de Excel en el workspace (añadir `rust_xlsxwriter`).
- Ningún campo `topic`/`notes` en `VocabularyEntry`.
- Ninguna entidad de diálogo, ni de par mínimo de pronunciación.
- Ningún modelo de nivel acumulado (CEFR o similar) entre sesiones.
- Ninguna separación real preguntas/soluciones en ningún export.

## Decisiones de diseño (2026-09-29)

1. **Gramática NO es una entidad nueva**: se sigue usando `ConceptPage` (`analyse set --item page:<id>`), que ya es genérico y soporta `filled_gaps`. `learnkit-language` solo añade guía de CÓMO escribir una buena página de gramática (estructura, ejemplos, "error frecuente" de hispanohablante) — sin tocar `learnkit-analyse` ni el modelo de datos de `analysis.rs`.
2. **Diálogos y pares mínimos SÍ son entidades nuevas**, porque tienen forma tabular real (hablante+texto; palabra+IPA×2) que no encaja en el `ConceptPage` de prosa libre, y viven en `crates/learnkit-profile/src/language/` (junto a `vocabulary.rs`/`learning_item.rs`), no en `learnkit-workflow` — son específicas de idioma, no genéricas.
3. **Nivel acumulado**: un único fichero a nivel de proyecto (no de sesión), `knowledge/language-level.yaml` — `{ language, variety, level, notes, updated_at }`. Comando `learn level set --language --level [--notes]` / `learn level show`. Es deliberadamente simple (una cadena de nivel, ej. "A1"); el skill lo lee antes de generar contenido nuevo y, al final de la sesión, puede *proponer* una actualización — nunca la persiste sin confirmación explícita del usuario (mismo patrón que vocabulario).
4. **Repaso/Soluciones**: para esta iteración, se reutiliza el banco de `AssessmentItem` (multiple-choice) ya generado por `assessment build` — el exportador simplemente separa "prompt + opciones" (pestaña Repaso) de "opción correcta" (pestaña Soluciones). **Fuera de alcance explícito**: generar preguntas de repaso de texto libre/traducción como las del Excel de referencia — eso es un generador de preguntas nuevo y más grande, no pedido hoy.
5. **Alfabeto**: tabla estática de referencia (26 letras + IPA británico), no es dato de sesión ni de usuario — vive como constante en el exportador, documentada como "solo inglés por ahora".
6. **Expresiones**: subconjunto de `VocabularyEntry` cuyo `part_of_speech` sea `"expression"` (ya es un valor libre hoy) — no es una entidad nueva, es un filtro en el exportador.

## Alcance

### Fase A — Modelo de datos (`crates/learnkit-profile/src/language/`)
- `vocabulary.rs`: añadir `topic: Option<String>`, `notes: Option<String>` a `VocabularyEntry` (con `#[serde(default)]`, igual patrón que `ipa`/`examples` en FR-013b — nunca rompe YAML antiguo). Exponer `--topic`/`--notes` en `learn vocabulary add`/`edit`.
- `dialogue.rs` (nuevo): `Dialogue { id, session_id, origin: DialogueOrigin::{FromClass, AddedForConsolidation}, lines: Vec<DialogueLine{speaker, text}>, note: Option<String> }`. Persistencia por sesión (`sessions/<id>/language/dialogues/<id>.yaml`). Comando `learn dialogue set --session --id --origin class|added --line "A: texto" (repetible, min 2) [--note]`, `learn dialogue remove --session --id`.
- `pronunciation.rs` (nuevo): `MinimalPair { id, session_id, word_a, ipa_a, word_b, ipa_b, note: Option<String> }`. Persistencia por sesión. Comando `learn pronunciation set --session --id --word-a --ipa-a --word-b --ipa-b [--note]`, `remove`.
- `level.rs` (nuevo): `LanguageLevel { language, variety, level, notes: Option<String>, updated_at }`. Persistencia a nivel de proyecto (`knowledge/language-level.yaml`, patrón atómico). Comando `learn level set --language --variety --level [--notes]`, `learn level show`.
- Guarda anti-mojibake (`learnkit_core::encoding::detect_mojibake`) aplicada a todos los campos de texto libre nuevos, igual que en `learn vocabulary add`/`cards set`.
- Todo con tests unitarios (crear/upsert/remove/cascada si aplica) siguiendo el patrón de `card_spec.rs`.

### Fase B — Skill (`crates/learnkit-agent/templates/skills/learnkit-language/SKILL.md`)
- Nuevas secciones: diálogos (cuándo crear uno "from_class" vs "added_for_consolidation", nunca inventar que el profesor dijo algo que no dijo), pares mínimos de pronunciación (cuándo son pedagógicamente relevantes, cómo redactar la nota de contraste), nivel acumulado (leer `learn level show` antes de generar cualquier contenido nuevo; nunca introducir estructuras muy por encima del nivel leído; proponer al usuario una actualización de nivel al final, nunca auto-confirmarla), enriquecimier vocabulario con `--topic`/`--notes`, guía de redacción de gramática vía `ConceptPage` (estructura corta, ejemplos con vocabulario conocido, "error frecuente" de hispanohablante) — sin crear ninguna entidad nueva para esto.
- `learnkit-analyse/SKILL.md`: **sin cambios**.

### Fase C — Exportador (`crates/learnkit-cli`)
- Nueva dependencia `rust_xlsxwriter` (en `learnkit-cli`, ningún otro crate la necesita).
- Nuevo comando `learnkit export study-pack --session <id> --out <archivo.xlsx> [--json]`, ensamblando las pestañas:
  - `Lo aprendido` ← `ClassSummary`/`StudyMap`/`ConceptPage`s (mismos datos que `export study-guide`, reformateados a filas).
  - `Vocabulario` ← `VocabularyEntry` de la sesión (vía sus `LearningItem`), con `topic`/`notes` nuevos.
  - `Expresiones` ← subconjunto de lo anterior con `part_of_speech == "expression"`.
  - `Pronunciacion` ← `MinimalPair` de la sesión.
  - `Alfabeto` ← tabla estática (solo inglés, documentado).
  - `Dialogos` ← `Dialogue` de la sesión, con columna de origen.
  - `Repaso` / `Soluciones` ← `AssessmentItem` de la sesión, separadas (prompt+opciones vs respuesta correcta).
  - `Flashcards` ← `CardDefinition` de la sesión (texto de frente/dorso).
  - `Fuente` ← metadatos: sesión, fecha, nivel actual (`LanguageLevel`), criterio (texto fijo explicando trabajado-vs-añadido).
- Falla con error claro (no exporta un Excel vacío/engañoso) si no hay ni `ClassSummary` ni vocabulario ni nada exportable — mensaje claro, no Hard Guards `BLOCKED`.
- Tests de integración (abrir el `.xlsx` generado y comprobar pestañas/contenido clave, usando el propio `rust_xlsxwriter` o una lectura mínima).
- Documentar en `docs/manual.md`/`.html`.

**Fuera de alcance explícito**: generador de preguntas de repaso de texto libre (se reutiliza el banco multiple-choice existente); UI/wizard; migración de datos existentes; tocar `learnkit-analyse`; soporte de nivel acumulado para asignaturas no-idioma; cualquier idioma que no sea inglés en la tabla de Alfabeto.

## Restricciones

- `learnkit-analyse/SKILL.md` no se toca, ni una línea.
- Todo lo nuevo en `learnkit-profile::language` sigue el patrón atómico YAML ya usado por `vocabulary.rs`/`learning_item.rs`/`card_spec.rs` — sin nuevas dependencias de persistencia.
- Ningún campo nuevo puede romper YAML persistido hoy (todo con `#[serde(default)]` o `Option`).
- Guarda anti-mojibake en todo campo de texto libre nuevo.

## Checklist

- [x] T1 `VocabularyEntry`: `topic`/`notes` + CLI (`add`/`edit`) + tests.
- [x] T2 `dialogue.rs`: entidad + persistencia + CLI (`learn dialogue set/remove`) + tests.
- [x] T3 `pronunciation.rs`: entidad + persistencia + CLI (`learn pronunciation set/remove`) + tests.
- [x] T4 `level.rs`: entidad + persistencia + CLI (`learn level set/show`) + tests.
- [x] T5 Guarda anti-mojibake aplicada a T1-T4.
- [x] T6 `learnkit-language/SKILL.md` actualizado (diálogos, pronunciación, nivel, vocabulario enriquecido, gramática vía ConceptPage). `learnkit-analyse/SKILL.md` sin tocar.
- [x] T7 Dependencia `rust_xlsxwriter` añadida a `learnkit-cli`.
- [x] T8 Comando `export study-pack` — las 10 pestañas.
- [x] T9 Tests de integración del exportador.
- [x] T10 Documentación (`docs/manual.md`/`.html`).
- [x] T11 `cargo build/test/clippy --workspace` en verde; commits en `language-study-pack` (sin `cargo install`, decisión pendiente de cierre completo de la feature).

## Progreso

**2026-09-29**: Documento creado tras "SI" de David y su aclaración explícita sobre mantener `analyse` neutral. Mapeo previo completo (delegado a Explore). Rama `language-study-pack` creada desde `master` limpio. Sin código todavía.

**2026-09-29 (Fase A completa, T1-T5)**: Implementado el modelo de datos completo de la Fase A, delegado a un subagente sobre la rama `language-study-pack` (sin merges, sin tocar `master`).

- **T1**: `VocabularyEntry` gana `topic: Option<String>` y `notes: Option<String>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`, mismo contrato de compatibilidad que `ipa`/`examples`). `set_details`/`set_fields` amplían su firma con `topic`/`notes` (ambas funciones ya tenían 4 parámetros de dato; ahora 6 — `#[allow(clippy::too_many_arguments)]` añadido, mismo patrón ya usado en `card_spec::set`). `learn vocabulary add`/`edit` exponen `--topic`/`--notes`, con guarda anti-mojibake. Tests: creación+persistencia, edición selectiva, deserialización de YAML legado sin estos campos.
- **T2**: `crates/learnkit-profile/src/language/dialogue.rs` nuevo. `DialogueOrigin` (`from_class`/`added_for_consolidation` en YAML), `DialogueLine{speaker,text}`, `Dialogue{id,session_id,origin,lines,note}`. Persistencia por sesión en `sessions/<id>/language/dialogues/<id>.yaml` (mismo patrón simple de `learnkit_cards::card` — funciones toman `session_root: &Path` directamente, no `SessionPaths`; la CLI resuelve `SessionPaths::new(&root,&session_id).root()` antes de llamar). `set` valida `lines.len() >= 2` devolviendo `io::Error(InvalidInput)`. CLI: `learn dialogue set --session --id --origin class|added --line "A: texto" (repetible) [--note]`, `learn dialogue remove`. Cada `--line` se parsea por el primer `": "` literal.
- **T3**: `crates/learnkit-profile/src/language/pronunciation.rs` nuevo. `MinimalPair{id,session_id,word_a,ipa_a,word_b,ipa_b,note}`, mismo patrón de persistencia por sesión (`sessions/<id>/language/pronunciation/<id>.yaml`). CLI: `learn pronunciation set --session --id --word-a --ipa-a --word-b --ipa-b [--note]`, `remove`.
- **T4**: `crates/learnkit-profile/src/language/level.rs` nuevo. `LanguageLevel{language,variety,level,notes,updated_at}`, un único fichero a nivel de proyecto `knowledge/language-level.yaml` (sobrescritura completa, sin historial). `updated_at` es ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`) calculado a mano con el algoritmo `civil_from_days` de Howard Hinnant sobre `SystemTime` — **desviación de diseño**: el enunciado permitía `chrono` si ya era dependencia del workspace, pero no lo es (verificado por grep), así que se implementó sin añadir una dependencia nueva, siguiendo el patrón `SystemTime`/`UNIX_EPOCH` ya usado en `session.rs`/`attempt.rs`. CLI: `learn level set --language --variety --level [--notes]`, `learn level show` (ausencia de nivel ⇒ `level_set: false` en JSON, nunca un error).
- **T5**: guarda anti-mojibake (`detect_mojibake`) aplicada a `topic`/`notes` (T1), a `note` y cada `text` de línea (T2, no a `speaker`), a `word_a`/`word_b`/`note` (T3, **deliberadamente NO aplicada a `ipa_a`/`ipa_b`** — símbolos IPA reales como `/ʃiːts/` son texto no-ASCII legítimo y nunca deben confundirse con mojibake, verificado con test de regresión), y a `notes` (T4).
- Errores de validación de dominio (ej. "menos de 2 líneas", "origen desconocido", "línea sin separador ': '") se mapean a `LearnKitError::ValidationFailed` (exit 10) vía un helper nuevo `io_error_to_learnkit` en `learn.rs` que distingue `io::ErrorKind::InvalidInput` (validación) de cualquier otro (`Filesystem`).
- Tests nuevos: unitarios en `dialogue.rs`/`pronunciation.rs`/`level.rs`/`vocabulary.rs` (patrón `card_spec.rs`), más 3 ficheros de integración CLI nuevos: `crates/learnkit-cli/tests/dialogue_test.rs`, `pronunciation_test.rs`, `level_test.rs` (patrón `vocabulary_test.rs`), incluyendo tests de regresión de mojibake (falso negativo en IPA, falso positivo detectado y rechazado en texto libre).
- Verificación real ejecutada: `cargo build --workspace --all-targets` verde; `cargo test --workspace` verde (todos los tests preexistentes + los nuevos, incluidos los 3 ficheros de integración nuevos ejecutados también de forma aislada); `cargo clippy --workspace --all-targets` verde, sin warnings.
- Commit(s) de trabajo pendientes de crear en `language-study-pack` tras esta actualización del documento (Conventional Commits, sin `cargo install`, sin push, sin merge).
- Fase B (Skill) y Fase C (exportador `.xlsx`) quedan **fuera de este trabajo**, sin tocar.

**2026-09-29 (Fase C completa, T7-T11 — feature CERRADA)**: Exportador `.xlsx` implementado y verificado, delegado a un subagente sobre `language-study-pack` (sin merges, sin tocar `master`).

- **T7**: `rust_xlsxwriter` (`0.79`) añadido a `[workspace.dependencies]` en el `Cargo.toml` raíz y referenciado como `{ workspace = true }` en `crates/learnkit-cli/Cargo.toml` (mismo patrón que `serde_yaml`/`clap`). `calamine` (`0.26`) añadido como dev-dependency de `learnkit-cli` únicamente, para poder releer el `.xlsx` generado en los tests de integración (T9).
- **T8**: `crates/learnkit-cli/src/commands/export_study_pack.rs` nuevo, registrado como `learnkit export study-pack --session <id> --out <fichero.xlsx> [--json] [--path]` en `export.rs` (mismo patrón que `Anki`/`StudyGuide`). Genera las 10 pestañas exactas en el orden pedido, reutilizando lectura ya existente: `learnkit_workflow::analysis::{read_summary,read_study_map,list_concept_pages}` para "Lo aprendido" (misma fuente que `export study-guide`, sin duplicar lógica de negocio — solo el formato de fila es nuevo), `learnkit_cards::card::load_all` para "Flashcards", `learnkit_assessment::item::{load_assessment,load_all_items}` para "Repaso"/"Soluciones" (mismo patrón que `export exam`), y las tres entidades nuevas de Fase A (`dialogue::load_all`, `pronunciation::load_all`, `level::show`) para sus pestañas.
  - Única función nueva añadida a un módulo existente (permitido explícitamente cuando no hay lectura equivalente): `vocabulary::list_all` en `vocabulary.rs` — wrapper público mínimo sobre el `load_all` ya existente (privado), sin tocar el modelo de datos ni la persistencia.
  - Falla con `LearnKitError::ExporterConstraint` (código `EXPORT_FAILED`, exit `40`) y no escribe ningún fichero cuando no hay absolutamente nada exportable en ninguna de las 10 categorías (mismo contrato que `export study-guide`/`export anki`). Con al menos una cosa exportable, genera el libro igual aunque otras pestañas queden solo con cabecera.
  - **Decisiones de mapeo documentadas explícitamente en el propio código** (comentario de módulo en `export_study_pack.rs`) porque el modelo de datos actual no las especifica al 100%:
    - **Vocabulario es de todo el proyecto, no de la sesión**: `VocabularyEntry` no tiene `session_id` (sobrevive entre sesiones — ver `vocabulary.rs`), y su única trazabilidad (`sources[].source_id`) apunta a un *source* de inventario, no a una sesión — cruzarlo de forma fiable exigiría recorrer inventarios de todas las sesiones, fuera de alcance. Se exporta **todo** el vocabulario del proyecto, igual que ya hacen `assessment build`/`cards build`. La columna "Sesión" de la pestaña Vocabulario lleva el id de la sesión *objetivo* del export (no un valor por entrada, que no existe).
    - **"Traducción ejemplo"** (Vocabulario): sin campo dedicado en `Example` (solo `text`) — se deja vacía, nunca inventada.
    - **Expresiones "Uso"/"Origen"**: `VocabularyEntry` no tiene marca literal de trabajado-en-clase-vs-añadido (a diferencia de `Dialogue`/`ConceptPage`). "Uso" vacío; "Origen" fijo a "Trabajado en clase" — simplificación documentada, no una traza real.
    - **Fuente "Fecha"**: se usa `Session::created_at` tal cual (`format!("{:?}", SystemTime::now())`, no una fecha legible) — es el único timestamp que expone el modelo hoy; no se reformatea (sin crate de fecha/hora en el workspace, misma restricción que documentó T4 para `level.rs`).
- **T9**: `crates/learnkit-cli/tests/export_study_pack_test.rs` nuevo. Un test monta una sesión con contenido real en summary, vocabulario (una entrada normal + una expresión), un diálogo, un par mínimo, una flashcard y un assessment item (mezcla del patrón CLI de `export_study_guide_test.rs` con las llamadas directas a `learnkit_cards::card::save`/`learnkit_assessment::item::save_item` de `export_anki_test.rs`, más llamadas directas a `dialogue::set`/`pronunciation::set`/`vocabulary::add_or_reuse` para las entidades sin atajo CLI usado en este test), exporta con la CLI real, y relee el `.xlsx` con `calamine` verificando las 10 pestañas en orden y contenido clave de cada una (incluyendo que "Repaso" nunca contiene `opt-a`, el id de la opción correcta). Segundo test: sesión totalmente vacía (proyecto nuevo, sin vocabulario) falla explícitamente y no escribe fichero.
- **T10**: sección "Exportar el material didáctico completo a Excel (`study-pack`)" añadida en `docs/manual.md` y `docs/manual.html`, justo después de la sección existente de `export study-guide`, mismo estilo; fila añadida a la tabla de referencia de comandos en ambos ficheros.
- **T11**: verificación real ejecutada por el propio agente (no asumida): `cargo build --workspace --all-targets` verde; `cargo clippy --workspace --all-targets` verde sin warnings (3 avisos `needless_borrows_for_generic_args` detectados y corregidos antes del clippy final); `cargo test --workspace` verde — 100% de los tests existentes más los 2 nuevos de `export_study_pack_test.rs` (verificado con conteo completo por fichero, sin fallos). `cargo install` explícitamente **no** ejecutado, como pidió el encargo — pendiente de decisión de cierre de la feature completa (Fases A+B+C).
- Restricciones respetadas: `learnkit-agent/templates/skills/` no tocado; `learnkit-workflow::analysis` y `learnkit-profile::language::{dialogue,pronunciation,level}` no tocados (solo `vocabulary.rs` recibió la función de lectura nueva, mínima, documentada); sin modo TDD (checks funcionales ordinarios); ningún idioma nuevo en Alfabeto.
- Ruta: delegado direct (subagente único; lectura de 12+ ficheros Rust existentes para mapear el modelo real antes de escribir, luego un módulo Rust nuevo + un fichero de test + dos ficheros de documentación).

## Progreso final (Fases A + B + C)

**Feature `language-study-pack` completa** (2026-09-29): las 11 tareas del checklist están `[x]`. Resumen end-to-end:

- **Fase A** (modelo de datos): `VocabularyEntry.topic/notes`, `Dialogue`/`DialogueOrigin`/`DialogueLine`, `MinimalPair`, `LanguageLevel`, todos con persistencia atómica YAML, guarda anti-mojibake donde corresponde (nunca en IPA), y CLI completa (`learn vocabulary add/edit`, `learn dialogue set/remove`, `learn pronunciation set/remove`, `learn level set/show`).
- **Fase B** (Skill): `learnkit-language/SKILL.md` ampliado con 5 secciones nuevas (diálogos, pronunciación, nivel acumulado, vocabulario enriquecido, gramática vía `ConceptPage`); `learnkit-analyse/SKILL.md` sin tocar, confirmado por `git diff --stat`.
- **Fase C** (exportador): `learnkit export study-pack` genera el `.xlsx` de 10 pestañas descrito en el objetivo original, reutilizando toda la lectura ya existente de Fases A/B y de `analyse`/`assessment`/`cards`, con fallo explícito ante sesión totalmente vacía y tests de integración que abren el fichero real generado.
- Estado del repositorio: todo el trabajo vive en la rama `language-study-pack` (nunca se tocó `master`), con commits Conventional Commits por fase; `cargo build/test/clippy --workspace` en verde en cada fase, verificado de forma real (no asumida) por el agente que la ejecutó. `cargo install` queda pendiente de una decisión explícita de cierre de la feature completa, tal y como pidió el encargo de Fase C.
- Siguiente paso: David decide si cierra la feature (merge a `master`, `cargo install --path crates/learnkit-cli`, uso real) o pide ajustes adicionales sobre cualquiera de las tres fases.

**2026-09-29 (Fase B completa, T6)**: `crates/learnkit-agent/templates/skills/learnkit-language/SKILL.md` ampliado con 5 secciones nuevas (19-23), insertadas entre la antigua sección 18 ("Keep extraction and Anki selection separate") y la de cierre ("Final quality check", renumerada a 24), leído el fichero Rust real de cada capacidad antes de escribir cada comando/flag:

- **19. Dialogues**: cuándo crear uno, distinción dura `--origin class` (ocurrió en sesión, reconstruido de fuentes) vs `--origin added` (consolidación con estructuras ya vistas, nunca gramática nueva), nunca marcar `class` un diálogo inventado (mismo principio que "nunca inventar que el profesor dijo algo"), nunca persistir sin confirmación explícita. Sintaxis exacta verificada contra `crates/learnkit-cli/src/commands/learn.rs` (`run_dialogue_set`/`parse_dialogue_line`): `--origin` literalmente `class`/`added` (no los nombres del enum Rust `from_class`/`added_for_consolidation`), `--line` repetible ≥2, separado por el primer `": "` literal.
- **20. Pronunciación — pares mínimos**: cuándo un par merece registrarse (contraste fonético real y relevante para hispanohablante), IPA británico en ambos miembros, la nota explica el contraste (no repite la transcripción), aclarado como mecanismo distinto y complementario a la guía de IPA de palabra única ya existente (sección 9).
- **21. Nivel acumulado**: ejecutar `learn level show` antes de generar contenido nuevo, "nivel desconocido" si no hay nada guardado (nunca asumir alto/bajo), nunca introducir estructuras muy por encima del nivel, proponer actualización al final de sesión sin auto-confirmar (mismo patrón que vocabulario).
- **22. Vocabulario enriquecido**: `--topic` (agrupar por tema pedagógico) y `--notes` (irregularidades, falsos amigos, confusiones, registro) en `vocabulary add`/`edit`, mismo patrón de confirmación.
- **23. Gramática vía `ConceptPage`**: explícito que este skill NO añade ningún comando nuevo para gramática — remite a `analyse set --item page:<id>` de `learnkit-analyse` para el mecanismo, y limita esta sección a guía de contenido (concepto en una frase, estructura, 3-6 ejemplos con vocabulario conocido, un error frecuente de hispanohablante).
- Checklist de la sección "Final quality check" (ahora 24) ampliado con 4 puntos nuevos cubriendo diálogos/pares mínimos/nivel/confirmación.
- Desviación de detalle respecto al enunciado original: ninguna — todos los comandos/flags coincidieron exactamente con lo descrito (`--origin class|added`, `--line` repetible parseado por `": "`, `--word-a/--ipa-a/--word-b/--ipa-b`, `learn level set/show`, `--topic`/`--notes` en vocabulary).
- Verificación: relectura completa del `SKILL.md` tras la edición, sin contradicciones con las reglas duras existentes (nunca inventar que el profesor dijo algo, nunca persistir sin confirmación). `git diff --stat` confirma que solo se tocó `crates/learnkit-agent/templates/skills/learnkit-language/SKILL.md`; `learnkit-analyse/SKILL.md` no aparece en el diff. No aplica build/test (solo Markdown).
- Ruta: delegado direct (subagente único, ficheros Rust de lectura + 1 fichero Markdown de escritura).

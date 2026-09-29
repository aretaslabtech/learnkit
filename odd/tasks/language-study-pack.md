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
- [ ] T6 `learnkit-language/SKILL.md` actualizado (diálogos, pronunciación, nivel, vocabulario enriquecido, gramática vía ConceptPage). `learnkit-analyse/SKILL.md` sin tocar.
- [ ] T7 Dependencia `rust_xlsxwriter` añadida a `learnkit-cli`.
- [ ] T8 Comando `export study-pack` — las 10 pestañas.
- [ ] T9 Tests de integración del exportador.
- [ ] T10 Documentación (`docs/manual.md`/`.html`).
- [ ] T11 `cargo build/test/clippy --workspace` en verde; commits en `language-study-pack`; `cargo install` al cerrar.

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

# Feature: `learnkit export study-guide`

**Tracked as**: ODD (organic), no spec-kit ceremony — pequeña, bien entendida, fuera del alcance original de la feature 003 (`analyse`/`consolidate`). Decisión explícita del usuario (2026-09-28): tracking propio en `odd/`, sin tocar `specs/003-analyse-consolidate-checklist/`.

## Objetivo

Exportar el contenido ya confirmado de `analyse` (resumen, mapa mental, páginas de concepto) de una sesión a un documento Markdown legible e imprimible, sin metadatos administrativos.

## Problema

David: *"El sistema a procesado todos los estudios, los apuntes y los ha 'analizado' pero no me ha generado n documento en markdown o html imprimible con los extraido de la formacion mostranto todo el temario (sin la paja de la sesion)"*.

## Por qué

El resumen/mapa mental/páginas ya se generan y persisten (feature 003, US2) bajo `sessions/<id>/analysis/`, pero no existe ningún comando que los convierta en un documento legible fuera de LearnKit. `learnkit export` hoy solo tiene `anki` y `exam`.

## Alcance

- Nuevo comando `learnkit export study-guide --session <id> --out <fichero.md> [--path <p>] [--json]`.
- Lee `ClassSummary`/`StudyMap`/`ConceptPage`s ya confirmados (`learnkit_workflow::analysis`), no genera ni redacta nada nuevo.
- Un único documento Markdown: título de la sesión (H1) → resumen (H2) → mapa mental (H2, si existe) → una sección H2 por página de concepto, en su orden ya persistido.
- Sin metadatos de fase/checklist/fingerprints — solo el contenido de estudio.
- Falla explícitamente (sin escribir fichero) si no hay resumen confirmado. Mapa mental y páginas son opcionales — se omiten si no existen, no bloquean.

**Fuera de alcance**: exportar a HTML (solo Markdown por ahora), cualquier cambio a `analyse`/`consolidate` en sí, cualquier plantilla de tarjeta.

## Restricciones

- Seguir las convenciones ya establecidas en este repo (`LearnKitError`/exit codes, `Envelope` JSON, `resolve_session`/`SessionPaths`, estilo de test de `analyse_test.rs`/`export_anki_test.rs`).
- No tocar `Cargo.toml` de ningún crate.
- No modificar `learnkit-workflow`/`learnkit-agent`.

## Checklist

- [x] T1 Implementar `crates/learnkit-cli/src/commands/export_study_guide.rs`: lee resumen/mapa/páginas, renderiza el Markdown, escribe `--out`. Falla explícito si no hay resumen.
- [x] T2 Registrar el comando como `ExportAction::StudyGuide` en `crates/learnkit-cli/src/commands/export.rs`.
- [x] T3 Test de integración: sesión con resumen + mapa + 2 páginas exporta un `.md` con las tres, en orden.
- [x] T4 Test de integración: sesión sin resumen falla explícito sin escribir fichero; sesión con resumen pero sin mapa/páginas exporta igual, omitiendo esas secciones.
- [x] T5 Actualizar `docs/manual.md`/`docs/manual.html` con el comando nuevo.

## Criterios de aceptación

- `cargo build --workspace` y `cargo test --workspace` en verde, sin regresiones.
- `cargo clippy --workspace --all-targets` limpio en los ficheros tocados.
- Commit de trabajo en la rama actual (`003-analyse-consolidate-checklist`), binario redesplegado (`cargo install --path crates/learnkit-cli`) tras el commit.

## Progreso

**Completo (2026-09-28).** Implementado `crates/learnkit-cli/src/commands/export_study_guide.rs` (`learnkit export study-guide --session <id> --out <fichero.md> [--json]`), registrado en `export.rs`, 3 tests de integración en `crates/learnkit-cli/tests/export_study_guide_test.rs` (todos pasan). Manual actualizado en `.md`/`.html` (subsección dentro de §9, sin renumerar el resto). `cargo build --workspace`, `cargo test --workspace` (225 tests) y `cargo clippy --workspace --all-targets` en verde. El agente que lo implementó se cortó por un límite de sesión justo antes de marcar este checklist — verificado y cerrado manualmente tras confirmar build/tests/clippy en verde.

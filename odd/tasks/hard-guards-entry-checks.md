# Feature: Hard Guards reales — guardas de entrada por comprobación en vivo

**Tracked as**: ODD. Cierra el gap documentado en `learnkit-implementation-spec/docs/04-workflow-guards.md` (Principio IV de la constitución) — decisión anterior de David era "solo dejarlo documentado", revertida hoy: **"Si, son necesarias"**.

## Objetivo

Que `cards build` y `assessment build` rechacen explícitamente ejecutarse cuando la fase de la que dependen no tiene datos reales que la respalden — en vez de producir en silencio un resultado vacío (0 tarjetas / 0 preguntas) como hoy.

## Problema

Confirmado en código (no solo documentado): de todos los comandos, hoy **solo `inventory` y `consolidate` escriben manifest de fase**. `learn vocabulary add`, `cards build`, `export anki`, `assessment build` nunca lo hacen. Nada impide ejecutar `cards build` con cero vocabulario confirmado, o `assessment build` con cero elementos de aprendizaje — simplemente producen un resultado vacío sin avisar de que falta el paso anterior.

## Decisión de diseño (2026-09-28, para no romper uso real)

David tiene una sesión real con 127 tarjetas y vocabulario confirmado **sin que nunca se haya escrito un manifest** (el manifest no existía como mecanismo cuando él trabajó). Una guarda que exigiera un manifest guardado de `vocabulary` bloquearía retroactivamente su propio proyecto.

Opción elegida (de 3 presentadas): **comprobación en vivo contra los datos reales**, no contra un manifest cacheado. La guarda de `cards build`/`assessment build` no pregunta "¿hay un manifest de `vocabulary` marcado válido?" — pregunta directamente "¿existe al menos un `LearningItem` en el proyecto ahora mismo?". Sigue siendo Hard Guards de verdad (revalida siempre, nunca confía en una declaración vieja — Principio IV), pero nunca puede romper retroactivamente un proyecto real porque no depende de que alguien haya escrito algo en el pasado.

`export anki` ya tiene su propia guarda real (FR-019, completitud de tarjetas) — no se toca. `consolidate` ya tiene su propia guarda real (elementos de `analyse` pendientes) — no se toca. Esta feature solo añade lo que falta: `cards build` y `assessment build`.

## Alcance

- `learnkit cards build`: si `learnkit_profile::language::learning_item::load_all_vocabulary_items` devuelve una lista vacía, DEBE fallar explícitamente (exit 20, `code: "BLOCKED"`, mensaje claro: "no hay vocabulario confirmado — ejecuta `learn vocabulary add` primero") en vez de generar 0 tarjetas en silencio.
- `learnkit assessment build`: mismo criterio, misma comprobación, mismo código de salida.
- Reutilizar el mismo código/forma de error que ya usa `consolidate` para su `BLOCKED` (exit 20, `code: "BLOCKED"`) — consistencia entre guardas.

**Fuera de alcance explícito**: invalidación en cascada por fingerprint (si el vocabulario cambia después de generar tarjetas, no se detecta automáticamente — eso exigiría fingerprintar `VocabularyEntry`/`LearningItem`, una feature bastante más grande, no pedida hoy). Cualquier guarda nueva sobre `export anki`/`consolidate` (ya la tienen). Escribir manifest para `vocabulary`/`cards`/`anki`/`assessment` (no hace falta con este diseño — la comprobación es siempre en vivo, no contra manifest).

## Restricciones

- Nunca debe bloquear un proyecto que ya tiene datos reales (esa es la razón de ser del diseño elegido) — verificarlo explícitamente con un test que simula el caso de David (vocabulario ya existente, ningún manifest de fase escrito nunca, `cards build` debe funcionar con normalidad).
- Mismo patrón de error/exit code que `consolidate` ya usa para BLOCKED.
- No tocar `learnkit-workflow`'s `PhaseManifest`/`recompute_state` — esta feature no usa ese mecanismo en absoluto, por diseño.

## Checklist

- [x] T1 Guarda de entrada en `learnkit cards build` (`crates/learnkit-cli/src/commands/cards.rs`): falla BLOCKED si no hay ningún `LearningItem`.
- [x] T2 Guarda de entrada en `learnkit assessment build` (`crates/learnkit-cli/src/commands/assessment.rs`): mismo criterio.
- [x] T3 Test: `cards build` sin vocabulario falla BLOCKED, exit 20.
- [x] T4 Test: `cards build` con vocabulario ya existente (creado directamente, sin haber pasado nunca por ningún manifest de fase) funciona con normalidad — regresión explícita del caso de David.
- [x] T5 Test: `assessment build` sin vocabulario falla BLOCKED; con vocabulario existente funciona con normalidad.
- [x] T6 Documentar brevemente en `docs/manual.md`/`.html` (sección de solución de problemas) el nuevo código `BLOCKED`/exit 20 para estos dos comandos.

## Criterios de aceptación

- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets` en verde.
- El test de "vocabulario ya existente sin manifest histórico" (T4) es la prueba de que esto no rompe uso real — debe pasar sin ambigüedad.
- Commit de trabajo, binario redesplegado.

## Progreso

**Completo (2026-09-28).** Guardas de entrada añadidas a `cards build` y `assessment build` (comprobación en vivo: `LearningItem` vacío → `BLOCKED`, exit 20). `cargo build`/`test`/`clippy --workspace` en verde. Test de regresión explícito (`*_works_for_preexisting_vocabulary_with_no_phase_manifest_history`) confirma que vocabulario ya existente sin historial de manifest sigue funcionando con normalidad — no rompe el proyecto real de David. Documentado en el manual (`.md`/`.html`).

# Feature: generación de imágenes por rejilla 4x4 + revisión de coherencia (Wikimedia y generada)

**Tracked as**: ODD, feature nueva fuera del alcance de cualquier feature de spec-kit existente (extiende `cards build`/resolución de imagen, feature 002, pero introduce un flujo humano-en-el-bucle completamente nuevo).

## Objetivo

Dos modos de resolución de imagen para una tarjeta, ambos con un paso explícito de evaluación de coherencia antes de dar la imagen por buena:

1. **Wikimedia** (ya existente): añadir una evaluación de la imagen después de resolverla, antes de darla por definitiva.
2. **Generación por rejilla**: para conceptos que Wikimedia no puede ilustrar (abstractos, como "-ty numbers"), generar hasta 16 imágenes de una vez pidiendo a un modelo externo (ChatGPT u otro) una rejilla 4x4, recortarla en 16 imágenes individuales, y asignar cada una a su concepto tras revisión de coherencia.

## Por qué

David: *"tendremos 2 modos 1. wikimedia. con evaluacion de la imagen despues de generada. Otra sera prompt para generar cuadricula de 4x4. de todas las imagenes. Usaremos chatgpt. Luego haremos corte grafico de la grilla, es decir 4x4 => 16. Haremos n prompts para cubrir las imagenes a generar. Sera el agente o llm el que genere los prompts. Tendremos como mucho una funcion que nos devuelve los elementos a generar."*

Cierra el gap #1 de la Guía maestra (fallback de imagen para conceptos abstractos) y añade un control de calidad que hoy no existe: `cards build` acepta hoy la primera imagen de Wikimedia con licencia válida sin comprobar si de verdad representa el concepto.

## Principio de arquitectura (ya establecido en este repo, se reutiliza aquí)

Rust nunca juzga contenido semántico (igual que la redacción de `analyse` la hace el agente, no Rust — `research.md` §2 de la feature 003). Aquí aplica igual: **Rust solo hace lo mecánico** (listar qué falta, cortar la rejilla, persistir el asset elegido); **el agente/LLM hace lo semántico** (redactar los prompts de generación, y juzgar si una imagen — de Wikimedia o recortada — representa de verdad el concepto). La "validación de coherencia" nunca es un algoritmo de visión en Rust — es siempre el agente mirando la imagen y decidiendo.

## Diseño del flujo

### Modo 1 — Wikimedia con evaluación

Sin cambios en la búsqueda/descarga (`WikimediaCommonsProvider`, ya con resize a 800px). Se añade un paso de revisión **opcional, no bloqueante por defecto** (para no romper el uso automático que ya funciona en 126/127 tarjetas reales): un comando que lista las imágenes ya resueltas de Wikimedia de una sesión para que el agente las revise, y puede "des-resolver" una que no sea coherente (vuelve a `pending_image`, y esa URL/candidato concreto queda excluido para no repetirla en el siguiente intento).

### Modo 2 — Generación por rejilla (nuevo)

1. **Rust devuelve los elementos a generar** (la "función" que menciona David): comando que lista, para una sesión, las tarjetas `pending_image`, hasta un lote de 16 — puramente mecánico, sin generar texto.
2. **El agente redacta el prompt** (vía una Skill nueva, genérica en cuanto a mecánica pero con conocimiento de que es una rejilla 4x4): a partir del lote de hasta 16 conceptos, escribe un prompt pidiendo una imagen de rejilla 4x4 (16 casillas, orden de lectura: fila 1 izquierda→derecha, luego fila 2, etc.) donde cada casilla ilustra un concepto concreto. Si hay más de 16 pendientes, la Skill genera varios prompts (uno por lote de hasta 16).
3. **El usuario ejecuta el prompt** en ChatGPT (u otra herramienta externa) manualmente — fuera de LearnKit — y descarga la imagen de rejilla resultante.
4. **Rust recorta la rejilla**: comando que, dado el fichero de imagen y el número de filas/columnas, lo divide en N imágenes individuales y las deja en disco (sin asignarlas todavía a ninguna tarjeta) — mecánico, usa la librería `image` (nueva dependencia, MIT/Apache-2.0, revisar licencia igual que se hizo con las dependencias de la feature 002).
5. **El agente valida la coherencia** de cada recorte contra el concepto que le tocaba (mismo orden del prompt del paso 2) y decide, para cada uno, aceptarlo o no.
6. **Rust asigna** cada recorte aceptado como el asset de imagen de su tarjeta (mismo mecanismo de `Asset`/licencia que ya existe, con un origen distinto de "fetched"/"generated" para audio). Los recortes no aceptados simplemente no se asignan — la tarjeta sigue `pending_image` para un intento posterior (no hace falta un comando de "rechazar" explícito).

## Alcance

- Nuevo módulo/comandos en `learnkit-media` + `learnkit-cli`:
  - `learnkit cards image-batch --session <id> [--limit 16] [--json]` — lista tarjetas `pending_image` (id, texto/consulta) hasta el límite.
  - `learnkit cards image-grid crop --file <rejilla.png> --rows <R> --cols <C> --out-dir <dir> [--json]` — recorta en R×C imágenes, las escribe en `<dir>`, devuelve la lista de ficheros en orden de lectura.
  - `learnkit cards image-grid assign --session <id> --item <learning_item_id> --file <recorte.png> [--json]` — asigna UN recorte ya aprobado como imagen de la tarjeta de ese elemento (nuevo `AssetOrigin` para distinguir su procedencia — imagen generada por rejilla, no Wikimedia ni suministrada).
  - `learnkit cards image-review --session <id> [--json]` — lista las imágenes de Wikimedia ya resueltas de la sesión, para que el agente las revise.
  - `learnkit cards image-reject --session <id> --item <learning_item_id> --reason <...> [--json]` — revierte una imagen de Wikimedia ya asignada a `pending_image`, registrando el motivo (evita reproponerla sin más contexto).
- Nueva Skill de agente `learnkit-image-prompts` (o ampliar una existente): redacta los prompts de rejilla a partir de `image-batch`, y hace la revisión de coherencia (tanto de Wikimedia vía `image-review` como de los recortes vía `image-grid crop`).
- Nueva dependencia: crate `image` (recorte de imágenes). Verificar licencia (MIT/Apache-2.0 esperado) igual que se hizo con las dependencias de la feature 002.
- Documentar el flujo completo en el manual.

**Fuera de alcance explícito**: cualquier integración directa con una API de generación de imágenes (el usuario ejecuta el prompt manualmente, LearnKit nunca llama a ChatGPT); generación de imagen individual (no en rejilla); cualquier cambio a la plantilla de gramática/conocimiento general (feature separada).

## Restricciones

- Rust nunca decide si una imagen es coherente — solo el agente. Ningún comando nuevo intenta "detectar" contenido de imagen mediante heurística o ML en Rust.
- Reutilizar el `Asset`/`asset::register_or_reuse` ya existente en `learnkit-media/src/asset.rs` en vez de duplicar su lógica de deduplicación/validación.
- Seguir las convenciones ya establecidas (`LearnKitError`, `Envelope`, `resolve_session`, estilo de test existente en `crates/learnkit-media`/`crates/learnkit-cli/tests/cards_test.rs`).
- Verificar la licencia de cualquier dependencia nueva antes de comprometerla (Principio de arquitectura, "sin AGPL").

## Checklist

- [x] T0 Resize de imagen de Wikimedia a 800px (`iiurlwidth`) — gap #3 de la Guía maestra, hecho como quick-win antes de esta feature. (`crates/learnkit-media/src/image.rs`)
- [x] T1 Añadir dependencia `image` al workspace; verificar licencia.
- [x] T2 `learnkit cards image-batch` — lista tarjetas `pending_image` de una sesión, hasta un límite.
- [x] T3 `learnkit cards image-grid crop` — recorta una rejilla R×C en N imágenes individuales.
- [x] T4 Nuevo `AssetOrigin` (o variante) para "imagen generada por rejilla, aprobada manualmente" — distinto de `fetched` (Wikimedia) y del `generated` de audio.
- [x] T5 `learnkit cards image-grid assign` — asigna un recorte aprobado a la tarjeta de un elemento.
- [x] T6 `learnkit cards image-review` / `learnkit cards image-reject` — listar y revertir imágenes de Wikimedia ya resueltas.
- [x] T7 Skill de agente `learnkit-image-prompts`: redacta los prompts de rejilla a partir de `image-batch`, hace la revisión de coherencia (rejilla y Wikimedia).
- [x] T8 Tests de integración/unitarios para T2-T5 y T6 (mecánicos: listado, recorte, asignación, revisión/rechazo de Wikimedia — sin depender de ningún LLM real).
- [x] T9 Documentar el flujo completo en `docs/manual.md`/`.html`.

## Criterios de aceptación

- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets` en verde.
- Commit(s) de trabajo en la rama actual, binario redesplegado tras cada commit.
- El recorte de una rejilla 4x4 real produce 16 imágenes en el orden de lectura correcto (verificado con una imagen de prueba generada en el propio test, no con una imagen real de ChatGPT).

## Progreso

2026-09-28: T1-T5 y T8 implementados (Modo 2, mecánico completo salvo T6/T7/T9).

- **T1**: `image = "0.25"` añadido a `[workspace.dependencies]` y a
  `crates/learnkit-media/Cargo.toml`. Licencia confirmada MIT/Apache-2.0 dual
  (crates.io/`image` — mismo patrón dual que el resto de dependencias de este
  repo, no AGPL). Compila sin conflictos junto a `reqwest`/`rusqlite`/`zip`
  ya existentes; trae bastantes transitivas de codecs (png, jpeg, gif, tiff,
  webp, etc.) por las *default features* — esperado y aceptado en el alcance.
- **T2**: `learnkit cards image-batch --session <id> [--limit 16] [--json]`
  en `crates/learnkit-cli/src/commands/cards_image.rs`. Puramente de
  lectura: `load_all` + `completeness()` para filtrar `PendingImage`, cruza
  con `learning_item::load_all_vocabulary_items` para el título. Orden
  determinista (cards ordenadas por id antes de aplicar el límite).
- **T3**: `learnkit cards image-grid crop --file <f> --rows <R> --cols <C>
  --out-dir <dir> [--json]`. Lógica de recorte pura y testable en
  `crates/learnkit-media/src/grid.rs::crop_grid` (usa `image::DynamicImage`),
  separada de la I/O (`write_crops`/`crop_file_to_dir`). Tamaño de celda por
  división entera truncada (`width/cols`, `height/rows`); el remanente (como
  mucho `cols-1`×`rows-1` px) se descarta en vez de reparirse de forma
  desigual — documentado en el código.
- **T4**: `AssetOrigin::GeneratedGrid { provider: String }` añadido en
  `crates/learnkit-media/src/asset.rs`, mismo patrón `#[serde(tag = "kind")]`
  que las variantes existentes (serializa como `{"kind":"generated_grid",
  "provider":"..."}`). Round-trip verificado con test dedicado.
- **T5**: `learnkit cards image-grid assign --session <id> --item <li_id>
  --file <crop.png> [--json]`. Registra el recorte vía
  `asset::register_or_reuse` (fingerprint = hash de contenido, origen
  `GeneratedGrid { provider: "chatgpt-grid-manual" }`), localiza la
  `CardDefinition` cuyo `learning_item_ids` contiene `--item` (falla
  explícito con `ExporterConstraint` si no existe) y reemplaza/añade el
  `Block::Image` del front.
- **T8**: unit tests en `learnkit-media/src/grid.rs` (recorte 4×4 sobre una
  imagen de prueba de 16 cuadrantes de color, verificación de orden de
  lectura por color de píxel + nombres de fichero zero-padded), en
  `learnkit-media/src/asset.rs` (round-trip de `GeneratedGrid`), en
  `learnkit-cli/src/commands/cards_image.rs` (helpers puros: reemplazo de
  bloque de imagen del front, detección de mime), e integración de extremo a
  extremo en `crates/learnkit-cli/tests/cards_image_test.rs` (nuevo) para
  `image-batch`/`image-grid crop`/`image-grid assign`.
- Verificación: `cargo build --workspace` y `cargo clippy --workspace
  --all-targets` en verde sin warnings; `cargo test --workspace` en verde
  (todas las suites `ok`, sin fallos).
- Sin commits creados todavía en el primer bloque de trabajo (T1-T5/T8,
  según instrucción explícita de no comprometer) ni `cargo install`
  ejecutado.

2026-09-28 (segundo bloque): T6, T7 y T9 implementados; feature completa.

- **T6**: `learnkit cards image-review --session <id> [--json]` y
  `learnkit cards image-reject --session <id> --item <li_id> --reason <r>
  [--json]`, en `crates/learnkit-cli/src/commands/cards_image.rs` (mismo
  fichero que T2-T5, wireado en `cards.rs` como dos nuevas variantes de
  `CardsAction`). `image-review` recorre las tarjetas de la sesión, mira el
  `asset_id` del bloque `Image` del anverso, carga el `Asset` y filtra por
  `origin == AssetOrigin::Fetched { .. }` (Wikimedia) — nunca lista imágenes
  `Supplied` ni `GeneratedGrid`; devuelve card_id, learning_item_id, título,
  asset_id, `path` (para poder abrir el fichero) y licencia/autor/URL de
  origen. `image-reject` localiza la tarjeta del `--item` dado (falla
  explícito con `ExporterConstraint`/exit 40 si no existe), limpia el
  `asset_id` del bloque `Image` del anverso a `None` (vuelve a
  `pending_image`; el fichero del `Asset` NUNCA se borra — otras tarjetas
  pueden seguir referenciándolo vía el dedup de `register_or_reuse`; falla
  explícito si no había nada que rechazar), guarda la tarjeta, y registra el
  rechazo.
  - **Registro del rechazo**: un fichero JSON por rechazo (nunca se
    sobrescribe uno anterior — historial completo), en
    `sessions/<session_id>/validation/rejected-images/rejection-NNN.json`
    (usa `SessionPaths::validation()`, ya existente; convención de
    "un fichero por entidad" ya usada en `learnkit-media::asset` para los
    metadatos de cada asset — se usó JSON, no YAML, para no añadir
    `serde_yaml` como dependencia nueva de `learnkit-cli`, que hoy no la
    tiene). Contenido: `learning_item_id`, `card_id`, `asset_id` (el
    rechazado), `reason` (el texto dado por el agente).
- **T7**: nueva Skill `crates/learnkit-agent/templates/skills/learnkit-image-prompts/SKILL.md`,
  registrada en `crates/learnkit-agent/src/templates.rs` exactamente igual
  que `learnkit-analyse` (una constante `include_str!` + una entrada más en
  el `Vec<AgentFileTemplate>` de `codex` y de `claude`). Instruye al agente
  a: 1) llamar a `image-batch` para obtener el lote (hasta 16, repetir por
  lotes si hay más); 2) redactar un único prompt de rejilla 4x4 en orden de
  lectura, sin texto/etiquetas dentro de la imagen; 3) pedir al humano que
  ejecute el prompt en ChatGPT y le devuelva la ruta del fichero resultante;
  4) llamar a `image-grid crop`; 5) mirar cada recorte y juzgar coherencia
  contra el concepto en la misma posición, asignando solo los coherentes
  vía `image-grid assign` y nunca los que no lo son; 6) el mismo principio
  para `image-review`/`image-reject` de Wikimedia. Deja explícito, igual que
  `learnkit-analyse`, que el CLI nunca juzga contenido — eso es siempre del
  agente, y nunca debe asignar/aceptar una imagen que no ha mirado.
  `install::tests::reinstalling_unchanged_files_is_idempotent` actualizado
  de 4 a 5 ficheros instalados (nueva Skill).
- **T9**: `docs/manual.md` y `docs/manual.html` — dos subsecciones nuevas
  añadidas al final de §8 "Generar tarjetas" ("Revisar la coherencia de las
  imágenes de Wikimedia" y "Generar imágenes por rejilla 4x4 para conceptos
  abstractos"), un párrafo nuevo en §12 mencionando la Skill
  `learnkit-image-prompts`, y 5 filas nuevas en la tabla "Referencia de
  comandos" de §13 (`image-batch`, `image-grid crop`, `image-grid assign`,
  `image-review`, `image-reject`). Ninguna sección se renumeró — mismo
  patrón que se usó para documentar `export study-guide` dentro de §9.
- Verificación final: `cargo build --workspace`, `cargo clippy --workspace
  --all-targets` (sin warnings) y `cargo test --workspace` en verde — 245
  tests pasados, 0 fallos, en las 11 crates del workspace (incluye los 3
  tests nuevos de T6 en `cards_image_test.rs`, ahora 8/8 en ese fichero).
- Sin commits creados en este segundo bloque tampoco (instrucción explícita
  del encargo) ni `cargo install` ejecutado — feature T0-T9 completa, lista
  para revisión/commit.

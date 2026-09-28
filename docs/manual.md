# Manual de uso de LearnKit

Guía práctica para usar LearnKit de principio a fin: desde inicializar un proyecto hasta tener un mazo de Anki, un examen y tu progreso por destreza. Todos los comandos de este manual están verificados contra el binario real (incluida una importación real en Anki Desktop).

## Índice

1. [Compilar el binario](#1-compilar-el-binario)
2. [Inicializar un proyecto](#2-inicializar-un-proyecto)
3. [Crear una sesión de clase](#3-crear-una-sesión-de-clase)
4. [Transcribir el audio](#4-transcribir-el-audio)
5. [Analizar la sesión (resumen, mapa mental, páginas)](#5-analizar-la-sesión-resumen-mapa-mental-páginas)
6. [Consolidar candidatos de vocabulario](#6-consolidar-candidatos-de-vocabulario)
7. [Añadir vocabulario](#7-añadir-vocabulario)
8. [Generar tarjetas](#8-generar-tarjetas)
9. [Exportar a Anki](#9-exportar-a-anki)
10. [Generar y hacer un examen](#10-generar-y-hacer-un-examen)
11. [Ver tu progreso](#11-ver-tu-progreso)
12. [Usar un agente (Claude/Codex) para sugerir vocabulario](#12-usar-un-agente-claudecodex-para-sugerir-vocabulario)
13. [Referencia de comandos](#13-referencia-de-comandos)
14. [Solución de problemas](#14-solución-de-problemas)
15. [Qué no hace (todavía) LearnKit](#15-qué-no-hace-todavía-learnkit)

---

## 1. Instalar el binario

```bash
cargo install --path crates/learnkit-cli
```

Esto compila e instala `learnkit` en `~/.cargo/bin` (en Windows: `%USERPROFILE%\.cargo\bin`), que normalmente ya está en tu `PATH` si instalaste Rust con `rustup`. A partir de aquí puedes llamar a `learnkit` desde cualquier carpeta, no solo desde dentro de este repositorio.

Si solo quieres compilarlo sin instalarlo (por ejemplo, para desarrollo), usa `cargo build --release`; el binario queda en `target/release/learnkit` (`.exe` en Windows) pero solo es accesible con la ruta completa.

Requisito: toolchain de Rust estable. Nada más es obligatorio para este primer paso — whisper.cpp, Piper y la red solo hacen falta para pasos concretos más adelante (ver [§14](#14-solución-de-problemas)).

## 2. Inicializar un proyecto

```bash
learnkit init --profile language --agents claude
```

- `--profile language`: activa el perfil de idiomas (vocabulario, IPA, variedad `en-GB` por defecto).
- `--agents claude`: instala la integración de Claude (`CLAUDE.md` + Skills). También puedes usar `codex`, o ambos separados por comas.

Si no pasas `--profile`/`--agents` y estás en una terminal interactiva, LearnKit te los pregunta con un menú. Comprueba el resultado:

```bash
learnkit status --json
```

## 3. Crear una sesión de clase

Cada sesión agrupa el material de una clase concreta.

```bash
learnkit session new "EOI — Unit 5 phrasal verbs"
```

Te devuelve un `session_id` (ejemplo: `eoi-unit-5-phrasal-verbs-a1b2c3d4`). Guárdalo, lo necesitas en casi todos los comandos siguientes.

Copia tus fuentes (notas, audio, imágenes) a la sesión e inventarialas:

```bash
learnkit ingest ./mis-notas.md ./clase.wav ./pizarra.jpg --session <session_id>
learnkit inventory --session <session_id>
```

`inventory` es idempotente: puedes repetirlo cuando quieras. Si cambias el contenido de una fuente y vuelves a inventariar, verás `updated` en vez de `unchanged`.

## 4. Transcribir el audio

**Opción A — tienes whisper.cpp instalado:**

```bash
learnkit transcribe --session <session_id>
```

**Opción B — no tienes whisper.cpp, pero ya tienes una transcripción** (hecha con Buzz, subtítulos exportados, etc.):

```bash
learnkit transcribe import ./transcripcion.srt --source <source_id> --session <session_id>
```

`<source_id>` es el id de la fuente de audio (aparece en la salida de `inventory --json`, campo `sources[].id`).

Puedes comprobar qué ve LearnKit de tu sesión (notas + segmentos de transcripción) con:

```bash
learnkit session show <session_id> --json
```

## 5. Analizar la sesión (resumen, mapa mental, páginas)

Antes de sacar vocabulario o tarjetas, LearnKit analiza la sesión: convierte
las notas y la transcripción en un **resumen**, un **mapa mental** de repaso
rápido y una o más **páginas de concepto**. LearnKit (el binario Rust) no
redacta nada por su cuenta — quien redacta es un agente (Claude/Codex) a
través de la Skill `learnkit-analyse`, que lee el material de la sesión, y
la CLI solo valida la estructura/trazabilidad de lo que confirmas y lo
persiste.

En la práctica, pídele a tu agente "analiza la sesión `<session_id>`" y deja
que confirme cada elemento con estos comandos (puedes ejecutarlos tú mismo
si prefieres redactar a mano):

```bash
learnkit analyse summary set --session <session_id> --file resumen.md
learnkit analyse mindmap set --session <session_id> --file mapa.md
learnkit analyse page add --session <session_id> --concept "layover" --file pagina-layover.md
```

Ejemplo real (fichero `resumen.md` con un resumen corto de una clase de
vocabulario de viaje):

```bash
$ learnkit analyse summary set --session vocabulario-de-viaje-ae7c5e71 --file resumen.md --json
{
  "ok": true,
  "code": "ANALYSE_SUMMARY_SET",
  "item_id": "summary",
  "state": "done",
  "already_done": false
}
```

- `summary set` produce el elemento `summary` del checklist de `analyse`: qué
  se trabajó, conceptos importantes, reglas, ejemplos a conservar, errores a
  repasar y vocabulario a estudiar.
- `mindmap set` produce `mindmap`: un esquema de repaso rápido, distinto en
  forma del resumen (LearnKit rechaza un mapa mental idéntico al resumen).
- `page add` produce un elemento `page-<n>` por cada página de concepto
  (una idea concentrada, trazable al material de origen); puedes llamarlo
  varias veces para añadir varias páginas.
- Si el material menciona un concepto pero no lo explica, el agente puede
  rellenar ese hueco con su propio conocimiento, pero debe declararlo con
  `--filled-gap "<concepto>:<nota>"` en `summary set` — nunca mezclarlo sin
  marcar con lo que sí viene del material original.
- Repetir un comando sin cambios en el material de origen no regenera nada
  (`already_done: true` en la respuesta) — es idempotente, igual que
  `cards build`.

Consulta el resultado con `learnkit status --session <session_id> --json`:
cada elemento del checklist de `analyse` aparece por separado, no como un
único estado agregado de la fase:

```json
{
  "checklist": [
    { "item_id": "summary", "state": "done" },
    { "item_id": "mindmap", "state": "done" },
    { "item_id": "page-1", "state": "done" }
  ],
  "phase": "analyse"
}
```

### Cuando falta material: `flag-pending` y `skip`

Si el material de la sesión no basta para completar un elemento concreto
(por ejemplo, no hay contenido suficiente para un mapa mental útil),
márcalo explícitamente en vez de inventar contenido pobre:

```bash
learnkit analyse flag-pending mindmap --session <session_id> --reason "el material no tiene contenido suficiente para un mapa mental util"
```

El elemento pasa a `pending_user_decision` con el motivo guardado, y el
resto de elementos (por ejemplo `summary`) sigue su curso normal en
paralelo. Puedes resolverlo de dos formas:

- **Aportando el material que faltaba** y confirmando normalmente:
  `learnkit analyse mindmap set --session <session_id> --file mapa-completo.md`
  — pasa a `done` y el estado pendiente desaparece.
- **Omitiéndolo explícitamente**, cuando de verdad no aplica a esa sesión:

```bash
learnkit analyse skip mindmap --session <session_id> --reason "esta clase no tiene contenido jerarquizable"
```

`flag-pending` y `skip` toman el `item_id` (`summary`, `mindmap` o
`page-<n>`) como argumento **después** del verbo, no como subcomando de
`summary`/`mindmap`/`page`. Una vez resuelto (confirmado o `skip`), deja de
bloquear los pasos posteriores (como `consolidate`, ver [§6](#6-consolidar-candidatos-de-vocabulario)) y no se vuelve a preguntar mientras el
material no cambie.

## 6. Consolidar candidatos de vocabulario

Una vez analizada la sesión, `consolidate` deriva candidatos de vocabulario
del resumen y las páginas de concepto, y comprueba cada uno contra el
vocabulario ya persistido (mismo mecanismo de deduplicación por lema que
`learn vocabulary add`, [§7](#7-añadir-vocabulario)) para no repetir lo que ya tienes registrado
de sesiones anteriores.

```bash
$ learnkit consolidate --session vocabulario-de-viaje-ae7c5e71 --json
{
  "ok": true,
  "code": "CONSOLIDATE_DONE",
  "candidates": [
    { "id": "cand-1", "text": "vocabulario", "already_exists": false,
      "source_ref": { "origin": "summary", "locator": "vocabulario" } },
    { "id": "cand-4", "text": "layover", "already_exists": true,
      "existing_vocabulary_id": "vocab-en-layover-7f3bf4",
      "source_ref": { "origin": "summary", "locator": "layover" } },
    { "id": "cand-5", "text": "boarding", "already_exists": false,
      "source_ref": { "origin": "summary", "locator": "boarding" } }
  ]
}
```

- `already_exists: true` + `existing_vocabulary_id` significa que ese
  candidato ya existe como entrada de vocabulario persistida (de esta sesión
  o de una anterior) — **no se elimina del listado**, solo se marca, para
  que sepas que ya lo tienes.
- Cada candidato conserva `source_ref` (de qué elemento del análisis viene:
  `summary`, `mindmap` o `page-<n>`, y su localizador) — trazabilidad hasta
  el análisis de la sesión.
- `learnkit consolidate list --session <session_id> --json` reconsulta los
  candidatos ya persistidos para la sesión, sin volver a derivarlos.
- Esta fase (por ahora) solo produce candidatos de **vocabulario** — no
  preguntas de gramática ni de conocimiento general (ver [§15](#15-qué-no-hace-todavía-learnkit)).

Si algún elemento de `analyse` sigue `pending_user_decision`, `consolidate`
se bloquea explícitamente en vez de consolidar con datos incompletos:

```bash
$ learnkit consolidate --session clase-con-poco-material-d0bde9cd --json
{
  "ok": false,
  "code": "BLOCKED",
  "pending_items": [
    { "item_id": "mindmap", "reason": "el material no tiene contenido suficiente para un mapa mental util" }
  ]
}
$ echo $?
20
```

El código de salida es `20`, y `pending_items` indica exactamente qué
elemento de `analyse` falta por resolver (con `flag-pending`/`mindmap set` o
`skip`, ver [§5](#5-analizar-la-sesión-resumen-mapa-mental-páginas)).

## 7. Añadir vocabulario

Confirma cada palabra o expresión que quieras estudiar:

```bash
learnkit learn vocabulary add \
  --session <session_id> \
  --lemma "get away with" \
  --sense "hacer algo malo sin ser castigado" \
  --source <source_id> \
  --locator "segment:00:02:30" \
  --suggested-by manual
```

- `--source` debe ser un id que exista en el inventario de esa sesión (LearnKit lo valida y rechaza typos).
- Si la misma expresión ya existe de una sesión anterior, se reutiliza — no se duplica.
- `--suggested-by agent` en vez de `manual` si viene de una sugerencia de tu agente (ver [§12](#12-usar-un-agente-claudecodex-para-sugerir-vocabulario)).

## 8. Generar tarjetas

```bash
learnkit cards build --session <session_id>
```

Para cada palabra sin imagen/audio propio, LearnKit:
- busca una imagen libre en Wikimedia Commons (con licencia y atribución guardadas),
- genera audio de pronunciación con Piper (si lo tienes instalado).

Comprueba el resultado:

```bash
learnkit cards validate --session <session_id> --json
```

Cada tarjeta aparece como `complete`, `pending_image` o `pending_audio`. Una tarjeta `pending_*` normalmente significa que no había una imagen de Wikimedia con licencia reutilizable para esa expresión (frecuente en modismos como "get away with") o que Piper no está instalado — puedes suministrar tú mismo el recurso, o dejarla así (no bloquea al resto).

## 9. Exportar a Anki

```bash
learnkit export anki --session <session_id> --out dist/mi-mazo.apkg
```

Si alguna tarjeta no está `complete`, el comando falla explícitamente (código de salida `40`) indicando exactamente cuál y qué le falta — nunca genera un `.apkg` a medias.

Importa `dist/mi-mazo.apkg` en Anki Desktop (`Archivo → Importar...`). **Verificado**: se importa correctamente, con imagen y audio en el lado que corresponda.

## 10. Generar y hacer un examen

```bash
learnkit assessment build --session <session_id>
learnkit export exam --assessment <assessment_id> --session <session_id> --out dist/examen.html
```

`<assessment_id>` te lo devuelve `assessment build --json` (campo `assessment_id`).

Abre `dist/examen.html` en tu navegador (no necesita servidor ni conexión). Al terminar, pulsa "Finish exam" y "Download results" — se descarga un `results.json`.

Importa esos resultados:

```bash
learnkit attempt import ./results.json --assessment <assessment_id> --session <session_id>
```

Puedes repetir el examen y reimportar tantas veces como quieras — nunca se sobrescriben intentos anteriores.

## 11. Ver tu progreso

```bash
learnkit progress
learnkit progress --skill listening
```

Muestra precisión y número de intentos, agregados por elemento de vocabulario y por destreza (`recognition`/`production`/`listening`). Se recalcula siempre desde el historial real de intentos, nunca desde un valor guardado.

## 12. Usar un agente (Claude/Codex) para sugerir vocabulario

Si instalaste la integración de Claude o Codex (`--agents` en el paso 2), tu agente tiene una Skill (`learnkit-language`) que:

1. Lee `learnkit session show <session_id> --json` (tus notas + segmentos de transcripción).
2. Te propone candidatos de vocabulario con su significado.
3. **Nunca** los guarda por su cuenta — solo ejecuta `learn vocabulary add` cuando tú confirmas cada uno.

En la práctica: simplemente pídele a tu agente "sugiéreme vocabulario de la sesión `<session_id>`" y sigue la conversación.

`--agents claude`/`codex` también instala una segunda Skill, `learnkit-analyse`
(ver [§5](#5-analizar-la-sesión-resumen-mapa-mental-páginas)), independiente de la anterior y no específica de idiomas: lee el
material de la sesión, redacta el resumen/mapa mental/páginas de concepto, y
los confirma con los comandos `learnkit analyse ...`. Cuando el material
tiene un hueco (menciona algo sin explicarlo), lo rellena con su propio
conocimiento pero lo declara con `--filled-gap`; cuando el material
simplemente no basta para un elemento, usa `flag-pending` en vez de
inventar contenido para que quedes tú quien decida (completarlo o `skip`).
Pídele a tu agente "analiza la sesión `<session_id>`" para arrancar este
flujo.

## 13. Referencia de comandos

| Comando | Qué hace |
|---------|----------|
| `learnkit init [PATH] [--profile <id>] [--agents <ids>] [--shell <sh\|ps>] [--json]` | Inicializa (o reafirma) un proyecto. |
| `learnkit agent install <codex\|claude> [--path <P>]` | Instala/reinstala la integración de un agente. |
| `learnkit status [--session <id>] [--json]` | Estado del proyecto (y de las fases de una sesión, si se indica). |
| `learnkit session new "<título>" [--profile <id>]` | Crea una sesión. |
| `learnkit session list` / `learnkit session show <id>` | Lista o muestra el detalle de una sesión. |
| `learnkit ingest <ficheros...> --session <id>` | Copia fuentes a la sesión. |
| `learnkit inventory --session <id>` | Inventaría (tipo, tamaño, hash) las fuentes de la sesión. |
| `learnkit transcribe --session <id> [--source <id>] [--model <path>]` | Transcribe audio con whisper.cpp. |
| `learnkit transcribe import <fichero> --source <id> --session <id>` | Importa una transcripción existente. |
| `learnkit analyse summary set --session <id> --file <f> [--filled-gap "<concepto>:<nota>"] [--force]` | Confirma el elemento `summary` de `analyse`. |
| `learnkit analyse mindmap set --session <id> --file <f> [--force]` | Confirma el elemento `mindmap` de `analyse`. |
| `learnkit analyse page add --session <id> --concept "<c>" --file <f>` | Añade y confirma un elemento `page-<n>` de `analyse`. |
| `learnkit analyse flag-pending <item_id> --session <id> --reason "<r>"` | Marca un elemento (`summary`/`mindmap`/`page-<n>`) como `pending_user_decision`. |
| `learnkit analyse skip <item_id> --session <id> --reason "<r>"` | Omite explícitamente un elemento pendiente, sin confirmar contenido real. |
| `learnkit consolidate --session <id> [--json]` | Deriva y deduplica candidatos de vocabulario a partir del análisis. |
| `learnkit consolidate list --session <id> [--json]` | Lista los candidatos ya persistidos de la sesión. |
| `learnkit learn vocabulary add --session <id> --lemma "<t>" --sense "<s>" --source <id> [--locator <l>] [--suggested-by agent\|manual]` | Confirma una entrada de vocabulario. |
| `learnkit cards build --session <id> [--template <id>]` | Genera tarjetas a partir del vocabulario. |
| `learnkit cards validate --session <id> [--json]` | Comprueba qué tarjetas están completas. |
| `learnkit export anki --session <id> --out <fichero.apkg>` | Exporta el mazo. |
| `learnkit assessment build --session <id>` | Genera el banco de preguntas (recognition/production/listening). |
| `learnkit export exam --assessment <id> --session <id> --out <fichero.html>` | Genera el examen HTML. |
| `learnkit attempt import <resultados.json> --assessment <id> --session <id>` | Importa los resultados de un examen. |
| `learnkit progress [--skill <s>] [--json]` | Muestra el progreso agregado. |

Todos los comandos aceptan `--json` para salida estructurada (pensada para agentes/scripts) y `--path <P>` para operar sobre un proyecto que no es el directorio actual.

## 14. Solución de problemas

| Situación | Qué significa | Qué hacer |
|-----------|----------------|-----------|
| `transcribe` falla con "tool not found" | No tienes `whisper-cli` (whisper.cpp) instalado o no está en el `PATH`. | Instálalo, o usa `transcribe import` con una transcripción ya hecha. Exit code `30`. |
| `cards build` deja audio sin generar | No tienes `piper` instalado. | Instálalo con una voz `en-GB`, o suministra tú el audio. No bloquea el resto de la tarjeta si el audio es opcional en la plantilla. |
| Una tarjeta queda `pending_image` | Wikimedia Commons no devolvió ninguna imagen con licencia reutilizable (frecuente en modismos/expresiones abstractas). | Suministra tú una imagen, o acepta que esa tarjeta no tenga imagen. |
| `export anki` falla con exit code `40` | Alguna tarjeta no está `complete`. | El mensaje indica exactamente cuál y qué le falta; corrígela o exporta solo las que sí lo están. |
| Sin conexión a internet | `cards build` no puede buscar en Wikimedia Commons. | Es el único punto de red de toda la herramienta; todo lo demás funciona offline. El fallo se trata como "sin imagen encontrada", no como error fatal. |

## 15. Qué no hace (todavía) LearnKit

- Exportar a Kahoot o Blooket (previsto en el roadmap, no implementado).
- Un comando `learnkit doctor` que compruebe de antemano qué herramientas externas tienes instaladas.
- Un `learnkit run <fase>` / `learnkit validate` genéricos — cada comando (`inventory`, `transcribe`, `cards build`...) valida sus propias dependencias directamente.
- Perfiles no lingüísticos (geografía, historia, Godot) — el perfil `language-en` es el único implementado por ahora. `analyse` (resumen/mapa mental/páginas) sí es genérico y no depende de idiomas, pero `consolidate` (ver siguiente punto) sigue acotado a vocabulario.
- Verificación real de importación en TAO/QTI, Kahoot o Blooket — solo Anki Desktop está verificado con datos reales.
- `consolidate` solo produce y deduplica candidatos de **vocabulario**; generalizarlo a preguntas de gramática o de conocimiento general (con su propia clave de deduplicación) queda para una feature posterior, apoyada en la plantilla de pregunta/respuesta+explicación todavía no implementada.

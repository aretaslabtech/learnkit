# Manual de uso de LearnKit

Guía práctica para usar LearnKit de principio a fin: desde inicializar un proyecto hasta tener un mazo de Anki, un examen y tu progreso por destreza. Todos los comandos de este manual están verificados contra el binario real (incluida una importación real en Anki Desktop).

## Índice

1. [Compilar el binario](#1-compilar-el-binario)
2. [Inicializar un proyecto](#2-inicializar-un-proyecto)
3. [Crear una sesión de clase](#3-crear-una-sesión-de-clase)
4. [Transcribir el audio](#4-transcribir-el-audio)
5. [Añadir vocabulario](#5-añadir-vocabulario)
6. [Generar tarjetas](#6-generar-tarjetas)
7. [Exportar a Anki](#7-exportar-a-anki)
8. [Generar y hacer un examen](#8-generar-y-hacer-un-examen)
9. [Ver tu progreso](#9-ver-tu-progreso)
10. [Usar un agente (Claude/Codex) para sugerir vocabulario](#10-usar-un-agente-claudecodex-para-sugerir-vocabulario)
11. [Referencia de comandos](#11-referencia-de-comandos)
12. [Solución de problemas](#12-solución-de-problemas)
13. [Qué no hace (todavía) LearnKit](#13-qué-no-hace-todavía-learnkit)

---

## 1. Instalar el binario

```bash
cargo install --path crates/learnkit-cli
```

Esto compila e instala `learnkit` en `~/.cargo/bin` (en Windows: `%USERPROFILE%\.cargo\bin`), que normalmente ya está en tu `PATH` si instalaste Rust con `rustup`. A partir de aquí puedes llamar a `learnkit` desde cualquier carpeta, no solo desde dentro de este repositorio.

Si solo quieres compilarlo sin instalarlo (por ejemplo, para desarrollo), usa `cargo build --release`; el binario queda en `target/release/learnkit` (`.exe` en Windows) pero solo es accesible con la ruta completa.

Requisito: toolchain de Rust estable. Nada más es obligatorio para este primer paso — whisper.cpp, Piper y la red solo hacen falta para pasos concretos más adelante (ver [§12](#12-solución-de-problemas)).

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

## 5. Añadir vocabulario

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
- `--suggested-by agent` en vez de `manual` si viene de una sugerencia de tu agente (ver [§10](#10-usar-un-agente-claudecodex-para-sugerir-vocabulario)).

## 6. Generar tarjetas

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

## 7. Exportar a Anki

```bash
learnkit export anki --session <session_id> --out dist/mi-mazo.apkg
```

Si alguna tarjeta no está `complete`, el comando falla explícitamente (código de salida `40`) indicando exactamente cuál y qué le falta — nunca genera un `.apkg` a medias.

Importa `dist/mi-mazo.apkg` en Anki Desktop (`Archivo → Importar...`). **Verificado**: se importa correctamente, con imagen y audio en el lado que corresponda.

## 8. Generar y hacer un examen

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

## 9. Ver tu progreso

```bash
learnkit progress
learnkit progress --skill listening
```

Muestra precisión y número de intentos, agregados por elemento de vocabulario y por destreza (`recognition`/`production`/`listening`). Se recalcula siempre desde el historial real de intentos, nunca desde un valor guardado.

## 10. Usar un agente (Claude/Codex) para sugerir vocabulario

Si instalaste la integración de Claude o Codex (`--agents` en el paso 2), tu agente tiene una Skill (`learnkit-language`) que:

1. Lee `learnkit session show <session_id> --json` (tus notas + segmentos de transcripción).
2. Te propone candidatos de vocabulario con su significado.
3. **Nunca** los guarda por su cuenta — solo ejecuta `learn vocabulary add` cuando tú confirmas cada uno.

En la práctica: simplemente pídele a tu agente "sugiéreme vocabulario de la sesión `<session_id>`" y sigue la conversación.

## 11. Referencia de comandos

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
| `learnkit learn vocabulary add --session <id> --lemma "<t>" --sense "<s>" --source <id> [--locator <l>] [--suggested-by agent\|manual]` | Confirma una entrada de vocabulario. |
| `learnkit cards build --session <id> [--template <id>]` | Genera tarjetas a partir del vocabulario. |
| `learnkit cards validate --session <id> [--json]` | Comprueba qué tarjetas están completas. |
| `learnkit export anki --session <id> --out <fichero.apkg>` | Exporta el mazo. |
| `learnkit assessment build --session <id>` | Genera el banco de preguntas (recognition/production/listening). |
| `learnkit export exam --assessment <id> --session <id> --out <fichero.html>` | Genera el examen HTML. |
| `learnkit attempt import <resultados.json> --assessment <id> --session <id>` | Importa los resultados de un examen. |
| `learnkit progress [--skill <s>] [--json]` | Muestra el progreso agregado. |

Todos los comandos aceptan `--json` para salida estructurada (pensada para agentes/scripts) y `--path <P>` para operar sobre un proyecto que no es el directorio actual.

## 12. Solución de problemas

| Situación | Qué significa | Qué hacer |
|-----------|----------------|-----------|
| `transcribe` falla con "tool not found" | No tienes `whisper-cli` (whisper.cpp) instalado o no está en el `PATH`. | Instálalo, o usa `transcribe import` con una transcripción ya hecha. Exit code `30`. |
| `cards build` deja audio sin generar | No tienes `piper` instalado. | Instálalo con una voz `en-GB`, o suministra tú el audio. No bloquea el resto de la tarjeta si el audio es opcional en la plantilla. |
| Una tarjeta queda `pending_image` | Wikimedia Commons no devolvió ninguna imagen con licencia reutilizable (frecuente en modismos/expresiones abstractas). | Suministra tú una imagen, o acepta que esa tarjeta no tenga imagen. |
| `export anki` falla con exit code `40` | Alguna tarjeta no está `complete`. | El mensaje indica exactamente cuál y qué le falta; corrígela o exporta solo las que sí lo están. |
| Sin conexión a internet | `cards build` no puede buscar en Wikimedia Commons. | Es el único punto de red de toda la herramienta; todo lo demás funciona offline. El fallo se trata como "sin imagen encontrada", no como error fatal. |

## 13. Qué no hace (todavía) LearnKit

- Exportar a Kahoot o Blooket (previsto en el roadmap, no implementado).
- Un comando `learnkit doctor` que compruebe de antemano qué herramientas externas tienes instaladas.
- Un `learnkit run <fase>` / `learnkit validate` genéricos — cada comando (`inventory`, `transcribe`, `cards build`...) valida sus propias dependencias directamente.
- Perfiles no lingüísticos (geografía, historia, Godot) — el perfil `language-en` es el único implementado por ahora.
- Verificación real de importación en TAO/QTI, Kahoot o Blooket — solo Anki Desktop está verificado con datos reales.

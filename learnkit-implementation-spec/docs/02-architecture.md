# 02 — Arquitectura

## 1. Vista general

```text
                   +--------------------+
                   |   Agent adapter    |
                   | Codex / Claude     |
                   +---------+----------+
                             |
                             v
+---------------------------------------------------------+
|                     learnkit CLI                        |
|  command routing | config | diagnostics | presentation  |
+--------------------------+------------------------------+
                           |
                           v
+---------------------------------------------------------+
|                     learnkit-core                       |
| session | workflow | guards | models | validation       |
+----------+-------------+--------------+-----------------+
           |             |              |
           v             v              v
      profiles       capabilities     exporters
                         |              |
       +-----------------+-----+   +----+----------------+
       | cards/media/assessment |   | Anki/Kahoot/Blooket|
       +------------------------+   +---------------------+
                           |
                           v
                    filesystem store
                           |
                       derived index
                         SQLite
```

## 2. Rust workspace propuesto

```text
learnkit/
├── Cargo.toml
├── crates/
│   ├── learnkit-cli/
│   ├── learnkit-core/
│   ├── learnkit-store/
│   ├── learnkit-workflow/
│   ├── learnkit-profile/
│   ├── learnkit-cards/
│   ├── learnkit-media/
│   ├── learnkit-transcription/
│   ├── learnkit-anki/
│   ├── learnkit-assessment/
│   ├── learnkit-export/
│   └── learnkit-agent/
├── profiles/
├── skills/
├── schemas/
├── tests/
└── docs/
```

No es obligatorio crear todos los crates el primer día. V1 puede empezar con 4-5 crates y separar cuando existan límites reales.

## 3. Responsabilidades

### `learnkit-cli`
- parsing de comandos;
- salida humana/JSON;
- códigos de salida;
- no contiene lógica de dominio.

### `learnkit-core`
- IDs;
- entidades comunes;
- errores;
- contratos entre componentes.

### `learnkit-store`
- lectura/escritura atómica;
- hashing;
- manifests;
- índice SQLite derivado;
- discovery de proyecto y sesiones.

### `learnkit-workflow`
- DAG de fases;
- dependencias;
- ejecución de validadores;
- guards;
- invalidación descendente.

### `learnkit-profile`
- carga de perfiles TOML/YAML;
- validadores específicos;
- schemas adicionales;
- plantillas.

### `learnkit-cards`
- modelos de Card Definition;
- templates;
- front/back;
- política de generación de tarjetas.

### `learnkit-media`
- asset registry;
- hashes/deduplicación;
- metadatos;
- transformación/normalización;
- providers de TTS/imagen como adaptadores.

### `learnkit-transcription`
- contrato `TranscriptionProvider`;
- normalización a un formato canónico de segmentos/timestamps;
- provider local `whisper.cpp` como implementación V1 preferida;
- provider opcional `faster-whisper` mediante worker aislado;
- importadores SRT/VTT/TXT/JSON para Buzz u otras herramientas;
- fingerprints de audio, modelo y configuración para reproducibilidad.

Los providers solo producen un `TranscriptDraft`; nunca modifican el estado de workflow ni marcan fases como válidas.

### `learnkit-anki`
- mapeo Card Definition -> note model;
- media bundle;
- `.apkg`;
- validación del paquete.

### `learnkit-assessment`
- banco de preguntas;
- exámenes;
- scoring;
- attempts;
- progress aggregation.

### `learnkit-export`
- Kahoot;
- Blooket;
- HTML/PDF en una fase posterior;
- futuros QTI/web packs;
- V2: adaptador QTI para TAO Community Edition.

### `learnkit-agent`
- generación/instalación de Skills;
- `AGENTS.md`/`CLAUDE.md` mínimos;
- plantillas de prompts;
- comandos para que los agentes llamen al CLI.

## 4. Dependencias Rust recomendadas

Preferencias, no contrato rígido:

- CLI: `clap`.
- serialización: `serde`, `serde_json`, `toml`.
- errores: `thiserror`, `anyhow` solo en borde CLI.
- tracing: `tracing`, `tracing-subscriber`.
- IDs: `uuid` o ULID.
- hashing: SHA-256.
- SQLite derivado: `rusqlite` o `sqlx` (elegir uno; para CLI síncrono, `rusqlite` mantiene V1 simple).
- CSV: `csv`.
- XLSX: crate de escritura XLSX seleccionada mediante spike técnico.
- ZIP/APKG: crate ZIP mantenida + SQLite.
- HTTP providers: `reqwest` cuando sea necesario.

No fijar versiones en la especificación; `Cargo.lock` fija las versiones reales del repositorio.

## 5. Python

Python **no forma parte del núcleo obligatorio**.

Puede utilizarse como worker opcional cuando una herramienta madura lo justifique, por ejemplo:
- `faster-whisper` para transcripción local si supera claramente al provider por defecto en un benchmark real;
- pipelines ML especializados;
- conversiones donde una biblioteca Python sea claramente superior.

Regla:

```text
Rust CLI -> typed provider interface -> external worker -> typed result
```

Nunca:

```text
Rust CLI -> scripts Python dispersos que modifican estado interno directamente
```

Los workers no pueden marcar fases como válidas; solo producen artefactos que Rust valida.

## 6. Plugins/providers

V1 debe usar interfaces internas, no un sistema dinámico de plugins complejo.

Ejemplo conceptual:

```rust
trait AudioProvider {
    fn synthesize(&self, request: AudioRequest) -> Result<AssetDraft>;
}

trait TranscriptionProvider {
    fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptDraft>;
}
```

Las implementaciones pueden ser:
- OpenAI/otro TTS;
- comando local;
- fichero suministrado por el usuario.

Para transcripción, V1 contempla:
- `whisper.cpp` como default local inicial;
- `faster-whisper` como provider opcional aislado;
- importación de transcripciones existentes (por ejemplo generadas con Buzz);
- providers cloud futuros bajo el mismo contrato.

La arquitectura queda preparada para plugins futuros sin obligarnos a resolver ABI dinámica en V1.

## 7. Frontera de evaluación V2 — TAO

TAO no forma parte del core ni de V1. En V2 se añade como sistema externo self-hosted:

```text
learnkit-assessment
        |
        v
learnkit-qti (V2)
        |
        v
TAO Community Edition
        |
        v
results adapter
        |
        v
Attempts -> Progress
```

Reglas:

1. TAO nunca será la fuente de verdad de las preguntas.
2. Los IDs internos de LearnKit deben sobrevivir al round-trip mediante metadata/fingerprints.
3. El exporter QTI debe ser determinista y validable.
4. El importer de resultados debe generar `Attempt` events, no escribir directamente métricas de progreso.
5. No introducir dependencias AGPL dentro del binario LearnKit salvo revisión explícita de licencia; preferir interoperabilidad por formatos/procesos externos.
6. QTI 2.2 es el baseline inicial de V2 por compatibilidad estable actual con TAO CE; el dominio no debe impedir un adapter QTI 3 posterior.

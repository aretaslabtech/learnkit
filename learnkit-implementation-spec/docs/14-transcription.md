# 14 — Transcripción

## 1. Objetivo

LearnKit debe procesar audio de clases o materiales sin quedar acoplado a Buzz, Whisper ni a un proveedor cloud. La transcripción es una **capability con providers sustituibles**.

```text
audio source
    |
    v
TranscriptionProvider
    |
    v
TranscriptDraft
    |
 normalization + validation
    |
    v
transcript.json  <- canónico
transcript.md    <- vista humana derivada
    |
    v
analyse / vocabulary / assessment / cards
```

Las fases posteriores nunca consumen formatos específicos de Whisper/Buzz.

## 2. Provider V1

### Default inicial: `whisper.cpp`

Motivos de diseño:
- ejecución local;
- fácil de invocar desde un binario Rust mediante adapter/command provider;
- evita hacer de Python una dependencia obligatoria;
- permite distribuir LearnKit manteniendo la orquestación en Rust.

"Default inicial" no significa "ganador definitivo". El benchmark con audios reales puede cambiar la recomendación.

### Provider opcional: `faster-whisper`

Se ejecuta como worker aislado. Python pertenece al provider, no a LearnKit Core.

Contrato:

```text
Rust -> request JSON -> worker -> result JSON -> Rust validation
```

El worker no puede:
- cambiar estados de sesión;
- marcar fases como `passed`;
- escribir fuera de su directorio temporal/output autorizado.

### Buzz/manual

Buzz sigue siendo útil para:
- transcribir manualmente fuera del pipeline;
- revisar/corregir audios difíciles;
- comparar resultados de engines.

LearnKit debe poder importar una transcripción existente y asociarla a un `Source`, evitando trabajo duplicado.

## 3. Formato canónico

Ejemplo simplificado:

```json
{
  "schema_version": 1,
  "transcript_id": "tr_...",
  "source_id": "src_...",
  "language": "en",
  "engine": {
    "provider": "whisper-cpp",
    "model": "large-v3",
    "config_fingerprint": "sha256:..."
  },
  "source_fingerprint": "sha256:...",
  "segments": [
    {
      "start_ms": 12420,
      "end_ms": 17810,
      "text": "..."
    }
  ]
}
```

Campos futuros permitidos sin romper el modelo:
- word timestamps;
- speaker/diarisation;
- confidence;
- manual corrections;
- language switches.

## 4. Reproducibilidad e invalidación

El fingerprint de una transcripción debe depender, como mínimo, de:
- bytes/fingerprint del audio;
- provider;
- model;
- idioma/auto-detect;
- parámetros que puedan alterar el resultado.

Si cambia cualquiera de ellos, LearnKit marca como stale los artefactos dependientes hasta que sean regenerados o explícitamente reconciliados.

Una corrección manual debe crear una nueva revisión del transcript conservando procedencia.

## 5. Guards

Una fase que requiera transcripción solo pasa si:
- todos los sources de audio requeridos tienen transcript asociado;
- el transcript valida contra schema;
- `source_fingerprint` coincide;
- no está marcado como fallido/incompleto;
- se conserva engine/import provenance.

Ejemplo:

```text
$ learnkit analyse --session eoi-2026-09-23

BLOCKED: transcription requirements failed
  x src_audio_02: transcript missing
  x src_audio_03: source fingerprint changed
```

## 6. Benchmark antes de fijar provider recomendado

Corpus inicial: 2-3 audios reales de EOI, incluyendo al menos un fragmento con:
- conversación natural;
- profesor + alumnos;
- vocabulario objetivo/phrasal verbs;
- ruido o solapamientos razonables.

Comparar:
1. vocabulario objetivo correctamente capturado;
2. nombres propios;
3. omisiones/alucinaciones;
4. segmentación y timestamps;
5. comportamiento ante pausas/ruido/interrupciones;
6. velocidad;
7. RAM/VRAM y facilidad de instalación.

El resultado se guarda como artefacto versionable de ingeniería. No necesitamos perseguir una métrica académica perfecta en V1: necesitamos saber qué engine funciona mejor con **nuestro material real**.

## 7. CLI objetivo

```bash
learnkit transcribe --session <id>
learnkit transcribe --source <source-id> --provider whisper-cpp
learnkit transcribe --source <source-id> --provider faster-whisper
learnkit transcribe import transcript.vtt --source <source-id>
learnkit transcribe benchmark --corpus tests/fixtures/transcription/
```

## 8. Fuera de V1

No son requisitos para la primera release:
- diarización avanzada;
- edición de waveform;
- editor completo de subtítulos;
- streaming en tiempo real;
- traducción automática del transcript;
- provider cloud por defecto.

Se dejan extensiones en el schema para incorporarlos después.

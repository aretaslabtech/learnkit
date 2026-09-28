# Contract: traits de provider (internos, entre crates)

**Feature**: [spec.md](./spec.md) | **Data model**: [data-model.md](./data-model.md)

Esta feature introduce tres providers sustituibles, todos con la misma forma conceptual (`request -> Result<Draft>`, nunca marcan fases como válidas — Principio III/XII de la constitución). Se documentan aquí porque son el "API pública" entre crates de dominio y son el punto de extensión para providers futuros (p.ej. `faster-whisper`, un TTS cloud, u otro repositorio de imágenes).

## `TranscriptionProvider` (`learnkit-transcription`)

```text
trait TranscriptionProvider {
    fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptDraft, ProviderError>;
}
```

- `TranscriptionRequest`: ruta del audio, idioma esperado (opcional), configuración del provider.
- `TranscriptDraft`: segmentos + metadata del engine — **nunca** el formato nativo del binario invocado; la normalización a `transcript.json` (data-model → Transcript) ocurre siempre en `learnkit-transcription`, no en el binario externo.
- Implementación V1: `WhisperCppProvider` (subproceso, research.md §1). Los importadores (SRT/VTT/TXT/JSON) implementan un trait hermano `TranscriptImporter` con la misma salida `TranscriptDraft`, para que el resto del sistema no distinga "generado" de "importado" salvo en el campo `engine` (data-model → Transcript).

## `VoiceProvider` (`learnkit-media`)

```text
trait VoiceProvider {
    fn synthesize(&self, request: VoiceRequest) -> Result<AssetDraft, ProviderError>;
}
```

- `VoiceRequest`: `{ text, locale (en-GB por defecto), voice_policy, purpose: "pronunciation" }` — coincide con el ejemplo de "audio request" de `docs/07-anki-media.md §6`.
- `AssetDraft`: ruta del audio generado + metadata suficiente para calcular el fingerprint de deduplicación (`hash(texto_normalizado + locale + voice_policy)`).
- Implementación V1: `PiperVoiceProvider` (subproceso, research.md §3).

## `ImageProvider` (`learnkit-media`)

```text
trait ImageProvider {
    fn search(&self, request: ImageRequest) -> Result<Vec<ImageCandidate>, ProviderError>;
}
```

- `ImageRequest`: `{ query, min_license: "reuse-with-attribution" }`.
- `ImageCandidate`: `{ url, author, license: {name, url}, thumbnail_url }` — el caller (`learnkit-cards`/`learnkit-media`) decide cuál descargar y registrar como `Asset` (data-model → Asset, `origin.kind = "fetched"`); el provider nunca escribe el `Asset` directamente, solo devuelve candidatos.
- Implementación V1: `WikimediaCommonsProvider` (HTTP vía `reqwest`, research.md §4). Si no hay candidatos con licencia válida, devuelve `Ok(vec![])`, no un error — la ausencia de imagen adecuada es un resultado válido (FR-017e), no un fallo de provider.

## Regla común

Ningún provider (ni el subproceso externo que invoca, ni la llamada HTTP) puede:
- cambiar el estado de una fase del workflow;
- marcar una tarjeta/transcripción como `valid`;
- escribir fuera del directorio de la sesión/`assets` que le corresponde.

Los validadores de `learnkit-workflow`/`learnkit-cards` son siempre los que deciden si el resultado de un provider es aceptable (Hard Guards).

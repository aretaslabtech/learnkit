# Fixtures: eoi-sample

Fixtures sintéticos usados por los tests de integración de la feature
`002-english-eoi-flow` (ver `specs/002-english-eoi-flow/tasks.md` T003):

- `notes.md`: notas de clase de ejemplo (texto plano, no real).
- `class-audio.wav`: 1 segundo de silencio a 8kHz mono. Sirve solo para probar
  la *forma* del pipeline de inventario/transcripción (tipo, tamaño, hash,
  invalidación), no la calidad real de una transcripción.
- `whiteboard.png`: PNG 1x1 mínimo válido. Sirve solo para probar la *forma*
  del pipeline de assets (tipo, validación, dedup), no contenido visual real.

La validación con audio/imágenes reales de una clase (calidad de
transcripción, resultado real de Wikimedia Commons, importación en Anki
Desktop) se hace siguiendo `specs/002-english-eoi-flow/quickstart.md`, no con
estos fixtures.

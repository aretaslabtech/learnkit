# 13 — Decisiones arquitectónicas iniciales

## ADR-001 — CLI y core en Rust

**Decisión:** Rust es la implementación principal.

**Motivo:** binario distribuible, tipado fuerte, buen control de filesystem/concurrencia y menor dependencia de runtimes externos.

**Python:** solo worker/provider aislado cuando aporte una biblioteca claramente superior.

## ADR-002 — Filesystem como source of truth

**Decisión:** YAML/JSON/Markdown/assets son canónicos. SQLite es índice/cache reconstruible.

**Motivo:** Git, diffs, auditabilidad y supervivencia del conocimiento fuera de LearnKit.

## ADR-003 — Hard guards en CLI

**Decisión:** las dependencias se revalidan al ejecutar fases/exporters.

**Motivo:** Skills/prompts pueden omitirse; el binario debe impedir estados inválidos.

## ADR-004 — Learning Item genérico

**Decisión:** el core utiliza Learning Item y no Vocabulary como entidad universal.

**Motivo:** soportar geografía, historia y Godot sin semántica lingüística artificial.

## ADR-005 — Vocabulary como extensión de idioma

**Decisión:** Vocabulary Entry vive en el perfil/capability de idiomas.

**Motivo:** conservar riqueza lingüística sin contaminar el core.

## ADR-006 — Card Definition independiente de Anki

**Decisión:** Anki es exporter.

**Motivo:** evitar lock-in y reutilizar tarjetas en otros destinos.

## ADR-007 — Assets independientes

**Decisión:** imagen/audio/vídeo tienen identidad propia y se referencian desde cards/assessments.

**Motivo:** reutilización, deduplicación, validación y soporte natural de audio frontal/reverso.

## ADR-008 — Assessment independiente de Cards

**Decisión:** ambos se derivan de Learning Items, pero no uno del otro.

**Motivo:** memorización y evaluación tienen objetivos distintos.

## ADR-009 — Kahoot y Blooket como exporters

**Decisión:** el Question Bank es nuestro; se exporta a formatos soportados por las plataformas.

**Motivo:** portabilidad y cero dependencia de APIs privadas.

## ADR-010 — Agent Skills ligeros

**Decisión:** los agentes reciben Skills/instrucciones, pero el enforcement vive en Rust.

**Motivo:** mantener portabilidad Claude/Codex y reducir prompts gigantes.

## ADR-011 — Transcripción desacoplada mediante providers

**Decisión:** LearnKit define un contrato propio de transcripción. `whisper.cpp` será el default local inicial de V1; `faster-whisper` podrá instalarse como worker/provider opcional y las transcripciones existentes de Buzz podrán importarse.

**Motivo:** el conocimiento y el workflow no deben depender de un motor concreto. El CLI Rust conserva la orquestación y validación; los engines solo generan borradores normalizados.

**Regla de selección:** antes de declarar definitivo el provider recomendado para EOI, ejecutar un benchmark sobre audios reales y comparar calidad en vocabulario objetivo, phrasal verbs, nombres propios, segmentos, timestamps, velocidad y coste operativo.

## ADR-012 — TAO Community Edition como runtime de examen en V2

**Decisión:** V2 integrará TAO Community Edition directamente para evaluación formal self-hosted. TCExam queda fuera del roadmap.

**Motivo:** TAO aporta un runtime de evaluación maduro y abierto, mientras LearnKit mantiene propiedad del banco de preguntas, trazabilidad y progreso. Evitamos construir un segundo motor de examen complejo.

## ADR-013 — QTI como frontera de interoperabilidad de evaluación

**Decisión:** LearnKit usará QTI como formato de intercambio con TAO. V2 empezará con QTI 2.2 como baseline. El dominio V1 será QTI-friendly pero no QTI-shaped.

**Motivo:** mantener el core independiente del XML/versión concreta y permitir una transición futura a QTI 3 sin migrar las entidades canónicas.

**Regla:** no utilizar scraping para integrar TAO cuando exista una vía estándar QTI. Los resultados externos se normalizan a `Attempt` y `Progress` sigue siendo derivado.

## Decisiones V2 diferidas

- método exacto de importación de resultados desde TAO CE;
- perfil exacto de QTI 2.2 y subconjunto de interacciones soportadas en el primer release;
- estrategia de media packaging QTI;
- momento de activar QTI 3 tras soporte estable y pruebas de round-trip;
- despliegue TAO local permanente vs entorno on-demand.

## Decisiones generales diferidas

- librería Rust exacta para XLSX;
- implementación directa de `.apkg` vs helper transitorio;
- provider TTS por defecto;
- resultado del benchmark `whisper.cpp` vs `faster-whisper` para decidir recomendación por perfil/hardware;
- provider de imagen por defecto;
- política Git LFS;
- formato final de IDs (UUIDv7 vs ULID);
- HTML exam renderer exacto;
- vídeo;
- sync de resultados externos.

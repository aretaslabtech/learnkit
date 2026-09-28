# `analyse-sample` fixture

Fixture de material de clase con un hueco deliberado, para probar la
detección/relleno de huecos de la fase `analyse` (FR-006, historia de
usuario US2 de `specs/003-analyse-consolidate-checklist/spec.md`).

`notes.md` menciona un concepto — la regla del *present perfect of
unfinished time periods* — sin llegar a explicarlo: el propio texto dice
explícitamente que "no ha llegado a explicar en qué consiste esa regla".
Un resumen o mapa mental redactado a partir de este fichero debe, o bien
dejar ese hueco señalado como pendiente de decisión del usuario
(`analyse <item> flag-pending --reason ...`), o bien rellenarlo
explícitamente marcando esa nota como añadida por el sistema
(`--filled-gap "<concepto>:<nota>"`), nunca completarlo en silencio como si
viniera del material original.

Este fixture es solo para pruebas automatizadas con contenido fijo (mismo
patrón que el resto de tests de integración del proyecto, sin invocar
ningún LLM real). La validación con contenido real de clase, redactado por
un agente de verdad, se hace siguiendo los escenarios de
`specs/003-analyse-consolidate-checklist/quickstart.md`.

# 08 — Assessment, juegos y progreso

## 1. Separación conceptual

```text
Cards       -> memorisation/retrieval practice
Assessment  -> evidence of knowledge/skill
Games       -> presentation mode sobre Assessment Items
Progress    -> vista derivada de Attempts
```

Kahoot/Blooket no son el banco de preguntas. LearnKit lo es.

## 2. Question Bank

Los Assessment Items canónicos viven localmente y pueden usarse en:
- examen LearnKit;
- Kahoot;
- Blooket;
- QTI/TAO en V2;
- futuro tutor adaptativo.

## 3. Constraints por exporter

El exporter transforma solo cuando la transformación es segura.

Si una pregunta excede límites:

```text
EXPORT FAILED: kahoot
q-17: question length 147 > 120
```

No truncar silenciosamente.

## 4. Kahoot V1

Kahoot soporta actualmente importación mediante su plantilla XLSX para preguntas Quiz estándar.

V1:
- descargar/fijar una copia compatible de la template como fixture de test;
- rellenar filas sin alterar celdas estructurales;
- validar límites antes de escribir;
- generar `.xlsx` listo para importar.

Restricción conocida de la importación por spreadsheet: las imágenes no viajan en esa importación. Por ello LearnKit debe producir un `kahoot-media-manifest.md/json` opcional indicando qué imagen añadir manualmente a cada pregunta si se desea multimedia.

## 5. Blooket V1

Blooket soporta importación de preguntas mediante CSV/spreadsheet.

V1:
- exporter CSV;
- multiple choice;
- correct answer positions;
- time limits soportados;
- typing-answer solo cuando el modelo interno sea compatible.

Las imágenes que no viajen por el importador deben manejarse como media manifest, no mediante scraping o APIs privadas.

## 6. Examen LearnKit

Primera versión recomendada: HTML autocontenido.

Características:
- preguntas aleatorizables;
- autocorrección de tipos objetivos;
- score final;
- export de Attempt JSON;
- funciona localmente sin servidor.

Respuesta corta puede ser:
- exact/normalized match V1;
- revisión manual para casos abiertos.

## 7. Attempts

Nunca sobrescribir resultados. Añadir eventos.

```jsonl
{"attempt_id":"...","assessment_id":"...","item_id":"q1","correct":true,"score":1.0,"skill":"recognition","at":"..."}
```

## 8. Progress

V1 calcula métricas simples y transparentes:
- accuracy all-time;
- accuracy últimos N intentos;
- último intento;
- número de intentos;
- mastery por learning item;
- mastery por skill/tag.

No usar un score misterioso de IA.

Ejemplo:

```text
Océano Pacífico        94% (17 attempts)
Océano Índico          61% (11 attempts)

recognition            90%
location_to_name       66%
```

## 9. Weak-area selection

V1 puede generar un nuevo assessment ponderando:
- errores recientes;
- items con pocos intentos;
- skills débiles.

Algoritmo determinista documentado. No llamarlo “adaptive learning” avanzado todavía.

## 10. Resultados externos

V1 prioriza resultados del examen propio porque el esquema está bajo nuestro control.

Importadores posteriores:
- Kahoot XLSX report;
- Blooket report spreadsheet cuando esté disponible para el plan del usuario.

Los importadores deben mapear preguntas por IDs/fingerprints, nunca solo por posición.

## 11. Estrategia V2 — TAO Community Edition

V2 utilizará **TAO Community Edition** para los exámenes formales self-hosted. No se implementará TCExam.

Responsabilidades:

```text
LearnKit
- owns Question Bank
- owns Learning Item links
- owns provenance
- owns Attempt history
- owns Progress

TAO CE
- delivers formal assessment
- enforces exam runtime rules
- captures responses and runtime scoring
- produces interoperable/exportable results
```

### QTI

La integración se hará mediante QTI, no mediante scraping de UI ni APIs privadas no necesarias.

Baseline inicial para V2: **QTI 2.2**, por ser una versión soportada actualmente de forma estable por TAO CE. El adapter se diseñará detrás de una interfaz para añadir QTI 3 posteriormente.

El modelo V1 debe conservar ya:
- IDs estables de items y choices;
- respuesta correcta explícita;
- scoring;
- feedback;
- assets/media;
- metadata extensible;
- grouping/assessment identity;
- fingerprints.

### Resultados

Los resultados de TAO deben convertirse siempre a eventos `Attempt` canónicos. `Progress` continúa siendo una vista derivada.

### Hard guard V2

No se permitirá marcar una evaluación TAO como importada si:
- hay item IDs sin mapping;
- hay respuestas para una revisión/fingerprint distinto del assessment exportado;
- el scoring recibido no puede normalizarse sin pérdida;
- faltan resultados requeridos para cerrar el intento.

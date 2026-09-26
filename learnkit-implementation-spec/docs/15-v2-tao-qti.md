# 15 — V2: TAO Community Edition + QTI

Status: **planned / not part of V1 implementation gate**

## 1. Objetivo

Añadir exámenes formales self-hosted sin convertir LearnKit en un LMS ni construir un motor de evaluación complejo propio. TAO Community Edition será el runtime de examen; LearnKit conserva el conocimiento y el historial del alumno.

## 2. Decisión de producto

- Plataforma objetivo: **TAO Community Edition**.
- Integración primaria: **QTI**.
- Baseline inicial: **QTI 2.2**.
- QTI 3: preparado arquitectónicamente, activado solo tras soporte estable y round-trip probado.
- TCExam: no se implementará.
- TAO no será una dependencia del flujo cotidiano V1.

## 3. Ownership

```text
LearnKit owns
  Sources
  Learning Items
  Question Bank
  Assessment definitions
  Provenance
  Attempts
  Progress

TAO owns at runtime
  Delivery session
  Candidate UI
  Exam navigation/runtime policy
  Response capture
  Runtime scoring
```

## 4. Arquitectura

```text
Assessment
   |
   v
QTI Adapter
   |
   +--> package validation
   |
   v
QTI package
   |
   v
TAO Community Edition
   |
   v
results export/API-supported path
   |
   v
TAO Results Adapter
   |
   v
Attempt events
   |
   v
Progress views
```

## 5. Requisitos heredados desde V1

Cada Assessment Item debe disponer de:
- ID estable;
- type;
- prompt;
- IDs estables por choice/response;
- correct response;
- scoring/weight;
- feedback opcional;
- Learning Item links;
- source provenance;
- Asset references;
- extensible metadata;
- content fingerprint.

No modelar V1 copiando QTI XML. El adapter hará la traducción.

## 6. Primer subconjunto QTI V2

Implementar primero los tipos que ya existen en V1:
- multiple choice / single response;
- true/false;
- short answer cuando pueda representarse sin semántica ambigua.

Después ampliar según necesidad real. No intentar soportar todas las interacciones QTI en el primer release.

## 7. Export pipeline

```text
learnkit export qti
  -> validate assessment
  -> resolve assets
  -> map internal model to QTI
  -> build package
  -> schema/profile validation
  -> write export manifest
```

El manifest debe guardar:
- assessment ID + revision/fingerprint;
- QTI adapter version;
- target profile (`tao`);
- exported item IDs;
- mapping internal ID <-> external identifier;
- asset hashes.

## 8. Results pipeline

```text
TAO result
  -> parse
  -> resolve export manifest
  -> verify assessment fingerprint
  -> map item/response IDs
  -> normalize score
  -> append Attempt events
  -> rebuild Progress
```

No importar resultados por orden de pregunta. La identidad debe ser explícita.

## 9. Hard guards

Bloquear exportación si:
- existen items inválidos;
- falta un asset required;
- un tipo de pregunta no está soportado por el adapter;
- hay IDs duplicados;
- el scoring no puede mapearse de forma inequívoca.

Bloquear importación si:
- no existe manifest de exportación compatible;
- el assessment cambió después de exportarse;
- hay respuestas para IDs desconocidos;
- se perdería información de scoring necesaria.

## 10. Deployment

V2 debe proporcionar documentación reproducible para levantar TAO CE en local/contenedor. No es objetivo que `learnkit` embeba TAO ni distribuya su código como parte del binario.

Comando futuro de diagnóstico:

```bash
learnkit tao doctor
```

Debe comprobar únicamente configuración/conectividad/capacidades requeridas, sin convertir TAO en source of truth.

## 11. Acceptance test V2

El gate mínimo es un round-trip real:

1. generar un assessment de geografía;
2. exportar QTI 2.2;
3. importarlo en TAO CE;
4. verificar texto, choices, scoring, feedback y media;
5. completar un examen;
6. recuperar resultados;
7. importarlos en LearnKit;
8. generar Attempts;
9. comprobar Progress;
10. repetir con un assessment de idioma.

## 12. Evolución a QTI 3

QTI 3 se añadirá como otro adapter/profile, no como migración destructiva del dominio:

```text
Assessment Model
  +-- QTI 2.2 adapter
  +-- QTI 3 adapter
```

La activación requiere tests de interoperabilidad contra una versión estable de TAO CE que lo soporte oficialmente.

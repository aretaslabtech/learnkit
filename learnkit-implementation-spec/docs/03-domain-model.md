# 03 — Modelo de dominio

## 1. Session

Unidad de trabajo que agrupa fuentes, pipeline y outputs.

Campos mínimos:

```yaml
id: 01J...
title: "Océanos y continentes"
profile: geography
created_at: 2026-09-26T12:00:00Z
source_dir: input/
workflow: default
```

## 2. Source

Representa una entrada original.

```yaml
id: src-001
kind: pdf
path: input/geografia.pdf
sha256: ...
status: inventoried
metadata:
  pages: 18
```

Tipos V1:
- text/markdown;
- PDF;
- image;
- audio;
- generic binary (inventariado, quizá no procesable).

## 3. Learning Item

Entidad central y genérica.

```yaml
id: li-pacific-ocean
kind: concept
title: "Océano Pacífico"
summary: "El océano de mayor superficie de la Tierra."
tags: [geography, oceans]
sources:
  - source_id: src-001
    locator: "page:4"
mastery_dimensions:
  - recognition
  - location
```

`kind` es extensible: `concept`, `term`, `event`, `person`, `place`, `procedure`, `api`, etc.

## 4. Vocabulary Entry

Extensión del perfil de idiomas. No pertenece al core obligatorio.

```yaml
id: vocab-en-get-away-with
language: en
variety: en-GB
lemma: "get away with"
part_of_speech: phrasal_verb
ipa: "/ɡet əˈweɪ wɪð/"
senses:
  - gloss: "do something wrong without being punished"
examples:
  - text: "He thought he'd get away with cheating."
collocations: []
usage_notes: []
sources: []
```

Una Vocabulary Entry puede vincularse a uno o varios Learning Items y sobrevivir a múltiples sesiones.

## 5. Card Definition

La tarjeta es independiente del exporter Anki.

```yaml
id: card-001
learning_item_ids: [li-pacific-ocean]
template: image-to-name-v1
front:
  blocks:
    - type: image
      asset_id: asset-img-pacific
    - type: text
      value: "¿Qué océano es este?"
back:
  blocks:
    - type: text
      value: "Océano Pacífico"
tags: [geography, oceans]
```

Para idiomas:

```yaml
front:
  blocks:
    - type: audio
      asset_id: aud-sentence-001
back:
  blocks:
    - type: text
      value: "I've been meaning to call you."
    - type: audio
      asset_id: aud-expression-001
```

Audio frontal y reverso son completamente independientes.

## 6. Asset

```yaml
id: aud-expression-001
type: audio
path: assets/audio/aud-expression-001.mp3
sha256: ...
mime: audio/mpeg
origin:
  kind: generated
  provider: provider-id
metadata:
  language: en-GB
  transcript: "get away with something"
  purpose: pronunciation
validation:
  state: valid
```

Tipos previstos:
- image;
- audio;
- video (schema reservado, V1 puede no generarlo);
- attachment.

Los assets son reutilizables por múltiples cards/assessments.

## 7. Assessment Item

```yaml
id: q-001
learning_item_ids: [li-pacific-ocean]
type: multiple_choice
prompt: "¿Cuál es el océano de mayor superficie?"
options:
  - id: opt-atlantic
    text: "Atlantic"
  - id: opt-pacific
    text: "Pacific"
  - id: opt-indian
    text: "Indian"
  - id: opt-arctic
    text: "Arctic"
correct_response:
  option_ids: [opt-pacific]
scoring:
  max_score: 1.0
difficulty: medium
skills: [recognition]
feedback:
  correct: "Correcto."
  incorrect: "Revisa la superficie relativa de los océanos."
metadata:
  external_ids: {}
sources: []
```

Diseño QTI-friendly desde V1:
- ID estable por item;
- prompt separado de las respuestas;
- choices con identidad estable, no solo posición;
- correct response explícita;
- scoring explícito;
- feedback opcional;
- media referenciada por Asset ID;
- metadata extensible para round-trip con plataformas externas.

Tipos V1 recomendados:
- multiple_choice;
- true_false;
- short_answer.

## 8. Assessment

Colección versionada de items con política de scoring.

```yaml
id: exam-geography-001
title: "Océanos y continentes — control 1"
items: [q-001, q-002]
scoring:
  kind: percent
pass_threshold: 0.70
delivery:
  shuffle_items: false
  shuffle_choices: false
metadata:
  external_ids: {}
```

## 9. Attempt

Evento inmutable de respuesta.

```json
{"attempt_id":"a1","assessment_id":"exam-geography-001","item_id":"q-001","at":"...","correct":true,"score":1.0,"duration_ms":8200}
```

V1 debe preferir un log append-only (`attempts.jsonl`) para facilitar Git, auditoría y reconstrucción.

## 10. Progress

No es la fuente primaria. Es una **vista derivada** de Attempts.

Se puede agregar por:
- learning item;
- tag/tema;
- skill (`recognition`, `production`, `listening`, etc.);
- ventana temporal.

## 11. Relaciones

```text
Source
  |
  v
Learning Item ----> Vocabulary Entry (solo perfiles de idioma)
  |  \
  |   +----> Assessment Item ---> Attempt ---> Progress view
  |
  +--------> Card Definition ---> Asset(s) ---> Anki exporter
```

## 12. Assessment interoperability

El modelo de dominio no debe copiar el XML de QTI. LearnKit mantiene un modelo simple y tipado y utiliza adapters:

```text
LearnKit Assessment Item <-> QTI adapter <-> TAO
```

Esto permite evolucionar de QTI 2.2 a QTI 3 sin migrar todo el dominio. Los elementos específicos de una plataforma deben permanecer en `metadata.external_ids` o en manifests de exportación, nunca convertirse en campos obligatorios del core.

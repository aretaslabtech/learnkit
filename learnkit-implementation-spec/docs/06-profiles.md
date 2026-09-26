# 06 — Profiles

## 1. Objetivo

El core no debe conocer “inglés”, “geografía” ni “Godot”. Un perfil declara:
- fases adicionales;
- schemas/extensiones;
- reglas de validación;
- templates de cards;
- mastery dimensions;
- prompts/skills de agente;
- exporters recomendados.

## 2. Perfil genérico

```toml
id = "generic"

[learning]
kinds = ["concept", "term", "procedure", "fact"]

[cards]
templates = ["qa-v1", "image-to-name-v1"]

[assessment]
types = ["multiple_choice", "true_false", "short_answer"]
```

## 3. Perfil `language-en`

Añade:
- Vocabulary Entry;
- `language=en`;
- variedad por proyecto/sesión, por ejemplo `en-GB`;
- IPA;
- meanings/senses;
- examples;
- collocations;
- usage notes;
- dimensions: recognition, production, listening, pronunciation, grammar.

Templates sugeridos:
- `word-to-meaning-v1`;
- `image-to-production-v1`;
- `audio-to-text-v1`;
- `sentence-listening-v1`;
- `expression-production-v1`.

El perfil decide si audio frontal/reverso es required/optional/disabled por template.

## 4. Perfil `geography`

Añade kinds/requisitos para:
- place;
- region;
- feature;
- map-location.

Dimensions:
- recognition;
- name_to_location;
- location_to_name;
- conceptual_relation.

Templates:
- `map-to-name-v1`;
- `name-to-map-v1`;
- `image-to-feature-v1`.

## 5. Perfil `history`

Dimensions posibles:
- chronology;
- cause_effect;
- people_events;
- source_interpretation.

Assets de imagen son útiles pero no necesariamente obligatorios.

## 6. Perfil `godot`

Extiende Learning Item con campos declarativos opcionales:
- engine_version;
- symbol;
- code_example;
- prerequisites.

Dimensions:
- concept_recognition;
- api_recall;
- code_application;
- debugging.

## 7. Herencia/composición

Evitar herencia compleja V1. Preferir composición explícita:

```toml
extends = ["generic", "language-base"]
```

Máximo 1-2 niveles. Si aparecen árboles profundos, rediseñar.

## 8. Regla de pureza

El core puede preguntar:

```text
profile.validate(entity)
profile.required_capabilities()
profile.card_templates()
```

No puede preguntar:

```text
if profile.name == "language-en" { ... }
```

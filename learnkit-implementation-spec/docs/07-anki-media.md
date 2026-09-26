# 07 — Anki y multimedia

## 1. Objetivo

Anki V1 debe soportar tarjetas reales con:
- texto;
- imagen;
- audio en frontal;
- audio en reverso;
- múltiples assets por lado si la template lo permite;
- tags;
- templates versionadas;
- `.apkg` autocontenido.

Vídeo se reserva en el modelo pero no es criterio de salida de V1.

## 2. Separación fundamental

```text
Learning Item
   -> Card Definition
      -> Asset references
         -> Anki renderer
            -> APKG
```

Nunca guardar HTML Anki como modelo primario.

## 3. Card blocks

Bloques V1:

```text
Text
Image(asset_id)
Audio(asset_id)
LineBreak
OptionalHtml (solo si está explícitamente permitido)
```

La template renderiza bloques a campos Anki.

## 4. Audio frontal y reverso

Debe ser una capacidad nativa:

```yaml
front:
  blocks:
    - type: audio
      asset_id: aud-listening-sentence
back:
  blocks:
    - type: text
      value: "I've been meaning to call you."
    - type: audio
      asset_id: aud-target-expression
```

No existe un único campo global `audio`.

## 5. Asset policies por template

```toml
[template.image-to-production-v1.front]
image = "required"
audio = "optional"

[template.image-to-production-v1.back]
audio = "required"
```

Valores:
- required;
- optional;
- disabled.

## 6. Generación y reutilización

El media engine recibe una intención, no “haz un mp3 cualquiera”.

Audio request:

```yaml
text: "get away with something"
language: en-GB
purpose: pronunciation
voice_policy: project-default
```

Fingerprint lógico:

```text
hash(normalized_text + locale + voice_policy + provider_options)
```

Permite reutilizar assets cuando sean equivalentes.

## 7. Validación de audio

Checks deterministas:
- fichero existe;
- tamaño > mínimo;
- MIME/formato permitido;
- decodificable;
- duración razonable;
- hash registrado.

Checks opcionales:
- transcript/reconocimiento coincide razonablemente;
- loudness normalization.

## 8. Validación de imagen

- formato permitido;
- dimensiones mínimas;
- decodificable;
- hash;
- no referencia rota.

El perfil/template decide si es required.

## 9. Anki package

El exporter genera un deck package `.apkg` con:
- colección/deck SQLite compatible;
- note/card models;
- templates HTML/CSS;
- media bundle y mapa de media.

La implementación debe aislar el formato Anki detrás de `learnkit-anki` porque el formato puede evolucionar.

## 10. Estrategia de implementación

Spike técnico inicial obligatorio:
1. generar `.apkg` mínimo en Rust;
2. importarlo en Anki Desktop;
3. comprobar una tarjeta con imagen;
4. comprobar audio frontal y reverso;
5. reexportar/importar para probar estabilidad de IDs;
6. automatizar smoke test donde sea posible.

Si implementar el formato actual directamente en Rust consume demasiado riesgo, V1 puede invocar un helper externo aislado **solo como renderer**, manteniendo Card Definition y validadores en Rust. Ese helper se considera temporal y reemplazable.

## 11. IDs y actualización

Las notas deben tener IDs/GUID deterministas derivados del `card_id`/`learning_item_id` para evitar duplicados al reimportar versiones posteriores.

## 12. Acceptance tests Anki

- tarjeta texto/texto;
- imagen frontal;
- audio frontal;
- audio reverso;
- audio distinto en ambos lados;
- tags;
- caracteres IPA;
- UTF-8 completo;
- 100 tarjetas + media;
- reexport sin duplicación inesperada.

# 12 — Escenarios de prueba end-to-end

Estos escenarios son acceptance tests del producto y de la arquitectura.

## A. Geografía — océanos y continentes

### Input
- 1 PDF;
- 2 imágenes/mapas;
- notas Markdown.

### Expected
- todos los sources inventariados;
- Learning Items para océanos/continentes;
- trazabilidad source -> item;
- al menos una card image-to-name;
- asset de imagen validado;
- `.apkg` importable;
- assessment multiple choice;
- Kahoot XLSX importable;
- Blooket CSV importable;
- examen HTML ejecutable;
- Attempt importado;
- progress muestra al menos item + skill.

### Guard test
Eliminar una imagen requerida y comprobar que Anki queda `BLOCKED`.

## B. Inglés — clase EOI

### Input
- notas Markdown;
- 1 audio de clase;
- 1 imagen/foto de pizarra o ficha.

### Expected
- audio inventariado y, mediante provider, transcrito;
- `transcript.json` registra engine/model/config, segmentos y timestamps;
- importar una transcripción Buzz/SRT/VTT equivalente evita retranscribir el audio;
- cambiar de provider no cambia el contrato consumido por fases posteriores;
- Vocabulary Entries para expresiones seleccionadas;
- `en-GB` y IPA donde el perfil lo exige;
- learning items lingüísticos;
- card de producción con imagen;
- card de listening con audio frontal;
- card con audio distinto en reverso;
- `.apkg` importable y media funcional;
- assessment dimensions: recognition/production/listening;
- progress por skill.

### Guard test
Quitar el audio del reverso en una template que lo marque `required`: export Anki debe fallar.

## C. Godot

### Input
- notas sobre Nodes/Scenes;
- extracto de documentación;
- ejemplo `.gd`.

### Expected
- Learning Items genéricos/técnicos;
- `engine_version` en metadata de perfil;
- cards de concepto sin multimedia obligatorio;
- questions sobre aplicación;
- ningún requisito de IPA/vocabulary aparece en el core.

### Guard test
Cambiar `engine_version` o el source y comprobar invalidación de artefactos dependientes.

## D. Cross-domain purity

Test de código/arquitectura:
- buscar dependencias desde `learnkit-core` hacia perfiles concretos;
- debe haber cero imports de módulos `language`, `geography`, `godot` en el core.

## E. Agent bypass

1. Escribir manualmente un manifest con `result=valid` aunque falte un asset.
2. Ejecutar export.
3. Expected: el CLI recalcula el guard y bloquea.

Este test demuestra que los hard guards son reales y no convenciones de prompt.

## F. Transcription provider portability

1. Seleccionar un fragmento de audio de clase con vocabulario real.
2. Ejecutarlo con `whisper.cpp`.
3. Ejecutarlo con `faster-whisper` cuando el provider esté instalado.
4. Importar una transcripción equivalente producida/revisada en Buzz.
5. Verificar que los tres caminos generan el mismo schema canónico.
6. Comparar errores relevantes: vocabulario objetivo, phrasal verbs, nombres propios, interrupciones, timestamps y omisiones.

Expected:
- ningún provider modifica estado interno directamente;
- los artefactos posteriores consumen el transcript canónico, no formatos específicos del engine;
- el benchmark deja resultados versionables;
- la elección del provider por defecto puede cambiar sin migrar sesiones existentes.

## G. V1 readiness for TAO/QTI (schema-only)

Este test no instala TAO ni genera QTI en V1. Comprueba que una pregunta V1 conserva:
- item ID estable;
- choice IDs estables;
- prompt;
- correct response;
- score/weight;
- feedback;
- asset references;
- metadata externa vacía/extensible.

Expected: serializar/deserializar no pierde ninguna de esas propiedades y ningún exporter V1 depende de la posición accidental de choices para identificar una respuesta.

## H. V2 — TAO/QTI round-trip (future acceptance test)

1. Levantar TAO Community Edition en un entorno reproducible.
2. Exportar un assessment LearnKit a QTI 2.2.
3. Importarlo en TAO.
4. Verificar items, respuestas, scoring, feedback y media.
5. Ejecutar un intento.
6. Exportar/recuperar resultados.
7. Importarlos en LearnKit.
8. Comprobar mapping a `Attempt` y actualización derivada de `Progress`.

Expected: ningún item queda huérfano y el round-trip mantiene identidad y scoring.

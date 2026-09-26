# External integration notes — verified 2026-09-26

Estas notas son deliberadamente separadas del diseño canónico porque los productos externos pueden cambiar.

## Kahoot

Fuente oficial:
https://support.kahoot.com/hc/en-us/articles/115002812547-How-to-import-questions-from-a-spreadsheet-to-your-kahoot

Verificado:
- importación por plantilla `.xlsx` oficial;
- solo preguntas Quiz estándar mediante spreadsheet;
- pregunta: máximo 120 caracteres;
- respuesta: máximo 75 caracteres;
- al menos 2 respuestas;
- tiempos permitidos: 5, 10, 20, 30, 60, 120 segundos;
- no se importan imágenes mediante spreadsheet.

Informes XLSX:
https://support.kahoot.com/hc/en-us/articles/360035547493-How-to-download-and-use-spreadsheet-reports

## Blooket

Fuente oficial:
https://help.blooket.com/hc/en-us/articles/16002377931543

Verificado:
- importación mediante CSV/spreadsheet template;
- soporta multiple choice;
- correct answers se expresan por posición;
- admite configuración de typing-answer en la plantilla documentada.

Reports:
https://help.blooket.com/hc/en-us/articles/16180020488727-How-to-Read-Blooket-Reports

## Anki

Manual oficial:
https://docs.ankiweb.net/exporting.html

Verificado:
- `.apkg` es el paquete de deck;
- los paquetes pueden incluir audio, imágenes y otro media.

## Agent Skills / Codex

OpenAI Skills:
https://developers.openai.com/api/docs/guides/tools-skills

Patrón verificado:
- Skill = directorio con `SKILL.md`;
- puede incluir references/scripts/assets;
- Codex usa Skills y `AGENTS.md` para workflows reutilizables.

## Claude Skills

Anthropic Skills best practices:
https://docs.claude.com/es/docs/agents-and-tools/agent-skills/best-practices

Patrón verificado:
- `SKILL.md` con name/description e instrucciones;
- referencias/scripts pueden mantenerse fuera del cuerpo principal;
- preferencia por validación determinista mediante scripts cuando proceda.

## Transcripción local

### Buzz
Repositorio:
https://github.com/chidiwilliams/buzz

Uso previsto en LearnKit:
- herramienta manual complementaria, no dependencia del core;
- LearnKit debe importar transcripciones existentes en formatos interoperables cuando estén disponibles;
- una transcripción importada se normaliza al mismo schema canónico que cualquier provider automático.

### whisper.cpp
Repositorio:
https://github.com/ggml-org/whisper.cpp

Uso previsto en LearnKit:
- provider local inicial de V1;
- invocado detrás de `TranscriptionProvider`;
- su output específico nunca se convierte en formato de dominio canónico.

### faster-whisper
Repositorio:
https://github.com/SYSTRAN/faster-whisper

Uso previsto en LearnKit:
- provider local opcional ejecutado como worker aislado;
- candidato a provider recomendado cuando un benchmark con material real demuestre una ventaja suficiente;
- la dependencia Python queda fuera del core Rust.

## Política de benchmark de transcripción

La selección de engine debe validarse con material real del perfil, no mediante una preferencia teórica. Para EOI se compararán al menos:
- conservación de vocabulario objetivo y phrasal verbs;
- nombres propios;
- omisiones/alucinaciones;
- timestamps/segmentación;
- ruido, pausas e interrupciones;
- velocidad y requisitos de hardware.


## TAO Community Edition — objetivo V2

Fuentes oficiales:
- https://www.taotesting.com/es/products/community-edition/
- https://www.taotesting.com/es/community/installation-guide/
- https://www.taotesting.com/es/products/
- https://userguide.taotesting.com/user-documentation/latest/public/tao-versions

Verificado 2026-09-26:
- TAO Community Edition es la edición open-source actual de TAO y sus componentes principales se publican bajo AGPLv3;
- puede desplegarse localmente mediante contenedor/Docker;
- TAO soporta import/export de items/tests QTI;
- TAO Core fue deprecado en enero de 2026 y sustituido por TAO Community Edition.

Estado QTI actual relevante para diseño:
- la información pública de la comunidad TAO indica soporte estable QTI 2.x (2.1/2.2);
- QTI 3 tiene soporte experimental y no debe asumirse como baseline hasta soporte oficial/estable.

Decisión LearnKit:
- V1: modelo Assessment QTI-friendly, sin integración TAO;
- V2: TAO CE + exporter QTI 2.2 + importer de resultados;
- QTI 3: adapter futuro tras validación real.
- TCExam: fuera del roadmap.

## 1EdTech QTI

Fuente oficial:
https://www.1edtech.org/standards/qti/index

Verificado:
- QTI estandariza intercambio de items, tests y datos de resultados entre herramientas de autoría, bancos de preguntas, plataformas de aprendizaje y motores de evaluación/scoring;
- QTI 3.0 es la versión moderna del estándar, pero LearnKit no confundirá la versión más reciente del estándar con la versión soportada de forma estable por el runtime TAO objetivo.

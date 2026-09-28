# Guía maestra de trabajo — Proyecto de clases de inglés

## 1. Objetivo del proyecto

Este proyecto sirve para convertir el material bruto de las clases de inglés —audios, transcripciones, apuntes, vocabulario, correcciones del profesor y ejemplos— en material de estudio claro, estructurado y reutilizable.

La prioridad no es simplemente resumir una clase, sino transformar cada sesión en un sistema de estudio que permita:

- comprender y repasar los contenidos de la clase;
- separar y organizar el vocabulario nuevo;
- identificar gramática, expresiones, *phrasal verbs*, *collocations* y pronunciación;
- recoger errores y correcciones importantes;
- crear material visual y esquemas para facilitar el repaso;
- generar tarjetas Anki de forma coherente;
- mantener una base lingüística consistente en **British English**;
- evitar trabajo manual innecesario.

Esta guía debe considerarse el criterio general de trabajo del proyecto salvo que en una conversación concreta se indique expresamente otra cosa.

---

## 2. Idioma de referencia

### 2.1. Variante principal

Todo el proyecto utilizará por defecto **British English**.

Esto afecta a:

- pronunciación;
- transcripción fonética IPA;
- ortografía;
- vocabulario;
- ejemplos;
- expresiones;
- audio;
- correcciones lingüísticas.

Ejemplos de ortografía preferida:

- `colour` en lugar de `color`;
- `organise` en lugar de `organize`, salvo que el contexto requiera otra variante;
- `travelling` en lugar de `traveling`;
- `centre` en lugar de `center`.

### 2.2. American English

El interés por American English se trabajará fuera de este proyecto.

Por tanto:

- no se mezclarán de forma sistemática variantes británicas y americanas;
- las tarjetas Anki principales usarán pronunciación británica;
- el IPA será británico cuando haya diferencias relevantes;
- no se crearán dos versiones UK/US de una tarjeta salvo petición expresa;
- si una diferencia UK/US es especialmente importante para comprender una clase, puede mencionarse brevemente como nota, pero sin convertirla en el foco del material.

El objetivo es mantener una referencia estable y evitar interferencias mientras se consolida la pronunciación británica.

---

## 3. Material que puede recibir el proyecto

Se podrá trabajar a partir de:

- audios completos de clase;
- fragmentos de audio;
- transcripciones;
- apuntes escritos durante la clase;
- fotografías o capturas de apuntes;
- listas de vocabulario;
- ejercicios;
- explicaciones del profesor;
- correcciones;
- frases que hayan generado dudas;
- material de apoyo;
- combinaciones de cualquiera de los anteriores.

No es necesario que el material llegue limpio ni organizado.

Cuando haya información suficiente, se reorganizará directamente siguiendo esta guía.


## 3.1. Transcripción y conservación de audios de clase

Cuando una clase incluya uno o varios audios y sea necesario analizarlos para extraer información fiable, se seguirá por defecto este procedimiento:

- transcribir el audio antes de realizar la consolidación final de la clase, salvo que exista ya una transcripción completa y fiable;
- conservar la transcripción como fuente de consulta independiente del resumen final;
- vincular claramente cada transcripción con su audio de origen;
- identificar cada archivo por la fecha de la clase y la hora de inicio de la grabación;
- incluir marcas de tiempo internas con una granularidad suficiente para poder volver al fragmento concreto del audio;
- indicar, cuando proceda, si la transcripción es automática, parcialmente revisada o verificada;
- marcar expresamente los fragmentos dudosos o ininteligibles y no inventar contenido;
- utilizar la transcripción como fuente para localizar explicaciones, vocabulario, ejemplos, correcciones, pronunciación y dudas del alumno;
- mantener el audio original cuando sea útil como referencia de pronunciación o contexto.

Como convención de nombres, se priorizarán formatos estables como:

`AAAA-MM-DD_HHMM_Transcripcion_clase.md`

Durante el procesamiento, la transcripción podrá permanecer en `01_Clases/00_Pendiente_de_procesar`. Al cerrar la clase, se conservará junto con el resto del material consolidado en `01_Clases/01_Clases_procesadas`, salvo que exista una razón organizativa para mantener el audio y la transcripción en ubicaciones separadas pero claramente relacionadas.

La transcripción y el resumen cumplen funciones distintas: la transcripción sirve como fuente de consulta y verificación; el resumen sirve como material de estudio condensado. Cuando el audio aporte valor futuro, se conservarán ambos.

---

## 4. Flujo de trabajo después de cada clase

Siempre que el material lo permita, se seguirá este proceso.

### Tratamiento y conservación de los audios de clase

Cuando una clase incluya uno o varios audios y sea necesario utilizarlos para recuperar información, se realizará una transcripción de trabajo antes o durante el análisis. La transcripción servirá como fuente consultable para localizar explicaciones, ejemplos, correcciones, pronunciación, vocabulario y dudas sin tener que volver a escuchar íntegramente la grabación.

Siempre que se genere una transcripción de un audio de clase:

- se conservará en el repositorio de Google Drive del proyecto para futuras consultas;
- se mantendrá vinculada de forma inequívoca al audio de origen;
- se indicarán la **fecha de la clase** y la **hora de inicio del audio** cuando estén disponibles;
- se incluirán **marcas de tiempo** dentro de la transcripción para poder regresar al punto exacto de la grabación;
- si una clase contiene varios archivos de audio, cada tramo conservará la referencia a su archivo y hora de inicio correspondientes;
- los fragmentos dudosos o inaudibles se marcarán como tales y no se completarán inventando contenido;
- se distinguirá claramente una transcripción automática o parcialmente revisada de una transcripción verificada;
- la transcripción se conservará aunque posteriormente se cree un resumen consolidado, porque actúa como fuente de consulta y verificación.

Formato recomendado para las marcas temporales:

- nombre de archivo: `AAAA-MM-DD_HHMM_Transcripcion_clase.md`;
- metadatos iniciales: fecha, hora de inicio, archivo de audio de origen y estado de revisión;
- marcas internas: `[HH:MM:SS]` relativas al audio y, cuando resulte útil y pueda calcularse con fiabilidad, también la hora real de la clase.

Mientras la clase esté pendiente de consolidar, la transcripción podrá permanecer en `01_Clases/00_Pendiente_de_procesar`. Al cerrar la clase, se conservará junto al material de esa sesión en `01_Clases/01_Clases_procesadas`, de forma que pueda recuperarse en futuras conversaciones como fuente documental.

La transcripción no sustituye al resumen final: el resumen debe seleccionar y explicar lo importante, mientras que la transcripción conserva la evidencia y el contexto de origen.

### Paso 1. Recuperar y ordenar el contenido

Primero se identificará qué partes pertenecen a:

- explicación principal;
- gramática;
- vocabulario;
- expresiones;
- ejemplos;
- pronunciación;
- errores;
- correcciones;
- preguntas;
- ejercicios;
- comentarios secundarios.

Se eliminará ruido que no aporte valor al estudio, pero no se descartará información potencialmente útil solo por estar desordenada.

### Paso 2. Identificar lo verdaderamente importante

No todo lo dicho en una clase debe convertirse en material de estudio.

Se priorizarán:

- conceptos nuevos;
- reglas relevantes;
- errores del alumno;
- correcciones del profesor;
- vocabulario útil;
- expresiones naturales;
- estructuras reutilizables;
- pronunciaciones que hayan presentado dificultad;
- ejemplos que ayuden a fijar una idea.

### Paso 3. Crear el resumen de la clase

Se generará un resumen claro y práctico.

No debe ser una simple transcripción reducida.

Debe explicar:

- qué se trabajó;
- qué conceptos son importantes;
- qué reglas aparecieron;
- qué ejemplos merecen conservarse;
- qué errores conviene revisar;
- qué vocabulario debería estudiarse.

### Paso 4. Crear un esquema o mapa mental

Cuando el contenido lo permita, se organizará visualmente o jerárquicamente para facilitar el repaso.

El mapa mental puede incluir:

- tema central;
- conceptos secundarios;
- reglas;
- excepciones;
- ejemplos;
- relaciones entre ideas;
- vocabulario conectado;
- errores frecuentes.

El objetivo es que pueda utilizarse como material de repaso rápido, no repetir todo el resumen.

### Paso 5. Separar el vocabulario

Todo el vocabulario relevante se extraerá a una sección independiente.

Se clasificará, cuando sea útil, en categorías como:

- vocabulario general;
- verbos;
- sustantivos;
- adjetivos;
- adverbios;
- *phrasal verbs*;
- *collocations*;
- expresiones idiomáticas;
- expresiones conversacionales;
- conectores;
- estructuras frecuentes;
- lenguaje funcional;
- pronunciación relevante.

### Paso 6. Seleccionar qué vocabulario merece tarjeta Anki

No todo término detectado debe convertirse automáticamente en tarjeta.

Se priorizará vocabulario que sea:

- nuevo;
- útil;
- frecuente;
- reutilizable;
- difícil de recordar;
- fácil de confundir;
- importante para el nivel del alumno;
- relevante dentro de la clase;
- especialmente útil para producción oral o comprensión auditiva.

Se evitará llenar Anki con palabras demasiado evidentes, redundantes o poco útiles.

### Paso 7. Preparar las tarjetas Anki

Las tarjetas se construirán siguiendo el criterio definido en esta guía.

### Paso 8. Revisión final

Antes de considerar terminado el material de una clase, se comprobará que:

- el inglés sea correcto;
- la variante británica sea consistente;
- el IPA corresponda a British English;
- la traducción sea natural;
- el audio corresponda exactamente a la palabra o expresión;
- la imagen represente bien el concepto;
- los ejemplos sean naturales;
- no haya contradicciones entre audio, IPA y texto;
- no se hayan convertido en tarjetas elementos que no aporten valor.

---

## 5. Estructura recomendada del material de cada clase

Siempre que tenga sentido, el resultado final de una clase seguirá aproximadamente este orden:

1. **Class summary**
2. **Key ideas**
3. **Grammar**
4. **Vocabulary**
5. **Useful expressions**
6. **Phrasal verbs / Collocations**
7. **Pronunciation**
8. **Corrections and mistakes**
9. **Useful examples**
10. **Anki candidates**
11. **Mind map / Study map**
12. **Review notes**

No es obligatorio incluir todas las secciones si la clase no contiene material relevante para alguna de ellas.

---

## 6. Resumen de clase

El resumen debe ser suficientemente detallado para permitir repasar la sesión sin volver a escuchar todo el audio.

Debe:

- conservar el hilo lógico de la clase;
- explicar ideas con lenguaje claro;
- evitar repetir contenido innecesario;
- destacar aquello que probablemente deba revisarse;
- conservar ejemplos especialmente útiles;
- separar los errores del alumno de las formas correctas.

Cuando haya una corrección importante del profesor, se deberá dejar claro:

- qué se dijo;
- cuál es la forma corregida;
- por qué;
- un ejemplo correcto adicional, cuando aporte valor.

---

## 7. Mapas mentales y esquemas

Los mapas mentales deben condensar, no duplicar.

Se utilizarán especialmente para:

- tiempos verbales;
- estructuras gramaticales;
- familias de vocabulario;
- diferencias entre términos;
- *phrasal verbs*;
- categorías semánticas;
- errores recurrentes;
- relaciones entre conceptos.

Siempre que sea posible, se intentará que el mapa responda visualmente a preguntas como:

- ¿qué conceptos están relacionados?
- ¿qué diferencias debo recordar?
- ¿qué estructura depende de cuál?
- ¿qué ejemplos representan cada caso?

---

# 8. Sistema de vocabulario

## 8.1. Principio general

El vocabulario debe almacenarse separado del resto de los apuntes para que pueda reutilizarse posteriormente en:

- Anki;
- repasos;
- listas temáticas;
- ejercicios;
- pruebas;
- actividades de pronunciación.

Cada entrada podrá incluir, según sea necesario:

- palabra o expresión;
- categoría gramatical;
- significado;
- traducción;
- IPA;
- ejemplo;
- notas de uso;
- *collocations*;
- registro;
- etiquetas;
- prompt de imagen;
- audio.

---

## 8.2. Traducción

La traducción al castellano debe transmitir el significado natural, no buscar una equivalencia palabra por palabra cuando esta resulte poco útil.

Si una palabra tiene varios significados:

- se priorizará el significado trabajado en clase;
- se podrán añadir otros significados frecuentes solo si son relevantes;
- se evitará sobrecargar una tarjeta con demasiadas acepciones.

---

## 8.3. Expresiones y unidades completas

Cuando una palabra se utilice normalmente dentro de una expresión, *collocation* o *phrasal verb*, se preferirá aprender la unidad completa.

Ejemplos:

- `make a decision`;
- `take responsibility`;
- `run out of`;
- `look forward to`;
- `at the end of the day`.

El objetivo es aprender inglés utilizable, no listas aisladas de traducciones.


## 8.4. Registro canónico de vocabulario en Google Sheets

El vocabulario del proyecto se registrará de forma acumulativa en un **Google Sheet canónico** del repositorio EOI. Ese documento será la fuente estructurada de vocabulario del proyecto y deberá actualizarse a medida que se procesen nuevas clases.

El objetivo es disponer de una única base reutilizable para estudio, revisión, filtrado y generación posterior de material Anki, evitando listas paralelas o duplicadas por clase.

Cada nueva palabra, expresión, *collocation*, *phrasal verb* u otra unidad léxica relevante se añadirá al mismo Google Sheet, salvo que exista una razón clara para utilizar otra estructura. Antes de añadir una entrada nueva, se comprobará si ya existe para evitar duplicados innecesarios.

El Sheet podrá incluir, según sea útil, columnas como:

- `English`;
- `Spanish`;
- `IPA_UK`;
- `Category`;
- `Example`;
- `Example_ES`;
- `Collocations / Notes`;
- `Register`;
- `Class_Date`;
- `Source`;
- `Anki_Candidate`;
- `Tags`;
- `Audio_UK`;
- `Image`;
- `Image_Prompt`;
- `Status`.

La fecha y la clase de origen deberán quedar registradas cuando aporten contexto útil. El Sheet servirá como base de datos de vocabulario, mientras que los documentos de clase podrán mostrar únicamente el subconjunto trabajado en cada sesión.

Cuando una entrada se seleccione para Anki, se reutilizarán los datos ya existentes en el Sheet en lugar de volver a crear la información desde cero.

El Google Sheet de vocabulario deberá mantenerse dentro del repositorio canónico de este proyecto y no mezclará contenido procedente de otros proyectos.

---

# 9. Filosofía de las tarjetas Anki

## 9.1. Objetivo cognitivo

La tarjeta principal no debe enseñar primero la palabra escrita.

Queremos favorecer esta asociación:

**concepto / imagen → sonido → significado → forma escrita**

y evitar depender inicialmente de:

**texto escrito → pronunciación → significado**

Esto es especialmente importante en inglés porque la relación entre ortografía y pronunciación no es completamente transparente.

Ver primero una palabra puede hacer que el cerebro genere una pronunciación incorrecta basada en la escritura.

Por eso la tarjeta principal debe obligar a reconocer el concepto y el sonido antes de mostrar la grafía.

---

# 10. Formato oficial de la tarjeta Anki principal

## 10.1. Anverso

El anverso contendrá, por defecto:

1. **Imagen representativa del concepto.**
2. **Audio de pronunciación británica.**
3. **Ningún texto escrito que revele la palabra o expresión.**

La imagen debe activar el significado.

El audio debe activar la forma sonora.

El alumno debe intentar recuperar mentalmente la palabra antes de mostrar el reverso.

---

## 10.2. Reverso

El reverso mostrará:

1. **Palabra o expresión escrita en inglés.**
2. **Traducción o significado natural en castellano.**
3. **IPA británico.**
4. **Frase de ejemplo en inglés**, cuando aporte contexto.
5. **Traducción del ejemplo**, cuando sea útil.
6. **Audio repetido**, opcionalmente.

---

## 10.3. Ejemplo de tarjeta

### Anverso

Imagen: una persona desplazándose en tren para ir al trabajo.

Audio: pronunciación británica de `commute`.

No aparece ninguna palabra escrita.

### Reverso

**commute**

desplazarse habitualmente al trabajo / hacer el trayecto diario

`/kəˈmjuːt/`

**Example:**  
*I commute to London every day.*

**Traducción:**  
Voy todos los días a Londres para trabajar.

---

# 11. Campos recomendados para Anki

Siempre que preparemos vocabulario para importar a Anki, se intentarán generar estos campos:

| Campo | Contenido |
|---|---|
| `English` | Palabra o expresión |
| `Spanish` | Traducción o significado natural |
| `IPA_UK` | Transcripción fonética británica |
| `Audio_UK` | Audio británico |
| `Image` | Imagen final |
| `Image_Prompt` | Prompt utilizado o preparado para crear la imagen |
| `Example` | Ejemplo en inglés |
| `Example_ES` | Traducción del ejemplo |
| `Category` | Tipo de vocabulario |
| `Tags` | Clase, tema y otras etiquetas |

No todos los campos tienen que mostrarse necesariamente en la tarjeta.

Los campos adicionales permiten reutilizar el material más adelante.

---

# 12. Criterios para las imágenes

## 12.1. Función de la imagen

La imagen no debe decorar la tarjeta.

Debe funcionar como un disparador de memoria.

Su objetivo es representar el concepto de manera que el alumno pueda acceder al significado sin leer la palabra.

---

## 12.2. Qué debe tener una buena imagen

Se priorizarán imágenes:

- claras;
- fáciles de interpretar;
- visualmente simples;
- memorables;
- relacionadas directamente con el significado trabajado;
- sin texto;
- sin definiciones;
- sin traducciones;
- sin elementos innecesarios.

---

## 12.3. Qué se evitará

Siempre que sea posible, se evitarán:

- palabras escritas dentro de la imagen;
- subtítulos;
- definiciones;
- traducciones;
- iconos ambiguos;
- escenas demasiado complejas;
- elementos que representen otro significado de la palabra;
- imágenes genéricas que no ayuden realmente a recordar.

---

## 12.4. Prompt de imagen

Cuando todavía no exista la imagen, se generará un prompt suficientemente claro para producirla.

El prompt debe describir:

- la escena;
- la acción;
- los objetos relevantes;
- el significado concreto que se quiere representar;
- la ausencia de texto;
- la necesidad de una composición clara.

Se evitará incluir la palabra inglesa dentro de la imagen.

---


## 12.5. Obtención de imágenes desde Wikimedia Commons

Para conceptos concretos y visualmente representables, **Wikimedia Commons** será una fuente preferente de imágenes antes de generar una imagen artificial, siempre que exista una opción clara, adecuada para el significado trabajado y con metadatos de licencia recuperables.

La API pública de MediaWiki permite buscar archivos de Commons y obtener tanto una URL de imagen o miniatura como los metadatos necesarios para conservar la atribución.

### Flujo de selección

Para cada fila de `Vocabulario_EOI` seleccionada para Anki:

1. comprobar el significado concreto que se quiere representar;
2. buscar en Wikimedia Commons varias imágenes candidatas, normalmente entre 3 y 5;
3. priorizar imágenes simples, claras, sin texto y que representen de forma inequívoca el significado trabajado;
4. descartar imágenes ambiguas, decorativas o que representen otra acepción de la palabra;
5. recuperar la URL de la imagen elegida y sus metadatos de licencia;
6. descargar una versión de tamaño razonable para Anki, normalmente entre 600 y 1000 píxeles en su lado principal, evitando originales innecesariamente grandes;
7. guardar el archivo final en `02_Anki/02_Imagenes`;
8. rellenar `Image` en `Vocabulario_EOI` únicamente después de confirmar que el archivo existe;
9. conservar junto a la entrada la procedencia, autoría y licencia necesarias para poder identificar la fuente posteriormente.

### API de búsqueda

Ejemplo conceptual de búsqueda de archivos:

```bash
curl "https://commons.wikimedia.org/w/api.php?action=query&generator=search&gsrsearch=island&gsrnamespace=6&gsrlimit=5&prop=imageinfo&iiprop=url&iiurlwidth=800&format=json"
```

`gsrnamespace=6` limita la búsqueda al espacio de nombres `File:`.

La consulta debe construirse usando el término o concepto que mejor represente el significado estudiado, no necesariamente una traducción literal cuando esta produzca resultados poco útiles.

### Recuperación de metadatos

Una vez elegida una imagen, se solicitarán sus metadatos:

```bash
curl "https://commons.wikimedia.org/w/api.php?action=query&titles=File:NOMBRE_DEL_ARCHIVO.jpg&prop=imageinfo&iiprop=url|extmetadata&format=json"
```

De `extmetadata` se conservarán, cuando estén disponibles y sean necesarios:

- URL de origen;
- autor o creador;
- licencia;
- información de crédito o atribución.

No se realizarán consultas de metadatos completas para grandes cantidades de candidatos si no es necesario. Primero se reducirá la selección y después se recuperará la información detallada del archivo elegido.

### Campos de procedencia

Cuando este flujo empiece a utilizarse de forma habitual, `Vocabulario_EOI` deberá conservar, además de `Image`, información suficiente para poder rastrear la imagen. Se podrán utilizar columnas como:

- `Image_Source`;
- `Image_Author`;
- `Image_License`.

La ausencia de estos campos en una versión anterior del Sheet no justifica perder la procedencia de las imágenes nuevas.

### Convención de nombres

Los archivos se guardarán con nombres estables, por ejemplo:

`img_island.jpg`  
`img_ice_cube.jpg`  
`img_handshake.jpg`

Se utilizarán minúsculas, guiones bajos y caracteres aptos para Anki.

### Cuándo no utilizar Wikimedia Commons

Wikimedia Commons es especialmente adecuada para objetos, lugares, animales, acciones visuales simples y otros conceptos concretos.

Para conceptos abstractos, lenguaje funcional o expresiones como `confident`, `mingle` o `Nice to meet you`, una imagen existente puede resultar ambigua o poco pedagógica. En esos casos se preferirá una imagen generada específicamente para representar el significado trabajado.

La prioridad será siempre la utilidad como disparador de memoria, no utilizar Wikimedia Commons a cualquier precio.

### Control de calidad

Antes de asociar una imagen a una tarjeta se comprobará:

- que represente el significado concreto de la entrada;
- que no contenga la respuesta escrita ni texto innecesario;
- que sea visualmente clara en el tamaño de una tarjeta;
- que no induzca a otra acepción;
- que su procedencia y licencia hayan quedado registradas;
- que el archivo final exista en `02_Anki/02_Imagenes`.


# 13. Palabras abstractas y conceptos difíciles de representar

No todas las palabras pueden representarse mediante un objeto o una escena evidente.

En palabras abstractas, conectores, determinados verbos, adverbios o expresiones complejas, se podrá utilizar:

- una situación;
- una mini escena;
- una metáfora visual;
- una secuencia;
- una reacción emocional;
- un contexto;
- una frase incompleta;
- otro tipo de estímulo que permita activar el significado.

La prioridad no es mantener el formato de imagen a cualquier precio.

Si una imagen resulta artificial o confusa, se buscará otra forma de representar el concepto.

---

# 14. Audio y pronunciación

## 14.1. Principio general

La calidad del audio es importante porque el alumno utilizará ese sonido como referencia para memorizar la pronunciación.

Por tanto, no se utilizará cualquier TTS simplemente por comodidad.

---

## 14.2. Orden de preferencia

Siempre que sea razonablemente posible:

1. **Audio humano fiable de British English.**
2. **TTS neuronal de alta calidad con voz británica.**
3. **TTS convencional**, solo cuando las opciones anteriores no resulten prácticas.

---

## 14.3. Coherencia del audio

Antes de utilizar un audio se comprobará, en la medida de lo posible, que:

- corresponda exactamente a la palabra o expresión;
- sea British English;
- no contenga una lectura diferente;
- no tenga una entonación artificial que pueda inducir a error;
- sea suficientemente claro;
- coincida con el IPA utilizado.

---

## 14.4. Palabras frente a frases

Para palabras individuales, se dará prioridad a una pronunciación de diccionario o a una voz de alta calidad.

Para frases completas, el TTS neuronal puede ser especialmente útil porque permite trabajar:

- ritmo;
- entonación;
- *connected speech*;
- reducción;
- pronunciación contextual.

---


## 14.5. Generación y almacenamiento automático del audio Anki

Para las entradas seleccionadas como candidatas a Anki, el audio se generará de forma **individual, una entrada por llamada**, mediante un servicio TTS accesible por REST que permita seleccionar una voz de **British English**.

La implementación técnica preferente será un servicio compatible con un endpoint tipo:

`POST /v1/audio/speech`

Puede utilizarse un *wrapper* de Microsoft Edge TTS compatible con API REST, como `openai-edge-tts` o una implementación equivalente, siempre que proporcione una pronunciación suficientemente fiable y permita seleccionar explícitamente una voz británica.

### Implementación de referencia: `travisvn/openai-edge-tts`

La implementación técnica de referencia para este proyecto será:

`https://github.com/travisvn/openai-edge-tts`

Este servicio expone una API HTTP compatible con el endpoint de audio de OpenAI:

`POST /v1/audio/speech`

y utiliza `edge-tts` por debajo para acceder a las voces online de Microsoft Edge. Permite devolver MP3 y seleccionar directamente voces británicas como:

- `en-GB-SoniaNeural`;
- `en-GB-RyanNeural`.

Ejemplo de despliegue local con Docker:

```bash
docker run -d -p 5050:5050 \
  -e API_KEY=whatever \
  travisvn/openai-edge-tts:latest
```

Ejemplo de generación de una entrada individual:

```bash
curl http://localhost:5050/v1/audio/speech \
  -H "Authorization: Bearer whatever" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1",
    "voice": "en-GB-SoniaNeural",
    "input": "walkie-talkie",
    "response_format": "mp3"
  }' \
  --output uk_walkie_talkie.mp3
```

### Servicio TTS de producción del proyecto

El servicio TTS actualmente desplegado para este proyecto es:

- **Base URL:** `https://tts.davidpalazon.net`
- **Endpoint de síntesis:** `https://tts.davidpalazon.net/v1/audio/speech`
- **Método:** `POST`
- **Autenticación:** cabecera `Authorization: Bearer <TTS_API_KEY>`
- **Compatibilidad:** API de audio estilo OpenAI
- **Backend:** `travisvn/openai-edge-tts`, que envuelve `edge-tts`
- **Despliegue:** Docker en VPS, publicado mediante Cloudflare Tunnel

La credencial real se proporcionará mediante configuración externa segura y **no se almacenará en esta guía, en `Vocabulario_EOI`, en las tarjetas ni en ningún documento persistente del repositorio**.

Ejemplo de llamada real utilizando una variable de entorno para la credencial:

```bash
curl https://tts.davidpalazon.net/v1/audio/speech \
  -H "Authorization: Bearer $TTS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1",
    "voice": "en-GB-SoniaNeural",
    "input": "walkie-talkie",
    "response_format": "mp3"
  }' \
  --output uk_walkie_talkie.mp3
```

Parámetros operativos previstos:

- `model`: `tts-1` o `tts-1-hd`;
- `voice`: nombre de voz de Microsoft Edge;
- para British English se priorizarán voces como `en-GB-SoniaNeural` y `en-GB-RyanNeural`;
- `input`: palabra o expresión exacta de la entrada;
- `response_format`: `mp3` por defecto.

Al ser compatible con el esquema de `audio/speech` de OpenAI, un cliente que soporte este formato puede reutilizarse cambiando únicamente la `base_url` y la credencial.

La URL base y la clave del servicio serán configurables externamente. La URL pública sí puede documentarse; las claves o *tokens* no se persistirán en los documentos del proyecto.

### Endpoint GET de generación directa

Para la generación de audios Anki se dispone además del siguiente endpoint público de consulta directa:

`https://tts-mcp2.davidpalazon.net/speak`

Formato de llamada:

`https://tts-mcp2.davidpalazon.net/speak?text={TEXT}&voice={VOICE}&format=mp3`

Ejemplo:

`https://tts-mcp2.davidpalazon.net/speak?text=boat&voice=en-GB-SoniaNeural&format=mp3`

Parámetros:

- `text`: palabra o expresión que debe sintetizarse;
- `voice`: voz de Microsoft Edge; por defecto para este proyecto se priorizará `en-GB-SoniaNeural`, salvo que se decida otra voz británica;
- `format`: `mp3` para las tarjetas Anki.

Los valores de `text` y demás parámetros deberán codificarse correctamente para una URL cuando contengan espacios o caracteres especiales.

Este endpoint se conservará como la vía preferente de descarga directa cuando el entorno de trabajo permita realizar la petición GET y recuperar el archivo binario. Si el entorno no permite descargarlo directamente, se utilizará el endpoint REST/MCP disponible sin cambiar el criterio lingüístico ni el nombre final del MP3.

### Flujo de generación

Para cada fila de `Vocabulario_EOI` marcada como candidata a Anki:

1. comprobar que `English`, `IPA_UK` y el significado trabajado son correctos;
2. seleccionar una voz `en-GB` estable para mantener coherencia entre tarjetas;
3. realizar **una petición REST independiente para esa palabra o expresión exacta**;
4. solicitar como salida un archivo MP3;
5. comprobar que la petición ha finalizado correctamente y que el archivo no está vacío ni corrupto;
6. guardar el MP3 en `02_Anki/03_Audio`;
7. escribir en `Audio_UK` el nombre exacto del archivo únicamente después de confirmar que el MP3 existe;
8. actualizar `Status` para reflejar que el audio está disponible.

No se agruparán varias palabras en un único audio. Una entrada Anki debe poder reutilizar su pronunciación de forma independiente.

### Convención de nombres

Se utilizarán nombres estables y aptos para Anki, por ejemplo:

`uk_island.mp3`  
`uk_walkie_talkie.mp3`  
`uk_nice_to_meet_you.mp3`

Los nombres se normalizarán en minúsculas, sin espacios ni caracteres problemáticos.

Si una misma grafía necesita dos pronunciaciones distintas, el nombre deberá incluir un calificador que las diferencie, por ejemplo:

`uk_record_noun.mp3`  
`uk_record_verb.mp3`

### Control de calidad

Antes de considerar terminado un audio se comprobará, siempre que sea posible, que:

- la voz utilizada sea de British English;
- la lectura corresponda exactamente a la entrada;
- la pronunciación sea coherente con `IPA_UK`;
- el comienzo y el final del audio no estén cortados;
- no haya palabras adicionales, anuncios ni sonidos ajenos a la entrada;
- el volumen resulte suficientemente claro para una tarjeta Anki.

Si el TTS produce una lectura dudosa, no se conservará automáticamente. La entrada quedará pendiente de revisión o se utilizará otra voz/fuente.

### Gestión del servicio REST

La URL base, claves o credenciales necesarias para el servicio TTS **no se almacenarán en `Vocabulario_EOI`, en los archivos de audio ni en documentos del repositorio**. Se proporcionarán mediante configuración externa segura, por ejemplo variables de entorno.

El repositorio conservará únicamente:

- el MP3 final;
- la referencia del archivo en `Audio_UK`;
- los datos lingüísticos necesarios para verificar la pronunciación.

### Fuente del audio

Este procedimiento se refiere a audio generado específicamente para las tarjetas. No se recortará por defecto la voz de la profesora o de otros participantes de las grabaciones de clase, porque puede contener ruido, voces solapadas o pronunciación dependiente del contexto.

Los audios de clase se conservarán como fuente de consulta. Para Anki se preferirá una pronunciación limpia, aislada, coherente y reutilizable.


# 15. IPA

El IPA se mostrará normalmente en el reverso.

Su función es:

- confirmar la pronunciación;
- mostrar sonidos que la ortografía no deja claros;
- ayudar a detectar diferencias entre lo escrito y lo pronunciado;
- permitir comprobar el sonido de una palabra incluso sin audio.

El IPA deberá corresponder a **British English**.

No se mezclará IPA británico con audio americano.

---

# 16. Frases de ejemplo

Las frases de ejemplo deben utilizarse cuando aporten algo que la traducción aislada no puede mostrar.

Se priorizarán ejemplos:

- naturales;
- breves;
- fáciles de entender;
- coherentes con British English;
- relacionados con el significado concreto estudiado;
- útiles para mostrar una *collocation* o estructura.

Se evitarán ejemplos artificiales creados únicamente para incluir la palabra.

---

# 17. Tarjetas adicionales

La tarjeta **imagen + audio → palabra escrita + significado** será la tarjeta principal.

Sin embargo, en el futuro podrán crearse otros tipos de tarjeta para trabajar habilidades específicas.

Por ejemplo:

### Producción activa

**Anverso:** concepto / castellano / contexto.  
**Reverso:** palabra inglesa + audio + IPA.

### Comprensión auditiva

**Anverso:** solo audio.  
**Reverso:** palabra + significado + IPA.

### Spelling

**Anverso:** audio.  
**Reverso:** forma escrita.

### Phrasal verbs o collocations

**Anverso:** escena o contexto.  
**Reverso:** expresión completa + significado + ejemplo.

Estas tarjetas serán complementarias.

No sustituirán al formato principal salvo que se decida expresamente.

---

# 18. Etiquetas y organización de Anki

Las tarjetas deben quedar organizadas para poder filtrarlas posteriormente.

Se podrán utilizar etiquetas como:

- `English`;
- `Class_01`;
- `Class_02`;
- `Vocabulary`;
- `Grammar`;
- `Phrasal_Verbs`;
- `Collocations`;
- `Pronunciation`;
- `Work`;
- `Travel`;
- `Daily_Life`;
- u otras categorías relevantes.

La estructura exacta podrá evolucionar cuando exista suficiente volumen de tarjetas.

---

# 19. Errores y correcciones del alumno

Las correcciones realizadas durante las clases tienen un valor especial porque indican puntos reales de dificultad.

Se recogerán de forma separada siempre que sea posible.

Para cada error relevante se podrá registrar:

- forma utilizada;
- forma correcta;
- explicación;
- ejemplo;
- categoría del error.

Las categorías pueden incluir:

- gramática;
- vocabulario;
- pronunciación;
- orden de palabras;
- preposiciones;
- tiempos verbales;
- falsos amigos;
- *collocations*;
- registro.

Los errores recurrentes podrán convertirse en material específico de repaso.

---

# 20. Pronunciación trabajada en clase

Si el profesor corrige explícitamente una pronunciación o aparece una dificultad importante, se destacará.

Se podrá incluir:

- palabra;
- IPA;
- sonido problemático;
- sílaba tónica;
- contraste con una pronunciación incorrecta;
- nota breve de articulación;
- audio de referencia.

La prioridad será corregir el patrón de pronunciación, no únicamente memorizar una palabra aislada.

---

# 21. Gramática

La gramática debe resumirse de forma práctica.

Siempre que sea posible incluirá:

- regla;
- cuándo se utiliza;
- estructura;
- ejemplos;
- errores habituales;
- diferencias con estructuras similares.

Se evitarán explicaciones excesivamente académicas si no ayudan al uso real del idioma.

---

# 22. Criterio para decidir qué conservar

Al procesar una clase, se aplicará este principio:

> Conservar aquello que aumente la capacidad de comprender, recordar o utilizar el inglés.

No es necesario conservar:

- repeticiones;
- comentarios administrativos;
- conversaciones irrelevantes;
- ejemplos redundantes;
- errores inmediatamente corregidos que no tengan valor pedagógico.

Sí conviene conservar:

- explicaciones clave;
- nuevas estructuras;
- correcciones;
- vocabulario útil;
- matices;
- dificultades;
- dudas;
- buenos ejemplos.

---

# 23. Cuando exista incertidumbre

No se debe inventar información para completar una tarjeta.

Si existe duda sobre:

- una palabra escuchada;
- una pronunciación;
- una expresión;
- la intención del profesor;
- un significado;
- un fragmento de audio;

se señalará la incertidumbre.

Se preferirá dejar un elemento pendiente antes que convertir una interpretación dudosa en material de estudio.

---

# 24. Resultado esperado al procesar una clase

Idealmente, cada clase terminará produciendo cuatro grandes bloques.

## A. Material de comprensión

- resumen;
- gramática;
- explicaciones;
- ejemplos;
- correcciones.

## B. Material de repaso

- mapa mental;
- esquema;
- puntos clave;
- errores frecuentes.

## C. Vocabulario

- lista estructurada;
- traducciones;
- IPA;
- ejemplos;
- categorías.

## D. Material Anki

- selección de vocabulario;
- prompts o imágenes;
- audio;
- IPA;
- traducciones;
- ejemplos;
- etiquetas;
- archivo de importación cuando proceda.

---

# 25. Control de calidad antes de cerrar una clase

Antes de entregar el resultado final se comprobará:

### Contenido

- [ ] El resumen refleja realmente la clase.
- [ ] No se han omitido correcciones importantes.
- [ ] Los conceptos están bien diferenciados.
- [ ] El vocabulario relevante está separado.

### British English

- [ ] La ortografía es británica.
- [ ] El vocabulario es coherente con British English.
- [ ] La pronunciación utilizada es británica.
- [ ] El IPA corresponde al audio británico.

### Anki

- [ ] La tarjeta principal no revela la palabra escrita en el anverso.
- [ ] El anverso contiene imagen + audio.
- [ ] La imagen representa el concepto.
- [ ] La imagen no contiene la respuesta escrita.
- [ ] El audio es claro y corresponde exactamente al término.
- [ ] El reverso contiene la palabra inglesa.
- [ ] La traducción castellana es natural.
- [ ] El IPA es correcto.
- [ ] El ejemplo es natural si se ha incluido.
- [ ] La tarjeta aporta valor real y no es redundante.

### Coherencia

- [ ] Imagen, sonido, significado, texto e IPA apuntan al mismo concepto.
- [ ] No se mezclan variantes UK y US accidentalmente.
- [ ] No se ha inventado información dudosa.

---

# 26. Principio final del proyecto

El objetivo no es acumular apuntes ni tarjetas.

El objetivo es construir un sistema de aprendizaje en el que cada elemento tenga una función clara:

**escuchar → reconocer → comprender → recordar → producir**

El material deberá favorecer especialmente la conexión directa entre el concepto y el inglés, reduciendo progresivamente la dependencia del castellano.

En las tarjetas principales, el punto de partida será:

**imagen / concepto + sonido británico**

y la comprobación llegará después mediante:

**palabra escrita + significado en castellano + IPA + contexto**

Esta será la filosofía general de trabajo del proyecto.

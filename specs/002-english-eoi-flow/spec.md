# Feature Specification: Flujo end-to-end de inglés (clase EOI — vocabulario)

**Feature Branch**: `002-english-eoi-flow`

**Created**: 2026-09-26

**Status**: Draft

**Input**: User description: "Flujo end-to-end de inglés (vocabulario): notas + audio de clase -> inventario/transcripción -> vocabulary learning items -> tarjetas con imagen -> audio en frontal y/o reverso -> .apkg -> assessment -> progreso por recognition/production/listening. Ver docs/12-test-scenarios.md (Escenario B) y README.md de learnkit-implementation-spec."

## User Scenarios & Testing *(mandatory)*

<!--
  Esta feature cubre deliberadamente todo el recorrido end-to-end de inglés
  (Escenario B de docs/12-test-scenarios.md) como una única especificación,
  por decisión explícita del usuario. Para mantener cada historia
  independientemente entregable/probable (principio de Spec Kit), el
  recorrido se divide en 7 historias de usuario en el orden en que el propio
  roadmap del proyecto (docs/11-v1-roadmap.md) las secuencia. Cada historia
  es un incremento de valor real, no solo un paso técnico.
-->

### User Story 1 - Registrar una sesión de clase con sus fuentes (Priority: P1)

Un profesor o estudiante que ha tenido una clase de inglés quiere agrupar el material de esa clase (notas, un audio de la grabación, una foto de la pizarra o de una ficha) en una sesión de trabajo, para que LearnKit sepa qué material pertenece a qué clase y pueda procesarlo de forma trazable.

**Why this priority**: Sin una sesión que agrupe las fuentes no existe ningún paso posterior; es el punto de entrada de todo el recorrido y ya aporta valor por sí solo (organización y trazabilidad del material, incluso antes de generar nada automáticamente).

**Independent Test**: Puede probarse de forma aislada creando una sesión, añadiéndole las notas/audio/imagen de una clase real, y comprobando que el sistema reporta cada fuente como inventariada (tipo, tamaño, huella de integridad) sin haber ejecutado ningún paso posterior.

**Acceptance Scenarios**:

1. **Given** un proyecto LearnKit inicializado con perfil de idiomas, **When** el usuario crea una nueva sesión con un título descriptivo, **Then** el sistema crea la sesión con un identificador estable y un directorio propio para su material.
2. **Given** una sesión existente, **When** el usuario añade a ella notas en Markdown, un fichero de audio, y una imagen, **Then** el sistema inventaría cada fuente registrando su tipo, tamaño y una huella de integridad, sin necesitar que el usuario describa manualmente el contenido.
3. **Given** una sesión con fuentes ya inventariadas, **When** el usuario vuelve a añadir la misma fuente sin cambios, **Then** el sistema reconoce que ya está inventariada y no la duplica.
4. **Given** una sesión con una fuente inventariada, **When** el contenido de esa fuente cambia en disco, **Then** el sistema detecta que la huella de integridad ya no coincide y marca la fuente (y cualquier trabajo derivado de ella) como desactualizada.
5. **Given** que el usuario crea una nueva sesión o marca explícitamente una como activa, **When** ejecuta cualquier comando posterior de esta feature sin indicar a qué sesión se refiere, **Then** el sistema usa esa sesión activa en su lugar, sin obligar a repetir su identificador en cada comando.

---

### User Story 2 - Obtener un transcript fiable del audio de clase (Priority: P2)

Un usuario que ha inventariado un audio de clase quiere obtener una transcripción de lo que se dijo, para poder extraer vocabulario y contenido de aprendizaje sin tener que escuchar el audio entero repetidamente, y sin quedar atado a una única herramienta de transcripción.

**Why this priority**: El vocabulario, las tarjetas de listening y el assessment de esta feature dependen de tener texto con marcas de tiempo del audio; es el segundo eslabón imprescindible del recorrido, justo después de tener el material organizado.

**Independent Test**: Puede probarse de forma aislada transcribiendo un audio de clase real y verificando que el resultado incluye texto segmentado con tiempos, y que importar una transcripción ya existente (por ejemplo corregida a mano) para la misma fuente produce un resultado con la misma forma sin volver a procesar el audio.

**Acceptance Scenarios**:

1. **Given** una fuente de audio inventariada, **When** el usuario solicita transcribirla, **Then** el sistema produce una transcripción con segmentos de texto y sus tiempos de inicio/fin, asociada de forma trazable a esa fuente.
2. **Given** una fuente de audio ya transcrita, **When** el usuario importa una transcripción alternativa para la misma fuente (por ejemplo, revisada manualmente fuera de LearnKit), **Then** el sistema la acepta como una nueva versión de la transcripción sin volver a procesar el audio, conservando de qué vino cada versión.
3. **Given** una transcripción generada con una herramienta de transcripción concreta, **When** los pasos posteriores (vocabulario, tarjetas, assessment) consumen esa transcripción, **Then** lo hacen a través de la misma forma de datos, sin importar qué herramienta la produjo.
4. **Given** una transcripción ya generada, **When** el audio original cambia, **Then** el sistema marca esa transcripción como desactualizada y bloquea los pasos que dependen de ella hasta que se regenere o se reconcilie explícitamente.

**Edge case relacionado**: si a un audio marcado como obligatorio le falta la transcripción, cualquier paso posterior que la necesite debe rechazarse explícitamente en lugar de continuar con contenido vacío.

---

### User Story 3 - Extraer vocabulario y contenido de aprendizaje de la clase (Priority: P3)

Un usuario que ya tiene notas y una transcripción de su clase quiere convertir ese material en entradas de vocabulario y elementos de aprendizaje reutilizables (expresiones, phrasal verbs, palabras nuevas), sin tener que leerse todo el material buscando candidatos a mano: el sistema le sugiere expresiones relevantes y el usuario solo confirma, edita o descarta.

**Why this priority**: Es el paso que transforma material en bruto en conocimiento estructurado; sin él no hay nada que convertir en tarjetas ni en preguntas de evaluación, pero solo aporta valor una vez existen sesión y transcripción.

**Independent Test**: Puede probarse de forma aislada a partir de una sesión con notas y transcripción ya disponibles, seleccionando expresiones concretas y comprobando que el sistema crea entradas de vocabulario y elementos de aprendizaje vinculados a esa clase, con trazabilidad hasta la fuente original.

**Acceptance Scenarios**:

1. **Given** una sesión con notas y/o transcripción disponibles, **When** el usuario solicita candidatos de vocabulario, **Then** el sistema analiza el material y propone una lista de expresiones/palabras candidatas, cada una señalando de qué nota o segmento de audio proviene.
2. **Given** una lista de candidatos sugeridos, **When** el usuario confirma uno, **Then** el sistema crea una entrada de vocabulario con su significado, variedad de inglés, y trazabilidad hasta la fuente y el punto exacto de origen; **When** el usuario descarta un candidato, **Then** no se crea ninguna entrada a partir de él.
3. **Given** el material de una clase, **When** el usuario prefiere añadir vocabulario que el sistema no sugirió, **Then** puede crear la entrada manualmente sin depender de ninguna sugerencia previa.
4. **Given** una entrada de vocabulario ya creada en una sesión anterior, **When** la misma expresión aparece en una clase nueva (sugerida o añadida manualmente), **Then** el sistema permite reutilizar la entrada existente en lugar de crear un duplicado.
5. **Given** una entrada de vocabulario, **When** se crea, **Then** el sistema genera también un elemento de aprendizaje genérico vinculado a ella, de forma que el resto del sistema (tarjetas, evaluación, progreso) puede trabajar sobre elementos de aprendizaje sin conocer detalles específicos de idiomas.
6. **Given** dos sesiones de clases distintas de la misma asignatura, **When** ambas generan vocabulario, **Then** el conocimiento de vocabulario sobrevive más allá de una sesión concreta y puede consultarse de forma acumulada.

---

### User Story 4 - Crear tarjetas de estudio con imagen y audio (Priority: P4)

Un usuario que ya tiene vocabulario y elementos de aprendizaje de la clase quiere convertirlos en tarjetas de repaso con apoyo visual y sonoro (una imagen para producción, audio para escucha), para memorizar el vocabulario de forma efectiva más allá de leer texto plano.

**Why this priority**: Las tarjetas son el mecanismo principal de memorización de LearnKit y dependen directamente del vocabulario/elementos de aprendizaje del paso anterior; sin vocabulario no hay nada que convertir en tarjetas.

**Independent Test**: Puede probarse de forma aislada a partir de elementos de aprendizaje ya existentes, generando una tarjeta con imagen en un lado y con audio en el otro (o en ambos), y comprobando que la tarjeta queda validada como completa según lo que su plantilla exige.

**Acceptance Scenarios**:

1. **Given** un elemento de aprendizaje de vocabulario con una imagen asociada, **When** el usuario genera una tarjeta de producción a partir de él, **Then** la tarjeta queda con la imagen en un lado y el texto/audio correspondiente en el otro, según la plantilla elegida.
2. **Given** un elemento de aprendizaje de vocabulario, **When** el usuario genera una tarjeta de listening, **Then** la tarjeta tiene audio en el lado frontal (lo que se escucha primero) y el contenido de respuesta en el reverso, pudiendo llevar también un audio distinto en el reverso (por ejemplo, la pronunciación de la respuesta).
3. **Given** una entrada de vocabulario sin audio propio suministrado, **When** se genera una tarjeta que requiere audio, **Then** el sistema genera automáticamente un audio de pronunciación en lugar de dejar la tarjeta sin ese recurso.
4. **Given** una entrada de vocabulario sin imagen propia suministrada, **When** se genera una tarjeta que requiere imagen, **Then** el sistema busca una imagen adecuada en Wikimedia Commons y, si encuentra una con licencia compatible, la asocia a la tarjeta junto con su atribución (fuente/autor/licencia).
5. **Given** una búsqueda en Wikimedia Commons sin resultados adecuados, **When** se intenta completar la tarjeta, **Then** la tarjeta queda señalada como pendiente de imagen en vez de recibir una imagen no relacionada.
6. **Given** una plantilla de tarjeta que exige imagen o audio en un lado como obligatorio, **When** falta ese recurso (ni suministrado, ni generado, ni encontrado), **Then** el sistema no permite considerar esa tarjeta completa hasta que el recurso exista y sea válido.
7. **Given** una imagen o audio ya usados en otra tarjeta (suministrado, generado, o de Wikimedia), **When** se necesita el mismo recurso en una tarjeta nueva, **Then** el sistema reutiliza el recurso existente en lugar de duplicarlo o de volver a buscarlo/generarlo.

**Edge case relacionado**: si se elimina o corrompe un recurso multimedia que una tarjeta marca como obligatorio, esa tarjeta debe quedar señalada como incompleta hasta que se resuelva, nunca exportarse silenciosamente sin él.

---

### User Story 5 - Exportar las tarjetas a un mazo importable en Anki (Priority: P5)

Un usuario que ya tiene tarjetas completas quiere obtener un fichero de mazo que pueda importar directamente en su programa de repaso espaciado habitual, para practicar fuera de LearnKit con su flujo de estudio de siempre.

**Why this priority**: Es el requisito no negociable de interoperabilidad del producto (Anki es un destino, no la base de datos); sin exportación funcional, las tarjetas creadas en el paso anterior no llegan a usarse en la práctica diaria del usuario.

**Independent Test**: Puede probarse de forma aislada exportando un conjunto de tarjetas ya completas a un fichero de mazo y verificando, al importarlo en un programa de repaso espaciado compatible, que las tarjetas aparecen con su imagen y su(s) audio(s) funcionando en el lado correcto.

**Acceptance Scenarios**:

1. **Given** un conjunto de tarjetas completas (con su imagen/audio válidos), **When** el usuario exporta el mazo, **Then** el sistema produce un único fichero de mazo autocontenido que incluye las tarjetas y todos sus recursos multimedia.
2. **Given** una tarjeta que su plantilla marca con audio obligatorio en el reverso pero que no lo tiene, **When** el usuario intenta exportar el mazo, **Then** la exportación falla señalando exactamente qué tarjeta y qué recurso falta, sin generar un mazo parcial o silenciosamente incompleto.
3. **Given** un mazo ya exportado e importado previamente, **When** el usuario vuelve a exportar tras añadir nuevas tarjetas, **Then** las tarjetas ya existentes no se duplican al reimportar el mazo actualizado.

---

### User Story 6 - Evaluar el dominio del vocabulario (recognition/production/listening) (Priority: P6)

Un usuario quiere comprobar, más allá de repasar tarjetas, si realmente domina el vocabulario de la clase — reconociendo palabras, produciéndolas activamente, y entendiéndolas al escucharlas — mediante un examen que pueda hacer localmente y que le dé una puntuación.

**Why this priority**: La evaluación mide algo distinto de la memorización con tarjetas (principio "Assessment ≠ Memorisation" del proyecto) y depende de que ya existan elementos de aprendizaje de vocabulario; solo aporta valor una vez hay contenido que evaluar.

**Independent Test**: Puede probarse de forma aislada generando un banco de preguntas sobre el vocabulario ya existente, cubriendo al menos las tres dimensiones (reconocimiento, producción, escucha), y comprobando que se puede completar un examen local y obtener una puntuación final sin depender de ningún servidor externo.

**Acceptance Scenarios**:

1. **Given** vocabulario y elementos de aprendizaje ya existentes de una o varias clases, **When** el usuario genera un banco de preguntas de evaluación, **Then** el sistema crea preguntas identificables de forma estable, cubriendo al menos reconocimiento, producción y comprensión auditiva, cada una vinculada al vocabulario que evalúa.
2. **Given** un banco de preguntas, **When** el usuario genera un examen a partir de él y lo completa localmente, **Then** el sistema calcula y muestra una puntuación final sin necesitar conexión a ningún servicio externo.
3. **Given** un examen ya completado, **When** el usuario lo revisa, **Then** cada respuesta queda registrada como un evento inmutable (nunca sobrescrito), asociado a la pregunta, si fue correcta, y a qué dimensión (reconocimiento/producción/escucha) pertenece.

---

### User Story 7 - Ver el progreso por destreza (Priority: P7)

Un usuario quiere ver, de un vistazo, en qué destrezas de vocabulario (reconocimiento, producción, escucha) está mejorando y en cuáles sigue flojo, para decidir en qué centrarse en la siguiente sesión de estudio, sin tener que revisar manualmente todos sus intentos anteriores.

**Why this priority**: Es la vista de valor final del recorrido — cierra el ciclo aprendizaje → evaluación → progreso — pero depende de que existan intentos de evaluación previos (paso anterior) para tener algo que agregar.

**Independent Test**: Puede probarse de forma aislada a partir de un historial de intentos de evaluación ya existente, solicitando una vista de progreso y comprobando que muestra métricas separadas por elemento de aprendizaje y por destreza, calculadas exclusivamente a partir de esos intentos.

**Acceptance Scenarios**:

1. **Given** un historial de intentos de evaluación de vocabulario, **When** el usuario solicita ver su progreso, **Then** el sistema muestra, como mínimo, precisión global y número de intentos por elemento de aprendizaje y por destreza (reconocimiento/producción/escucha).
2. **Given** un historial de intentos que incluye tanto respuestas antiguas como recientes, **When** el usuario solicita el progreso reciente, **Then** el sistema puede distinguir precisión reciente de precisión histórica total.
3. **Given** que no existe ningún cálculo de progreso guardado de antemano, **When** se solicita el progreso, **Then** el sistema lo recalcula a partir de los intentos existentes en ese momento, nunca a partir de un valor cacheado que pueda quedar desincronizado.

---

### Edge Cases

- ¿Qué ocurre si el audio de clase tiene mala calidad (ruido, solapamientos, varios hablantes) y la transcripción resultante tiene errores evidentes en el vocabulario objetivo? El usuario debe poder sustituir la transcripción por una corregida (User Story 2) sin perder el trabajo de vocabulario ya construido sobre segmentos que sí eran correctos, y sin que el sistema oculte que hubo un cambio.
- ¿Qué ocurre si dos expresiones de vocabulario distintas comparten la misma imagen o el mismo audio? Debe permitirse la reutilización del recurso sin que eso implique que ambas expresiones son la misma entrada de vocabulario.
- ¿Qué ocurre si se intenta generar una tarjeta o una pregunta de evaluación a partir de un elemento de aprendizaje que todavía no tiene ninguna entrada de vocabulario ni contenido asociado? El sistema debe rechazarlo explícitamente en lugar de generar una tarjeta/pregunta vacía.
- ¿Qué ocurre si el usuario borra una fuente (por ejemplo, el audio original) después de haber generado vocabulario y tarjetas a partir de ella? El vocabulario y las tarjetas ya creados deben seguir siendo válidos (la trazabilidad hacia la fuente puede quedar rota/señalada, pero el conocimiento ya extraído no desaparece).
- ¿Qué ocurre si el mismo audio se transcribe dos veces con configuraciones distintas (por ejemplo, cambiando de herramienta de transcripción)? Ambas transcripciones deben poder convivir con trazabilidad de cuál se usó para generar qué vocabulario, sin forzar una migración automática silenciosa del trabajo ya hecho.
- ¿Qué ocurre si un examen se interrumpe a mitad (el usuario lo cierra sin terminar)? Las respuestas ya registradas hasta ese punto deben conservarse como intentos válidos; no se pierde el progreso parcial.

## Requirements *(mandatory)*

### Functional Requirements

**Sesión y fuentes**

- **FR-001**: El sistema DEBE permitir crear una sesión de trabajo con un título descriptivo y un perfil de idiomas activo, obteniendo un identificador estable y un espacio propio para su material.
- **FR-002**: El sistema DEBE permitir añadir a una sesión fuentes de al menos estos tipos: notas de texto/Markdown, audio, e imagen, registrando para cada una su tipo, tamaño y una huella de integridad.
- **FR-003**: El sistema DEBE detectar cuándo una fuente ya inventariada no ha cambiado (para no duplicar trabajo) y cuándo su contenido ha cambiado en disco (para marcar como desactualizado cualquier trabajo derivado de ella).
- **FR-003b**: El sistema DEBE recordar cuál es la "sesión activa" del proyecto (la última creada, o la marcada explícitamente por el usuario) y usarla por defecto en cualquier comando de esta feature que necesite saber a qué sesión se refiere, cuando no se indique una explícitamente. El usuario DEBE poder cambiar la sesión activa en cualquier momento sin perder el resto del estado del proyecto.

**Transcripción**

- **FR-004**: El sistema DEBE permitir obtener una transcripción de una fuente de audio, con el texto segmentado y marcas de tiempo, sin exponer a los pasos posteriores ningún detalle específico de la herramienta usada para generarla.
- **FR-005**: El sistema DEBE permitir importar una transcripción ya existente (por ejemplo, corregida manualmente fuera de LearnKit) para una fuente de audio, sin necesidad de volver a procesar el audio original.
- **FR-006**: El sistema DEBE conservar la procedencia de cada transcripción (qué la generó o de dónde se importó) y DEBE marcar como desactualizada una transcripción, y cualquier trabajo que dependa de ella, cuando el audio original cambie.
- **FR-007**: Cualquier paso posterior que dependa de una transcripción DEBE rechazarse explícitamente si la transcripción requerida falta o está desactualizada, en lugar de continuar con contenido vacío o parcial.

**Vocabulario y elementos de aprendizaje**

- **FR-008**: El sistema DEBE permitir crear una entrada de vocabulario (palabra o expresión) a partir de una nota o de un segmento de una transcripción, conservando su significado, variedad de inglés, y la trazabilidad hasta el punto exacto de origen.
- **FR-009**: El sistema DEBE evitar la duplicación de una entrada de vocabulario ya existente cuando la misma expresión vuelve a aparecer en una sesión distinta, permitiendo reutilizarla.
- **FR-010**: El sistema DEBE crear, junto a cada entrada de vocabulario, un elemento de aprendizaje genérico vinculado a ella, de forma que tarjetas, evaluación y progreso puedan operar sobre elementos de aprendizaje sin necesitar lógica específica de idiomas.
- **FR-011**: El vocabulario y los elementos de aprendizaje DEBEN sobrevivir a la sesión en la que se crearon y seguir siendo consultables/reutilizables desde sesiones posteriores.
- **FR-012**: El sistema DEBE analizar automáticamente las notas/transcripción de una sesión y sugerir candidatos de vocabulario objetivo (palabras y expresiones potencialmente relevantes, por ejemplo phrasal verbs o términos poco frecuentes), presentándolos como sugerencias.
- **FR-012b**: Ninguna sugerencia automática DEBE convertirse en una entrada de vocabulario sin confirmación explícita del usuario; el usuario puede aceptar, descartar, o editar cada sugerencia antes de que se cree la entrada, y también puede añadir vocabulario manualmente sin partir de ninguna sugerencia.
- **FR-012c**: El sistema DEBE permitir eliminar una entrada de vocabulario ya persistida (y su elemento de aprendizaje asociado) a través de un comando del CLI, nunca editando manualmente los ficheros de `knowledge/`. Eliminar una entrada de vocabulario DEBE eliminar también, de forma consistente, cualquier tarjeta ya generada que dependa de su elemento de aprendizaje (en cualquier sesión), para no dejar tarjetas huérfanas apuntando a un elemento de aprendizaje inexistente.

**Tarjetas y multimedia**

- **FR-013**: El sistema DEBE permitir generar, a partir de un elemento de aprendizaje de vocabulario, una tarjeta de estudio con al menos un lado con imagen y al menos un lado con audio, según la plantilla elegida.
- **FR-014**: El sistema DEBE soportar audio distinto e independiente en el lado frontal y en el lado reverso de una misma tarjeta.
- **FR-015**: Cuando la plantilla de una tarjeta marque un recurso (imagen o audio) como obligatorio en un lado, el sistema NO DEBE considerar esa tarjeta completa/exportable hasta que ese recurso exista y sea válido.
- **FR-016**: El sistema DEBE reutilizar un recurso multimedia (imagen o audio) ya existente cuando sea equivalente al que necesita una tarjeta nueva, en lugar de duplicarlo.
- **FR-017**: El sistema DEBE permitir asociar imágenes y audios a una tarjeta mediante recursos suministrados por el usuario (por ejemplo, una foto de la pizarra, un audio grabado por el profesor).
- **FR-017b**: Cuando una entrada de vocabulario no tenga audio propio suministrado por el usuario, el sistema DEBE poder generar automáticamente un audio de pronunciación (texto a voz) para ella, a partir de su texto, variedad de inglés, y una política de voz del proyecto; el audio generado se trata igual que cualquier otro recurso multimedia (reutilizable, validable, sustituible por uno suministrado por el usuario si se prefiere). El provider de voz por defecto DEBE poder ser un servicio REST externo (no solo un binario local) configurado mediante variables de entorno (URL base y credencial), sin que la credencial quede nunca persistida en el repositorio (ver `research.md` §3, revisión 2026-09-27).
- **FR-017c**: Cuando una entrada de vocabulario no tenga imagen propia suministrada por el usuario, el sistema DEBE poder buscar y obtener una imagen adecuada de un repositorio de imágenes libres (Wikimedia Commons) a partir del texto/significado de la entrada, en lugar de dejar la tarjeta sin imagen.
- **FR-017d**: Toda imagen obtenida de Wikimedia Commons DEBE conservar registrada su fuente, autor y licencia, y esa atribución DEBE quedar disponible allí donde la tarjeta/mazo exportado lo requiera, para cumplir los términos de la licencia de la imagen.
- **FR-017e**: Si ninguna imagen de Wikimedia Commons resulta adecuada (sin resultados, o resultados de licencia incompatible), el sistema DEBE dejar la tarjeta señalada como pendiente de imagen en lugar de asignar una imagen no relacionada o con licencia incompatible.
- **FR-017g**: Un fallo transitorio de la búsqueda de imágenes (red, timeout, respuesta no parseable) NO DEBE tratarse de inmediato como "sin imagen adecuada" (FR-017e); el sistema DEBE reintentar la búsqueda un número acotado de veces antes de dejar la tarjeta como pendiente de imagen, para no confundir una intermitencia del proveedor externo con una limitación real de la búsqueda (ver `research.md` §4, revisión 2026-09-27).
- **FR-017f**: Reconstruir el conjunto de tarjetas (`cards build`) NO DEBE volver a resolver imagen/audio para una tarjeta que ya esté completa; solo debe procesar tarjetas ausentes o pendientes, salvo que el usuario pida explícitamente forzar la regeneración completa. Esto evita que la intermitencia de un proveedor externo (por ejemplo, Wikimedia Commons) convierta en pendiente una tarjeta que ya se había resuelto correctamente en una ejecución anterior.
- **FR-017h** (añadida post-release tras uso real, 2026-09-28): cuando un recurso multimedia **opcional** (no obligatorio según la plantilla) no se pueda resolver, `cards build` DEBE informar explícitamente de qué tarjeta, qué lado y por qué motivo — nunca descartar ese motivo en silencio. Encontrado tras uso real: con `TTS_API_KEY` sin configurar, 127/127 tarjetas quedaron sin audio (el audio del reverso es opcional en `image-to-production-v1`) sin ningún aviso en `cards build`, `cards validate` ni `status`; la única forma de detectarlo fue leer el YAML de una tarjeta a mano.

**Exportación a Anki**

- **FR-018**: El sistema DEBE permitir exportar un conjunto de tarjetas completas a un único fichero de mazo autocontenido (tarjetas + todos sus recursos multimedia) importable en un programa de repaso espaciado compatible.
- **FR-019**: La exportación DEBE fallar de forma explícita, señalando qué tarjeta y qué recurso falta, si alguna tarjeta incluida no cumple los requisitos obligatorios de su plantilla; no debe producir un mazo parcial o silenciosamente incompleto.
- **FR-019b** (añadida post-release tras uso real, 2026-09-28): el usuario DEBE poder pedir explícitamente que la exportación excluya las tarjetas incompletas y exporte igualmente el resto (`--skip-incomplete` o equivalente), en vez de que una única tarjeta atascada (p. ej. por falta de imagen para un concepto abstracto) bloquee la exportación de un mazo por lo demás completo. Esto NO DEBE ser el comportamiento por defecto (FR-019 se mantiene: sin el flag explícito, sigue fallando); y el resultado DEBE listar exactamente qué tarjetas se excluyeron y por qué, nunca omitirlas en silencio.
- **FR-020**: Reexportar un mazo tras añadir nuevas tarjetas NO DEBE duplicar las tarjetas ya existentes al reimportarlo.

**Evaluación**

- **FR-021**: El sistema DEBE permitir generar un banco de preguntas de evaluación sobre el vocabulario/elementos de aprendizaje existentes, cubriendo como mínimo las destrezas de reconocimiento, producción, y comprensión auditiva.
- **FR-022**: Cada pregunta de evaluación DEBE tener una identidad estable, una respuesta correcta explícita, una puntuación, y (cuando aplique) audio para las preguntas de comprensión auditiva.
- **FR-023**: El sistema DEBE permitir completar un examen de forma local, sin depender de ningún servicio externo, y mostrar una puntuación final al terminar.
- **FR-024**: Cada respuesta dada en un examen DEBE registrarse como un evento inmutable (nunca sobrescrito), incluyendo a qué pregunta y destreza corresponde y si fue correcta.

**Progreso**

- **FR-025**: El sistema DEBE permitir consultar el progreso agregando los intentos de evaluación por elemento de aprendizaje y por destreza (reconocimiento/producción/escucha), mostrando como mínimo precisión y número de intentos.
- **FR-026**: El progreso DEBE recalcularse siempre a partir de los intentos existentes en el momento de la consulta, nunca servirse desde un valor guardado que pueda quedar desincronizado de los intentos reales.

### Key Entities

- **Sesión**: agrupación de trabajo de una clase concreta; contiene las fuentes y es el punto de entrada de todo el recorrido.
- **Fuente**: material original inventariado (nota, audio, imagen), con su tipo, tamaño y huella de integridad.
- **Transcripción**: texto segmentado con marcas de tiempo derivado de una fuente de audio; conserva su procedencia y puede tener varias versiones para la misma fuente.
- **Entrada de vocabulario**: palabra o expresión de inglés con significado, variedad, y trazabilidad hasta su origen; sobrevive a la sesión donde se creó.
- **Elemento de aprendizaje**: representación genérica de conocimiento (aquí, ligada a una entrada de vocabulario) sobre la que operan tarjetas, evaluación y progreso sin lógica específica de idioma.
- **Tarjeta de estudio**: unidad de repaso con contenido en dos lados (frontal/reverso), cada lado con sus propios recursos multimedia y reglas de obligatoriedad según su plantilla.
- **Recurso multimedia**: imagen o audio reutilizable, referenciado por una o varias tarjetas/preguntas, con validación de que existe y es válido. Su origen puede ser: suministrado por el usuario, generado (audio de pronunciación por texto a voz), u obtenido de un repositorio externo (imágenes de Wikimedia Commons, con fuente/autor/licencia registrados).
- **Mazo exportado**: fichero autocontenido con tarjetas y sus recursos, listo para importar en un programa de repaso espaciado externo.
- **Pregunta de evaluación**: unidad de evaluación con identidad estable, respuesta correcta, puntuación, y destreza que mide.
- **Intento**: evento inmutable que registra una respuesta a una pregunta de evaluación.
- **Vista de progreso**: agregación derivada de los intentos, por elemento de aprendizaje y por destreza; nunca es la fuente de verdad, siempre se recalcula.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Un usuario puede pasar de tener las notas y el audio de una clase real a tener un mazo de tarjetas importable en un programa de repaso espaciado, sin necesitar herramientas externas más allá de las que use para revisar/corregir una transcripción si lo desea.
- **SC-002**: El 100% de las tarjetas incluidas en un mazo exportado tienen sus recursos multimedia obligatorios presentes y válidos; ninguna exportación produce un mazo con recursos rotos o ausentes.
- **SC-003**: Cambiar la herramienta usada para transcribir un mismo audio no obliga a rehacer manualmente el vocabulario, las tarjetas o las preguntas de evaluación ya creadas a partir de una transcripción anterior compatible.
- **SC-004**: Un examen de vocabulario generado a partir de una clase cubre las tres destrezas (reconocimiento, producción, escucha) y puede completarse y puntuarse en menos de 15 minutos para un vocabulario objetivo de una clase típica (10-20 expresiones).
- **SC-005**: Un usuario puede identificar, a partir de la vista de progreso, en qué destreza concreta tiene peor rendimiento sin tener que revisar manualmente los intentos individuales.
- **SC-006**: Reutilizar una entrada de vocabulario, una imagen o un audio ya existente en una sesión nueva no crea una copia adicional visible como si fuera contenido distinto.
- **SC-007**: El 100% de las imágenes obtenidas de Wikimedia Commons que terminan en una tarjeta llevan su atribución (fuente/autor/licencia) accesible; ninguna imagen de Wikimedia Commons se usa sin su atribución registrada.
- **SC-008**: Al menos el 90% de las entradas de vocabulario sin audio/imagen propios terminan con un recurso automático (audio generado o imagen de Wikimedia Commons) sin intervención manual del usuario, para un vocabulario objetivo típico de clase (10-20 expresiones habituales, no jerga muy específica).

## Assumptions

- El "audio de clase" es un fichero de audio ya grabado y disponible localmente (por ejemplo, grabado con el móvil del profesor); esta feature no cubre grabación en vivo desde LearnKit.
- La "sesión activa" (FR-003b) es un puntero local al proyecto (no versionado, no compartido entre máquinas), análogo al `.specify/feature.json` que ya usa el propio flujo de Spec Kit de este repositorio para la feature activa; su ausencia o borrado nunca es un error — el usuario siempre puede volver a fijarla o seguir indicando `--session` explícitamente.
- La corrección de errores de transcripción (por ruido, solapamientos, nombres propios mal reconocidos) se realiza fuera de LearnKit (por ejemplo, con una herramienta externa de transcripción/edición) y se reincorpora mediante importación (FR-005); esta feature no incluye un editor de transcripciones dentro de LearnKit.
- La variedad de inglés por defecto del perfil de idiomas es la británica (en-GB), siguiendo `docs/06-profiles.md`; el usuario puede indicar una variedad distinta por proyecto/sesión si lo necesita.
- Kahoot y Blooket (exporters de juegos) quedan fuera del alcance de esta feature; el roadmap del proyecto los sitúa en una fase posterior separada de la evaluación local. El examen local (FR-023) es el único mecanismo de evaluación cubierto aquí.
- El "banco de preguntas" y el examen de esta feature son evaluación LearnKit local; la integración con plataformas externas de evaluación formal (TAO/QTI) es una fase de V2 explícitamente fuera de alcance.
- Cada tarjeta/pregunta se relaciona con exactamente un perfil de idiomas activo en la sesión; combinar vocabulario de varios idiomas en una misma sesión no se contempla en esta feature.
- La generación automática de audio (FR-017b) usa una voz/política por defecto configurable a nivel de proyecto (por ejemplo, una voz británica neutra); elegir entre proveedores de voz concretos o voces personalizadas por usuario no se especifica en esta feature más allá de que exista una política de voz por defecto.
- La búsqueda de imágenes en Wikimedia Commons (FR-017c/d/e) se limita a licencias que permitan reutilización con atribución (por ejemplo, Creative Commons); no se contempla en esta feature negociar ni pagar licencias no libres.
- El análisis lingüístico que sugiere candidatos de vocabulario (FR-012) es una ayuda, no una fuente de verdad: sus sugerencias nunca se consideran vocabulario "creado" hasta la confirmación del usuario (FR-012b), y su calidad puede variar según el material; no se fija en esta feature una tasa de acierto exacta del sugeridor.

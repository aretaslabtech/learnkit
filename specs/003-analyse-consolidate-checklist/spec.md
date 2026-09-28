# Feature Specification: Checklist por fase y fases `analyse`/`consolidate` reales

**Feature Branch**: `003-analyse-consolidate-checklist`

**Created**: 2026-09-28

**Status**: Draft

**Input**: User description: "Rediseñar el motor de fases de sesión (`analyse`/`consolidate`) para que cada fase tenga un checklist de sub-elementos verificable, no un único estado global, y para que el sistema pregunte al usuario cuando falte información en vez de fallar o continuar en silencio. La fase `analyse` debe producir resumen + mapa mental + 'páginas' de concepto por sesión, rellenando huecos de la documentación de origen cuando sea posible. La fase `consolidate` debe generar el listado de vocabulario/preguntas candidatas, deduplicado contra el almacén persistente existente. El pipeline se generaliza más allá de inglés/EOI: sesiones = un día de formación de cualquier asignatura."

## Clarifications

### Session 2026-09-28

- Q: ¿Quién genera el contenido de redacción libre (resumen, mapa mental, páginas de concepto) — un agente vía Skill, o el binario Rust llamando directamente a un servicio externo? → A: Un agente vía Skill (mismo patrón que `learnkit-language` con el vocabulario de la feature 002): el agente redacta, la CLI solo valida estructura/trazabilidad y persiste — nunca genera prosa.
- Q: `consolidate` (FR-011) reutiliza el dedup por `lemma` de la feature 002, específico de vocabulario de idioma — ¿esta feature limita `consolidate` a candidatos de vocabulario, dejando el dedup de preguntas de gramática/conocimiento general para la feature posterior de la plantilla de gramática? → A: Sí — `consolidate` en esta feature produce y deduplica solo candidatos de tipo vocabulario; la generalización del dedup a otros tipos de candidato queda fuera de alcance, para cuando se aborde la plantilla de gramática/conocimiento general.

## User Scenarios & Testing *(mandatory)*

<!--
  Esta feature corrige una divergencia real entre la visión del usuario (control
  de fase por sesión estilo spec-kit, con checklist y huecos explícitos) y el
  motor de fases actual (crates/learnkit-workflow), que solo tiene un estado
  global por fase y ninguna fase `consolidate`. Las historias siguen el orden de
  dependencia natural: primero el checklist (infraestructura visible), luego el
  contenido real de `analyse`, luego qué pasa cuando falta información, y por
  último `consolidate` (que consume la salida de `analyse`).
-->

### User Story 1 - Ver de un vistazo en qué punto está una sesión (Priority: P1)

Un usuario que gestiona varias sesiones de formación (una por día de clase, de cualquier asignatura) quiere consultar el estado de una sesión concreta y ver, dentro de cada fase, exactamente qué elementos están hechos y cuáles faltan — no solo si la fase en conjunto está "válida" o no — de la misma forma en que este mismo proyecto usa `.specify/` para trackear en qué paso está una feature.

**Why this priority**: Sin esta visibilidad, ninguna de las historias siguientes aporta valor observable: el usuario no puede saber si "analyse" está realmente completo o solo parcialmente, ni decidir qué falta por hacer. Es la base sobre la que se apoyan las demás.

**Independent Test**: Puede probarse de forma aislada consultando el estado de una sesión cuya fase `analyse` tiene, por ejemplo, el resumen generado pero no el mapa mental, y comprobando que el sistema reporta ambos elementos por separado en vez de un único estado agregado.

**Acceptance Scenarios**:

1. **Given** una sesión en la que la fase `analyse` ha generado el resumen pero todavía no el mapa mental ni las páginas de concepto, **When** el usuario consulta el estado de la sesión, **Then** el sistema muestra los tres elementos de `analyse` por separado, cada uno con su propio estado (hecho / pendiente / bloqueado), no un único estado para toda la fase.
2. **Given** una sesión con todos los elementos de una fase completos, **When** el usuario consulta su estado, **Then** la fase se muestra como completa y cada uno de sus elementos aparece marcado como hecho.
3. **Given** una sesión cuya fase `inventory` cambia (por ejemplo, se añade una fuente nueva) después de que `analyse` ya estaba completa, **When** el usuario consulta el estado, **Then** los elementos de `analyse` que dependían de esa fuente se muestran como desactualizados, no como hechos.

---

### User Story 2 - Obtener resumen, mapa mental y páginas de concepto de una sesión (Priority: P1)

Un usuario que ha inventariado el material de una sesión (notas, transcripción de audio, fotos) quiere que el sistema genere, a partir de ese material, un resumen claro de lo trabajado, un mapa mental de repaso rápido y unas páginas de concepto concentradas — y que, si el material de origen tiene huecos evidentes o está mal explicado, el sistema intente completarlo antes de dar el análisis por terminado.

**Why this priority**: Es el primer paso que aporta valor de estudio real (más allá de organizar ficheros): convierte material de clase en contenido de repaso utilizable, y es la entrada de la que dependen tanto `consolidate` como, más adelante, la generación de tarjetas.

**Independent Test**: Puede probarse de forma aislada analizando una sesión con notas y transcripción de ejemplo, y comprobando que el resultado incluye un resumen con la estructura esperada (qué se trabajó, conceptos importantes, reglas, ejemplos, errores a repasar, vocabulario), un mapa mental, y al menos una página de concepto, todos ellos trazables al material de origen.

**Acceptance Scenarios**:

1. **Given** una sesión con notas y transcripción inventariadas, **When** el usuario ejecuta el análisis de la sesión, **Then** el sistema genera un resumen que identifica qué se trabajó, los conceptos importantes, reglas, ejemplos a conservar, errores a repasar y vocabulario a estudiar.
2. **Given** la misma sesión, **When** el análisis se completa, **Then** el sistema genera además un mapa mental o esquema pensado para repaso rápido (no una repetición del resumen) y al menos una página de concepto que concentra la explicación de una idea trabajada en la sesión.
3. **Given** una sesión cuyo material de origen deja un concepto mencionado pero sin explicar, **When** el sistema detecta ese hueco durante el análisis, **Then** intenta completarlo (documentando de dónde proviene la información añadida) antes de considerar ese elemento del análisis como hecho.
4. **Given** una sesión ya analizada, **When** el usuario vuelve a analizarla sin que el material de origen haya cambiado, **Then** el sistema no regenera el resumen/mapa/páginas desde cero (reutiliza el resultado existente), igual que ya ocurre hoy con la idempotencia de otras fases del proyecto.

---

### User Story 3 - Ser consultado cuando falta información para completar un elemento (Priority: P2)

Un usuario cuyo material de origen no es suficiente para generar alguno de los elementos de una fase (por ejemplo, no hay contenido suficiente para un mapa mental útil, o una fuente de audio obligatoria no tiene transcripción) quiere que el sistema se lo indique explícitamente y le ofrezca una decisión — completar el elemento manualmente, omitirlo de forma explícita, o aportar más material — en vez de que la fase falle sin explicación o, peor, se dé por completa con contenido pobre.

**Why this priority**: Evita dos fallos igual de malos: bloquear al usuario sin decirle qué hacer, o generar contenido de baja calidad silenciosamente. Depende de que existan elementos de checklist individuales (Historia 1) sobre los que aplicar esta detección.

**Independent Test**: Puede probarse de forma aislada analizando una sesión con material insuficiente para el mapa mental (pero suficiente para el resumen), y comprobando que el sistema marca específicamente ese elemento como "necesita decisión del usuario", con un mensaje que explica qué falta, mientras el resumen se completa con normalidad.

**Acceptance Scenarios**:

1. **Given** una sesión cuyo material no permite generar un elemento concreto del análisis, **When** el sistema lo detecta, **Then** marca ese elemento (no toda la fase) como pendiente de decisión del usuario, con un mensaje que explica qué información falta.
2. **Given** un elemento marcado como pendiente de decisión, **When** el usuario aporta el material que faltaba y vuelve a ejecutar el análisis, **Then** el sistema genera ese elemento con normalidad sin repetir el trabajo ya hecho en los demás elementos.
3. **Given** un elemento marcado como pendiente de decisión, **When** el usuario decide explícitamente omitirlo, **Then** el sistema registra esa decisión y dejar de bloquear los pasos posteriores por ese elemento, sin volver a preguntarlo en cada ejecución.

---

### User Story 4 - Consolidar el vocabulario/preguntas candidatas de una sesión sin duplicar lo ya existente (Priority: P3)

Un usuario que ha completado el análisis de una sesión quiere obtener, a partir de ese análisis, un listado de vocabulario candidato para convertir en tarjetas — comprobado automáticamente contra lo que ya existe de sesiones anteriores, para no proponerle de nuevo algo que ya tiene registrado. (Candidatos de preguntas de gramática/conocimiento general quedan para una feature posterior, ver Assumptions.)

**Why this priority**: Es el puente entre "analizar" y "aprender/tarjetas" (ya existente en la feature 002); depende de que `analyse` produzca ya un resultado real (Historia 2), así que llega en último lugar.

**Independent Test**: Puede probarse de forma aislada consolidando dos sesiones distintas que comparten una misma palabra/concepto en su análisis, y comprobando que la segunda consolidación reconoce la entrada ya existente en vez de proponerla como candidata nueva.

**Acceptance Scenarios**:

1. **Given** una sesión analizada, **When** el usuario ejecuta la consolidación, **Then** el sistema produce un listado de candidatos de vocabulario derivado del resumen/páginas de esa sesión.
2. **Given** un candidato cuyo concepto ya existe como entrada persistida de una sesión anterior, **When** se consolida la nueva sesión, **Then** el sistema no lo repite en el listado de candidatos nuevos, indicando que ya existe.
3. **Given** un listado de candidatos consolidado, **When** el usuario lo consulta, **Then** cada candidato conserva de qué parte del análisis de la sesión proviene (trazabilidad), igual que ya exige el resto del proyecto para el vocabulario.

---

### Edge Cases

- ¿Qué ocurre si el usuario ejecuta `consolidate` sobre una sesión cuya fase `analyse` todavía tiene elementos pendientes de decisión? El sistema debe bloquear explícitamente esa parte de la consolidación afectada por el hueco, no consolidar con datos incompletos silenciosamente.
- ¿Qué ocurre si el material de una sesión es tan escaso que ni el resumen es generable (por ejemplo, una fuente vacía)? El elemento "resumen" debe quedar como pendiente de decisión igual que cualquier otro elemento, no producir un resumen vacío marcado como hecho.
- ¿Qué ocurre si el usuario cambia una fuente después de que un elemento del análisis ya fue completado, generado o rellenado manualmente por el usuario? Ese elemento debe pasar a desactualizado como el resto del pipeline (mismo comportamiento de invalidación en cascada que ya existe para `inventory`/`vocabulary`/`cards`).
- ¿Qué ocurre si dos elementos de checklist de `analyse` son independientes entre sí (p. ej. el mapa mental no depende del resumen)? Deben poder completarse en cualquier orden y consultarse su estado por separado, no forzar un orden interno rígido salvo que exista una dependencia real de datos.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE permitir que una fase del pipeline (empezando por `analyse`) reporte el estado de varios elementos de checklist por separado (p. ej. resumen, mapa mental, páginas de concepto), en lugar de un único estado global para toda la fase.
- **FR-002**: `learnkit status --session <id>` DEBE mostrar, para cada fase con checklist, el estado individual de cada uno de sus elementos (hecho / pendiente / bloqueado / pendiente de decisión del usuario), no solo el estado agregado de la fase.
- **FR-003**: El sistema DEBE seguir soportando fases sin checklist (estado único, como hoy) para no obligar a todas las fases existentes (`inventory`, `vocabulary`, `cards`, `anki`, `assessment`) a adoptar un checklist si no lo necesitan.
- **FR-004**: Cuando cambie el material de origen del que depende un elemento de checklist ya completado, el sistema DEBE marcar ese elemento (no necesariamente toda la fase) como desactualizado, siguiendo el mismo principio de invalidación en cascada que ya aplica a las fases existentes.
- **FR-005**: El sistema DEBE implementar la fase `analyse` para que, a partir del material inventariado de una sesión (notas, transcripción), permita registrar tres elementos redactados por un agente vía Skill (mismo patrón que `learnkit-language` en la feature 002 — el agente lee el material a través de una consulta CLI, redacta, y confirma con un comando CLI que valida estructura/trazabilidad, nunca la calidad de la prosa): un resumen de la clase (qué se trabajó, conceptos importantes, reglas, ejemplos a conservar, errores a repasar, vocabulario a estudiar), un mapa mental/esquema de repaso rápido, y una o más páginas de concepto concentrado.
- **FR-006**: El sistema DEBE exponer al agente, a través de una consulta CLI, cuándo el material de origen de una sesión deja un concepto mencionado pero sin explicar (hueco/punto débil), para que el agente pueda intentar rellenar esa información antes de confirmar el elemento afectado como completo; el contenido añadido por el agente para rellenar un hueco DEBE quedar marcado de forma trazable como añadido por el sistema, distinguible del que proviene directamente del material original.
- **FR-007**: Cuando el sistema no pueda generar un elemento de checklist por falta de material suficiente, DEBE marcarlo explícitamente como pendiente de decisión del usuario (no como fallo genérico ni como completado con contenido pobre), junto con un mensaje que explique qué información falta.
- **FR-008**: El sistema DEBE permitir al usuario resolver un elemento pendiente de decisión de dos formas: aportando el material que faltaba y volviendo a ejecutar la fase, u omitiendo explícitamente ese elemento — y DEBE recordar esa decisión sin volver a preguntarla en cada ejecución posterior mientras el material no cambie.
- **FR-009**: Re-ejecutar `analyse` sobre una sesión sin cambios en su material de origen NO DEBE regenerar los elementos ya completados (idempotencia), siguiendo el mismo patrón ya usado en `cards build` (ver FR-017f de la feature 002).
- **FR-010**: El sistema DEBE implementar la fase `consolidate` (`requires = ["analyse"]`) para producir, a partir del resumen/páginas de una sesión analizada, un listado de candidatos de vocabulario para convertir en tarjetas. En esta feature, `consolidate` se limita a candidatos de tipo vocabulario; la extensión a candidatos de preguntas de gramática/conocimiento general (y su propia clave de deduplicación) queda fuera de alcance, para la feature posterior que aborde la plantilla de gramática/conocimiento general (ver Assumptions).
- **FR-011**: Al consolidar, el sistema DEBE comprobar cada candidato de vocabulario contra el almacén persistente de vocabulario ya existente (reutilizando tal cual el mecanismo de deduplicación por lema ya implementado en la feature 002, FR-009/FR-011 de `specs/002-english-eoi-flow/spec.md`) y NO DEBE proponerlo de nuevo como candidato nuevo si ya existe.
- **FR-012**: Cada candidato producido por `consolidate` DEBE conservar trazabilidad hacia la parte del análisis de la sesión de la que proviene.
- **FR-013**: `consolidate` NO DEBE completarse para las partes de la sesión afectadas por un elemento de `analyse` que siga pendiente de decisión del usuario; DEBE bloquear explícitamente esa parte en vez de consolidar con datos incompletos.
- **FR-014**: El pipeline resultante (`inventory → analyse → consolidate → vocabulary`) DEBE funcionar igual para sesiones de cualquier asignatura, no solo para el perfil de idiomas, en lo que respecta a `analyse`: el resumen/mapa mental/páginas, el checklist y la detección de huecos no pueden depender de conceptos específicos de idiomas (Principio VI, "Domain Profiles Extend, Never Contaminate"). La fase `consolidate` en sí queda acotada a candidatos de vocabulario en esta feature (ver FR-010); su generalización a otros tipos de asignatura es la única parte del pipeline que no se completa todavía en esta feature.

### Post-release (añadidas tras uso real, 2026-09-28)

- **FR-015**: Al redactar el resumen de una sesión, el agente NO DEBE limitarse a leer solo `text_material` (notas + transcripción) de `session show --json` — DEBE revisar también cada fuente de tipo imagen (`sources[].kind == "image"`, p. ej. fotos de pizarra/apuntes) directamente antes de considerar completo el material de origen, para no perder contenido estructurado (p. ej. tablas de vocabulario) que solo existe en esas fotos. Si delega esa lectura en un subagente, DEBE exigirle contenido literal/estructurado (tablas completas), no solo un resumen narrativo — encontrado tras uso real: un resumen se generó a partir de un subagente que solo devolvió una narrativa, y dos tablas completas de vocabulario en fotos de la pizarra se quedaron fuera silenciosamente.
- **FR-016**: El listado de candidatos que produce `consolidate` (FR-010) DEBE aplicar un filtrado significativamente más agresivo que "palabra ≥4 caracteres fuera de una lista mínima de palabras vacías" — encontrado tras uso real: ese filtro mínimo dejó pasar 831 candidatos de un único resumen, la inmensa mayoría ruido (p. ej. "resumen", "clase", "turnos"). `consolidate` sigue sin pretender sustituir el juicio del agente (su salida sigue siendo un borrador a curar, nunca vocabulario confirmado directamente), pero el borrador debe ser sustancialmente más útil de partida.

### Key Entities *(include if feature involves data)*

- **ChecklistItem**: un elemento individual dentro de una fase con checklist (p. ej. "resumen" dentro de `analyse`). Tiene su propio estado (pendiente / hecho / bloqueado / pendiente de decisión del usuario), su propia huella de las entradas de las que depende, y opcionalmente una decisión del usuario registrada (omitido, o material adicional aportado).
- **ClassSummary (Resumen)**: el resumen generado de una sesión — qué se trabajó, conceptos importantes, reglas, ejemplos a conservar, errores a repasar, vocabulario a estudiar — trazable al material de origen.
- **StudyMap (Mapa mental / esquema)**: la representación de repaso rápido generada de una sesión, distinta del resumen, pensada para no repetir todo su contenido.
- **ConceptPage (Página de concepto)**: una unidad de explicación concentrada de un concepto trabajado en la sesión; una sesión puede producir varias.
- **ConsolidatedCandidate (Candidato consolidado)**: un candidato de vocabulario derivado del análisis de una sesión (en esta feature; otros tipos de candidato quedan para una feature posterior), pendiente de confirmarse como `VocabularyEntry`/`LearningItem` real, con trazabilidad a la parte del análisis de la que proviene y un estado que indica si ya existe una entrada equivalente.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Un usuario puede ver, para cualquier sesión analizada, el estado individual de cada elemento de `analyse` (resumen, mapa mental, páginas) sin necesitar abrir ningún fichero manualmente — toda la información está en la consulta de estado de la sesión.
- **SC-002**: Para una sesión con material completo, el análisis produce los tres elementos (resumen, mapa mental, al menos una página) sin ninguna intervención manual del usuario.
- **SC-003**: Para una sesión con material insuficiente en un punto concreto, el usuario recibe una indicación específica de qué falta en el 100% de los casos, en vez de un fallo genérico o un elemento completado con contenido vacío/pobre.
- **SC-004**: Repetir el análisis de una sesión ya completa sin cambios en su material no vuelve a generar ningún elemento ya hecho.
- **SC-005**: Consolidar dos sesiones distintas que comparten un mismo concepto de vocabulario no produce candidatos duplicados en el listado de candidatos nuevos.

## Assumptions

- El almacén persistente de vocabulario y su deduplicación por lema, ya implementados en la feature 002 (`VocabularyEntry`, FR-009/FR-011), se reutilizan tal cual para `consolidate`; esta feature no los rediseña, solo los consume desde una fase nueva.
- Esta feature se limita al motor de fases (`analyse`/`consolidate`) y no toca la generación de tarjetas ni las plantillas existentes; los 3 gaps de imagen/tarjeta de vocabulario frente a la Guía maestra (fallback de imagen generada por IA, formato con IPA/ejemplo, resize de imagen) y la nueva plantilla de pregunta/respuesta+explicación para conceptos generales quedan fuera de alcance, para una feature posterior que se apoye en esta base ya corregida. Esa misma feature posterior es también donde `consolidate` se generalizará a candidatos que no sean vocabulario (con su propia clave de deduplicación) — en esta feature, `consolidate` solo produce y deduplica candidatos de vocabulario.
- El pipeline debe generalizarse más allá del perfil de idiomas (sesiones de cualquier asignatura), pero la validación concreta de este trabajo puede seguir haciéndose con el material de inglés/EOI ya disponible como fixture principal, dado que es el caso de uso real existente.
- "Rellenar huecos" en el material de origen (FR-006) lo hace el agente que redacta el elemento (ver Clarifications), usando su propio conocimiento/razonamiento; no requiere que la CLI/Rust llame a ningún servicio de búsqueda o generación externo. El alcance exacto de qué herramientas puede usar el agente para investigar (por ejemplo, si puede o no usar búsqueda web) se deja para el plan técnico, no para esta especificación de producto.

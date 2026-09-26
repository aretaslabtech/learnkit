# Feature Specification: Inicialización de proyecto e integración de agentes (`learnkit init`)

**Feature Branch**: `001-cli-init-agent-hooks`

**Created**: 2026-09-26

**Status**: Draft

**Input**: User description: "CLI init + hooks de agente: el comando `learnkit init` que inicializa un proyecto/carpeta y genera las Skills/instrucciones ligeras para que Codex y Claude consuman el mismo workflow (docs/05-cli-spec.md, docs/09-agent-integration.md)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inicializar un proyecto LearnKit nuevo en una carpeta (Priority: P1)

Un usuario que quiere empezar a usar LearnKit ejecuta un único comando dentro de una carpeta (nueva o existente) y obtiene una estructura de proyecto válida: configuración base, perfil por defecto, y los ficheros de integración de agentes necesarios para que Codex y/o Claude puedan operar sobre ese proyecto desde el primer momento.

**Why this priority**: Sin una inicialización fiable no existe ningún flujo posterior (sesiones, ingesta, tarjetas, exámenes); es el punto de entrada obligatorio de todo el sistema.

**Independent Test**: Puede probarse de forma aislada ejecutando el comando de inicialización en una carpeta vacía y verificando que el proyecto resultante pasa la comprobación de salud del sistema (equivalente a `learnkit doctor`) sin errores.

**Acceptance Scenarios**:

1. **Given** una carpeta vacía y una terminal interactiva, **When** el usuario inicializa un proyecto LearnKit en ella sin indicar perfil ni agentes, **Then** el sistema le presenta un menú interactivo en tres pasos (perfil, agente/s, y preferencia de shell) y, tras su selección, crea la estructura mínima de proyecto con el perfil elegido, con la integración de al menos un agente instalada (nunca cero agentes), y con la preferencia de shell guardada en la configuración del proyecto.
2. **Given** una carpeta vacía y una ejecución no interactiva (sin terminal disponible, por ejemplo invocada por un agente automatizado o un script), **When** se inicializa el proyecto sin indicar perfil ni agentes, **Then** el sistema aplica el perfil genérico, el agente por defecto, y una preferencia de shell detectada automáticamente del sistema operativo, sin preguntar nada y sin bloquear el proceso a la espera de una entrada que nunca llegará.
3. **Given** una carpeta vacía, **When** el usuario inicializa el proyecto solicitando explícitamente un perfil y/o uno o varios agentes concretos mediante opciones de línea de comandos, **Then** el sistema no muestra ningún menú interactivo y aplica directamente las opciones indicadas.
4. **Given** una carpeta que ya contiene un proyecto LearnKit inicializado, **When** el usuario repite la inicialización con las mismas opciones (o las mismas selecciones de menú), **Then** el comando no duplica ni corrompe ficheros existentes y el proyecto permanece en un estado válido (operación idempotente).
5. **Given** una carpeta que contiene ficheros no relacionados con LearnKit (por ejemplo, otro proyecto), **When** el usuario inicializa LearnKit en ella, **Then** el comando no sobrescribe ni elimina ficheros ajenos y añade únicamente la estructura propia de LearnKit.

---

### User Story 2 - Añadir integración de un agente a un proyecto ya inicializado (Priority: P2)

Un usuario que ya tiene un proyecto LearnKit en marcha decide, más adelante, empezar a usar un agente adicional (por ejemplo, añadir Claude cuando antes solo tenía Codex) y necesita generar la integración de ese agente sin repetir la inicialización completa del proyecto.

**Why this priority**: Es una operación incremental habitual (adoptar un agente nuevo) que debe funcionar sin forzar al usuario a reinicializar ni arriesgar el estado ya existente del proyecto.

**Independent Test**: Puede probarse de forma aislada sobre un proyecto ya inicializado, solicitando la integración de un agente que aún no estaba instalado, y verificando que solo se añaden los ficheros de ese agente sin tocar el resto del proyecto.

**Acceptance Scenarios**:

1. **Given** un proyecto LearnKit inicializado sin integración de agentes, **When** el usuario solicita instalar la integración de un agente concreto, **Then** se generan únicamente los ficheros de ese agente y el resto del proyecto permanece sin cambios.
2. **Given** un proyecto LearnKit con la integración de un agente ya instalada, **When** el usuario repite la instalación de ese mismo agente, **Then** el comando no duplica contenido y deja los ficheros en un estado equivalente al esperado (idempotente).

---

### User Story 3 - Verificar que un proyecto está correctamente inicializado (Priority: P3)

Un usuario o un agente automatizado quiere comprobar, antes de empezar a trabajar, si la carpeta actual contiene un proyecto LearnKit válido y bien configurado, para decidir si debe inicializarlo o puede continuar con el flujo normal.

**Why this priority**: Es soporte diagnóstico que reduce errores en cadena (sesiones, validaciones) causados por una inicialización incompleta, pero el sistema puede funcionar con una comprobación básica hasta que se prioricen otras fases.

**Independent Test**: Puede probarse ejecutando la comprobación de estado sobre una carpeta sin inicializar y sobre una carpeta correctamente inicializada, y verificando que el resultado distingue ambos casos con claridad, incluyendo un modo de salida estructurado apto para agentes automatizados.

**Acceptance Scenarios**:

1. **Given** una carpeta sin proyecto LearnKit, **When** se ejecuta la comprobación de estado, **Then** el resultado indica claramente que no hay proyecto inicializado y sugiere el comando de inicialización.
2. **Given** un proyecto LearnKit correctamente inicializado, **When** se ejecuta la comprobación de estado en modo estructurado (apto para agentes), **Then** el resultado indica que el proyecto es válido y qué agentes tienen integración instalada, en un formato que un programa pueda interpretar sin analizar texto libre.

---

### Edge Cases

- ¿Qué ocurre si se solicita inicializar con un perfil que no existe o no está soportado? El comando debe fallar de forma explícita e informativa, sin crear una estructura parcial o inconsistente.
- ¿Qué ocurre si se solicita integración para un agente no reconocido (nombre mal escrito o no soportado)? El comando debe rechazar la operación indicando qué agentes están soportados, sin generar ficheros de integración para el agente desconocido.
- ¿Qué ocurre si el proceso de inicialización se interrumpe a mitad (por ejemplo, el usuario lo cancela)? Una nueva ejecución de inicialización sobre la misma carpeta debe poder completar o corregir el estado sin quedar bloqueada por restos parciales.
- ¿Qué ocurre si el usuario no tiene permisos de escritura en la carpeta objetivo? El comando debe fallar con un mensaje claro sobre el problema de permisos, sin dejar ficheros a medio escribir.
- ¿Qué ocurre si ya existen ficheros de integración de un agente que fueron editados manualmente por el usuario? Repetir la instalación de ese agente no debe destruir silenciosamente personalizaciones sin indicárselo al usuario.
- ¿Qué ocurre si el usuario cancela el menú interactivo (por ejemplo, con Ctrl+C) antes de completar la selección de perfil o de agente? El comando debe abortar sin crear ninguna estructura parcial, igual que ante cualquier otra interrupción a mitad de proceso.
- ¿Qué ocurre si se indica `--agents` explícitamente pero con una lista vacía? El sistema debe rechazarlo como equivalente a no cumplir el mínimo de un agente, con el mismo tipo de error que un agente no soportado, en lugar de instalar el proyecto sin ningún agente.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE proporcionar una operación de inicialización que, ejecutada sobre una carpeta, cree la estructura mínima de un proyecto LearnKit (configuración de proyecto, perfil activo, y los directorios base que el resto del sistema necesita para funcionar).
- **FR-002**: La operación de inicialización DEBE ser idempotente: ejecutarla varias veces con las mismas opciones sobre la misma carpeta no debe duplicar contenido ni dejar el proyecto en un estado distinto al de una única ejecución correcta.
- **FR-003**: La operación de inicialización DEBE permitir especificar un perfil de dominio (por ejemplo, genérico, idiomas, geografía) que determine las reglas y plantillas adicionales aplicadas al proyecto. Si no se especifica ninguno y la ejecución es interactiva, el sistema DEBE presentar un menú con los perfiles disponibles (perfil genérico preseleccionado) antes de continuar; si no se especifica ninguno y la ejecución no es interactiva, se DEBE aplicar el perfil genérico por defecto sin preguntar.
- **FR-004**: La operación de inicialización DEBE instalar la integración de, como mínimo, un agente en toda ejecución exitosa (nunca un proyecto sin ningún agente). Si el usuario especifica uno o varios agentes explícitamente, se instalan esos. Si no especifica ninguno y la ejecución es interactiva, el sistema DEBE presentar un menú con los agentes soportados (Codex, Claude, u otros que el sistema soporte en el futuro) para que el usuario seleccione al menos uno antes de continuar. Si no especifica ninguno y la ejecución no es interactiva, el sistema DEBE instalar un agente por defecto predefinido sin preguntar.
- **FR-005**: El sistema DEBE proporcionar una operación independiente para instalar (o reinstalar) la integración de un agente concreto sobre un proyecto ya inicializado, sin requerir repetir la inicialización completa del proyecto.
- **FR-006**: La instalación de integración de un agente DEBE producir únicamente instrucciones/guía ligera para ese agente (equivalente a los ficheros `AGENTS.md`/`CLAUDE.md` y las Skills correspondientes descritos en la especificación de integración de agentes); el enforcement de reglas de negocio (guards, validación de fases) permanece siempre en el propio CLI, nunca en dichos ficheros de guía.
- **FR-007**: Ni la inicialización de proyecto ni la instalación de integración de agentes DEBEN sobrescribir o eliminar ficheros que no pertenezcan a la estructura conocida de LearnKit dentro de la carpeta objetivo.
- **FR-008**: Cuando se reinstale la integración de un agente sobre ficheros que el sistema detecte como modificados respecto a la plantilla original, el sistema DEBE advertir al usuario en lugar de sobrescribir silenciosamente.
- **FR-009**: El sistema DEBE proporcionar una operación de comprobación de estado que indique si la carpeta actual contiene un proyecto LearnKit válido y, si lo es, qué agentes tienen integración instalada.
- **FR-010**: La operación de comprobación de estado DEBE soportar un modo de salida estructurado (interpretable por un programa/agente) además de una salida legible por humanos, siguiendo el mismo principio de contrato estable descrito para el resto del CLI.
- **FR-011**: Si se solicita un perfil o un agente no soportado por el sistema, la operación correspondiente DEBE rechazar la solicitud con un mensaje explícito indicando las opciones válidas, sin crear una estructura parcial o inconsistente.
- **FR-012**: Toda operación de esta feature DEBE finalizar con un código de resultado estable que distinga entre éxito, error de uso/configuración, y fallo de escritura en el sistema de ficheros, de forma consistente con el resto del CLI.
- **FR-013**: El sistema DEBE distinguir automáticamente entre una ejecución interactiva (una persona frente a una terminal capaz de mostrar un menú y recibir una selección) y una ejecución no interactiva (invocada por un agente automatizado, un script, o con la entrada/salida redirigida), sin requerir que el usuario lo indique explícitamente. El modo `--json` DEBE tratarse siempre como no interactivo, incluso si por casualidad se ejecuta desde una terminal.
- **FR-014**: Cuando se muestre el menú interactivo de perfil y/o agentes, el usuario DEBE poder cancelarlo (por ejemplo, con Ctrl+C) sin que se cree ninguna estructura de proyecto parcial, de forma consistente con el resto de fallos a mitad de proceso (ver Edge Cases).
- **FR-015**: La operación de inicialización DEBE registrar una preferencia de shell (por ejemplo, `sh`/`ps`) en la configuración del proyecto, como un tercer paso del mismo menú interactivo (perfil → agente/s → shell) cuando la ejecución es interactiva, u obtenida automáticamente del sistema operativo cuando no lo es o cuando se especifica por opción de línea de comandos. Esta preferencia es puramente informativa en esta feature: no cambia el contenido de ningún fichero generado ni el comportamiento de ningún comando; queda guardada para que futuras features que sí generen artefactos dependientes de shell puedan leerla sin volver a preguntar.

### Key Entities

- **Proyecto LearnKit**: representa una carpeta inicializada con la estructura y configuración base de LearnKit (perfil activo, directorios de sesiones/perfiles/reglas). Es la unidad sobre la que operan el resto de comandos del sistema.
- **Perfil**: conjunto de reglas y plantillas de dominio (genérico, idiomas, geografía, Godot, etc.) aplicado a un proyecto durante la inicialización; determina comportamientos adicionales sin alterar el núcleo del sistema.
- **Integración de agente**: conjunto de ficheros de instrucciones ligeras y Skills asociados a un agente concreto (Codex, Claude, u otros), instalados sobre un proyecto para que ese agente pueda operar siguiendo el mismo workflow que cualquier otro agente soportado.
- **Preferencia de shell**: valor (`sh` o `ps`) guardado en la configuración del proyecto que indica qué familia de shell prefiere el usuario; no tiene efecto funcional en esta feature (ver FR-015).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Un usuario nuevo puede pasar de una carpeta vacía a un proyecto LearnKit inicializado y listo para usarse en menos de un minuto y con un único comando.
- **SC-002**: Repetir la inicialización o la instalación de integración de un agente sobre el mismo proyecto, sin cambiar opciones, produce el mismo resultado observable en el 100% de los casos (idempotencia verificable).
- **SC-003**: Un proyecto inicializado con integración para dos agentes distintos (por ejemplo Codex y Claude) permite que ambos agentes completen la misma tarea de ejemplo siguiendo el mismo workflow, sin necesidad de instrucciones específicas por agente más allá de sus ficheros de integración.
- **SC-004**: El 100% de los intentos de inicializar con un perfil o agente no soportado terminan en un mensaje de error claro y ninguno deja una estructura de proyecto parcial o corrupta en la carpeta.
- **SC-005**: La comprobación de estado permite a un agente automatizado determinar, sin intervención humana ni análisis de texto libre, si debe inicializar el proyecto o puede continuar trabajando en él.
- **SC-006**: El 100% de los proyectos inicializados con éxito (interactiva o no interactivamente) terminan con al menos un agente instalado; ningún proyecto queda en un estado "sin ningún agente".
- **SC-007**: Un usuario que ejecuta la inicialización sin ninguna opción desde una terminal puede completar la selección de perfil y agente y obtener el proyecto listo en menos de 30 segundos de interacción con el menú.

## Assumptions

- La carpeta objetivo es local y accesible por el usuario/proceso que ejecuta la inicialización; no se contempla inicialización remota en esta feature.
- Los agentes soportados en esta primera versión son Codex y Claude, siguiendo `docs/09-agent-integration.md`; añadir soporte para agentes adicionales es una extensión futura que reutiliza el mismo mecanismo de instalación de integración.
- La lista de perfiles disponibles en el momento de esta feature es la definida en `docs/06-profiles.md`; el perfil "genérico" es el valor por defecto cuando no se especifica ninguno.
- El "modo estructurado" de salida referido en FR-010 es el mismo mecanismo `--json` descrito de forma transversal para todo el CLI en `docs/05-cli-spec.md`, y no una funcionalidad exclusiva de esta feature.
- La detección de ficheros de integración de agente modificados manualmente (FR-008) se basa en comparar contra el contenido de la plantilla original; no se requiere en esta feature un sistema de control de versiones propio para dichos ficheros.
- El agente por defecto usado en ejecuciones no interactivas sin `--agents` (FR-004) es **Claude**, siguiendo la configuración de herramientas ya usada en este mismo repositorio (`.specify/init-options.json` → `"ai": "claude"`); es un valor por defecto, no una restricción — el usuario puede pedir explícitamente Codex u otro agente soportado con `--agents`.
- La detección de "ejecución interactiva" (FR-013) se basa en si tanto la entrada estándar como la salida estándar del proceso son una terminal (TTY); no se contempla en esta feature una configuración explícita para forzar un modo u otro más allá de pasar `--profile`/`--agents` (que ya evitan el menú) o `--json` (que siempre fuerza el modo no interactivo).
- El menú interactivo de esta feature es un selector de terminal simple (lista con selección por teclado), sin dependencia de una interfaz gráfica ni de un navegador; no se contempla en esta feature ningún otro canal de interacción (web, GUI).
- La detección automática de shell por defecto (FR-015) usa una heurística simple basada en el sistema operativo (`ps` en Windows, `sh` en cualquier otro caso); no se contempla en esta feature detectar el shell realmente activo en el proceso padre (por ejemplo, distinguir bash de zsh), ya que solo hay dos valores soportados por ahora.
- La preferencia de shell es deliberadamente informativa/sin efecto en esta feature (FR-015); no se crea aquí ningún generador de scripts `.sh`/`.ps1` — LearnKit ya expone su funcionalidad como un binario multiplataforma, a diferencia de Spec Kit, que sí depende de scripts por shell al no tener binario propio.

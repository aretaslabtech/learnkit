# Data Model: `learnkit init` y hooks de integración de agentes

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Research**: [research.md](./research.md)

Entidades relevantes para esta feature (ver también "Key Entities" en `spec.md`). Se describen a nivel conceptual/de dominio, sin acoplar a un formato de serialización concreto más allá de lo ya fijado en `research.md`.

## Project (Proyecto LearnKit)

Representa una carpeta inicializada como proyecto LearnKit.

| Campo | Tipo | Descripción | Reglas |
|-------|------|-------------|--------|
| `root_path` | path | Carpeta raíz del proyecto (donde vive `learnkit.toml`). | Debe ser una carpeta con permisos de escritura; no se persiste (se deriva del cwd/arg en cada invocación). |
| `profile_id` | string | Identificador del perfil activo (p.ej. `generic`, `language`, `geography`). | Debe existir entre los perfiles soportados (`docs/06-profiles.md`); por defecto `generic`. |
| `shell_preference` | string (`sh`\|`ps`) | Preferencia de shell guardada durante `init` (FR-015). | Puramente informativa en esta feature; no condiciona ningún comportamiento. Por defecto, detectada automáticamente del sistema operativo. |
| `initialized_at` | timestamp | Momento de la primera inicialización exitosa. | Se fija una sola vez; no cambia en reinstalaciones/idempotencia. |
| `schema_version` | string/semver | Versión del layout de proyecto generado por `init`. | Permite detectar proyectos creados por una versión anterior del CLI en el futuro; fuera de alcance evolucionar/migrar en esta feature. |
| `installed_agents` | list<AgentIntegration> | Agentes con integración instalada en este proyecto. | Puede estar vacía; se deriva inspeccionando el filesystem (no requiere estado adicional oculto — Principio IV, Hard Guards: el estado real es el filesystem, no una declaración). |

**Validación**: un `Project` es "válido" (para `status`) si y solo si existen y son legibles `learnkit.toml` y el directorio `.learnkit/` con su `profile_id` resuelto a un perfil soportado. Si `learnkit.toml` existe pero referencia un `profile_id` no soportado, el proyecto se reporta como inválido con un código de error explícito, nunca como parcialmente válido.

## Profile (Perfil)

Conjunto de reglas/plantillas de dominio aplicado a un proyecto.

| Campo | Tipo | Descripción | Reglas |
|-------|------|-------------|--------|
| `id` | string | Identificador único (`generic`, `language`, `geography`, `godot`, ...). | Debe pertenecer al catálogo soportado por el CLI en el momento de la inicialización. |
| `display_name` | string | Nombre legible para salida humana. | — |
| `template_files` | list<path> | Ficheros/plantillas que este perfil copia bajo `.learnkit/profiles/<id>/` al inicializar. | Estáticos por versión del CLI; no se generan dinámicamente en esta feature. |

**Relación**: `Project.profile_id` → exactamente un `Profile`. Un proyecto no tiene múltiples perfiles activos simultáneamente en esta feature (multi-perfil, si se necesitara, queda fuera de alcance).

## AgentIntegration (Integración de agente)

Conjunto de ficheros de guía/Skills instalados para un agente concreto sobre un proyecto.

| Campo | Tipo | Descripción | Reglas |
|-------|------|-------------|--------|
| `agent_id` | string | Identificador del agente (`codex`, `claude`, ...). | Debe pertenecer al catálogo soportado (FR-011); rechazar con mensaje explícito si no. |
| `installed_files` | list<AgentFile> | Ficheros generados para este agente (p.ej. `AGENTS.md`, `.agents/skills/**/SKILL.md` para Codex; `CLAUDE.md`, Skills equivalentes para Claude). | Cada fichero se rastrea individualmente para poder detectar modificación (FR-008). |
| `template_version` | string/semver | Versión de la plantilla usada para generar `installed_files`. | Se usa junto al hash (ver `AgentFile`) para decidir si una reinstalación es un no-op, una actualización limpia, o requiere advertencia. |

**Relación**: `Project.installed_agents` → 0..N `AgentIntegration` (uno por agente instalado). Instalar el mismo agente dos veces con el mismo `template_version` y sin modificaciones manuales es idempotente (FR-002) y no crea una segunda `AgentIntegration`, solo confirma/re-escribe el mismo estado.

## AgentFile (Fichero de integración de agente)

Un fichero individual gestionado como parte de una `AgentIntegration`.

| Campo | Tipo | Descripción | Reglas |
|-------|------|-------------|--------|
| `relative_path` | path | Ruta relativa a la raíz del proyecto (p.ej. `AGENTS.md`, `.agents/skills/learnkit-session/SKILL.md`). | Nunca fuera de la raíz del proyecto. |
| `template_hash` | string (SHA-256) | Hash del contenido esperado según la plantilla/versión actual del CLI. | Ver decisión en `research.md §5`. |
| `on_disk_hash` | string (SHA-256) | Hash del contenido actualmente presente en disco (si el fichero existe). | Calculado en tiempo de ejecución, no persistido como fuente de verdad aparte del propio fichero. |

**Estado derivado** (no persistido, calculado en cada ejecución — coherente con Hard Guards):

- `missing` — el fichero no existe → instalar limpio.
- `up_to_date` — `on_disk_hash == template_hash` → no-op idempotente.
- `modified_by_user` — el fichero existe y su hash no coincide con ningún `template_hash` conocido de una versión anterior instalada → advertir en vez de sobrescribir (FR-008).
- `stale_template` — el fichero coincide con el hash de una versión anterior de la plantilla (no modificado por el usuario, pero desactualizado) → se puede actualizar sin advertencia.

## StatusReport (Resultado de `learnkit status`)

Estructura de salida (humana y `--json`) que expone el estado de un `Project`.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `ok` | bool | `true` si la carpeta contiene un `Project` válido. |
| `code` | string | Código estable (`PROJECT_NOT_INITIALIZED`, `PROJECT_READY`, `PROFILE_INVALID`, ...), mismo estilo que el contrato general `docs/05-cli-spec.md §13`. |
| `profile_id` | string \| null | Perfil activo si el proyecto es válido. |
| `installed_agents` | list<string> | IDs de agentes con integración instalada (derivado de `AgentIntegration`, no de una lista declarada). |
| `shell_preference` | string \| null | Preferencia de shell guardada en `learnkit.toml` (FR-015), leída del `Project` si es válido. |
| `issues` | list<{check, message}> | Problemas detectados (p.ej. perfil no soportado, fichero de agente modificado pendiente de revisión). |

No hay transición de estados en el sentido de máquina de estados (esta feature no introduce un workflow de fases); el `StatusReport` es siempre una foto recalculada del filesystem, nunca un valor cacheado de una ejecución anterior (Principio IV).

# Contract: comandos CLI de esta feature

**Feature**: [spec.md](./spec.md) | **Data model**: [data-model.md](./data-model.md)

Este contrato cubre exclusivamente los comandos introducidos/afectados por esta feature. El resto de comandos listados en `docs/05-cli-spec.md` (session, cards, export, etc.) no se ven modificados y quedan fuera de alcance.

## `learnkit init [PATH] [--profile <id>] [--agents <id>[,<id>...>] [--shell <sh|ps>] [--json]`

**Descripción**: inicializa (o reafirma, si ya existe) un proyecto LearnKit en `PATH` (por defecto, el directorio actual).

**Argumentos**:
- `PATH` (opcional, posicional): carpeta objetivo. Por defecto `.`.
- `--profile <id>` (opcional): perfil a activar. Por defecto `generic`.
- `--agents <id>[,<id>...]` (opcional): lista de agentes para los que instalar integración en la misma ejecución. Por defecto ninguno.
- `--shell <sh|ps>` (opcional): preferencia de shell a guardar en `learnkit.toml` (FR-015). Si se omite, se resuelve interactivamente (menú) o automáticamente (fallback no interactivo), igual que perfil/agentes.
- `--json` (opcional): salida estructurada en lugar de texto legible.

**Comportamiento**:
- Idempotente (FR-002): repetir con las mismas opciones (o las mismas selecciones de menú) no cambia el resultado observable.
- No sobrescribe ficheros ajenos a la estructura LearnKit (FR-007).
- Si `--profile` no es soportado → falla sin crear estructura parcial (FR-011).
- Si `--agents` incluye un agente no soportado, o se pasa `--agents` con lista vacía → falla sin instalar integración de ningún agente de la lista, incluidos los válidos (fallo atómico a nivel de comando; evita estados "a medias" respecto a la lista solicitada).
- **Mínimo un agente siempre (FR-004)**: toda ejecución exitosa deja instalado al menos un agente; nunca es válido un proyecto inicializado sin ningún agente.
- **Selección interactiva (FR-003/FR-004/FR-013/FR-014/FR-015)**: si ninguno de `--profile`/`--agents`/`--shell` se especificó, y stdin+stdout son un TTY (detección per `research.md` §9) y no se pasó `--json`, el comando muestra un menú en **tres pasos** (perfil, con `generic` preseleccionado → agente/s, selección múltiple con mínimo uno → shell, con la detección automática de `research.md` §11 preseleccionada) antes de escribir nada. Si el usuario cancela el menú (p.ej. Ctrl+C) en cualquiera de los tres pasos, el comando termina sin crear ninguna estructura parcial, con exit code `2`.
- **Fallback no interactivo**: si no es interactivo (sin TTY, o `--json`) y no se especificaron `--profile`/`--agents`/`--shell`, se aplica el perfil `generic`, se instala el agente por defecto (`claude`, ver `spec.md` → Assumptions), y se guarda la preferencia de shell detectada automáticamente del sistema operativo (`research.md` §11) — todo sin preguntar nada.
- `--profile`/`--agents`/`--shell` explícitos siempre tienen prioridad sobre el menú interactivo: si se pasan, nunca se muestra el menú, incluso en una terminal interactiva. Cada opción se resuelve de forma independiente: si solo se pasa `--shell` (por ejemplo), el menú sigue mostrándose para perfil y agentes pero no para shell.
- La preferencia de shell (FR-015) es puramente informativa en esta feature: no afecta al contenido de `AGENTS.md`/`CLAUDE.md`/`SKILL.md` ni a ningún otro comportamiento del CLI.

**Salida `--json` (éxito)**:
```json
{
  "ok": true,
  "code": "PROJECT_INITIALIZED",
  "profile_id": "generic",
  "installed_agents": ["codex", "claude"],
  "shell_preference": "sh"
}
```

**Salida `--json` (fallo — perfil no soportado)**:
```json
{
  "ok": false,
  "code": "PROFILE_UNSUPPORTED",
  "profile_id": "unknown-profile",
  "supported_profiles": ["generic", "language", "geography", "godot"]
}
```

**Exit codes** (coherentes con `docs/05-cli-spec.md §14`): `0` éxito; `2` uso/config inválida (perfil o agente no soportado, argumentos inválidos); `50` fallo de filesystem (p.ej. sin permisos de escritura).

---

## `learnkit agent install <agent_id> [--path <PATH>] [--json]`

**Descripción**: instala o reinstala la integración de un agente concreto sobre un proyecto ya inicializado, sin repetir `init`.

**Argumentos**:
- `agent_id` (posicional, obligatorio): `codex` | `claude` (catálogo soportado en el momento de esta feature).
- `--path <PATH>` (opcional): carpeta del proyecto. Por defecto `.`.
- `--json` (opcional): salida estructurada.

**Precondición**: `PATH` debe ser ya un `Project` válido (ver `data-model.md`); si no lo es, falla indicando que se ejecute `learnkit init` primero.

**Comportamiento**:
- Si el agente no existe todavía instalado → instalación limpia (FR-005).
- Si ya está instalado y sin modificaciones manuales → no-op idempotente (FR-002).
- Si ya está instalado y con ficheros detectados como modificados manualmente → no sobrescribe, reporta advertencia con la lista de ficheros afectados (FR-008).

**Salida `--json` (instalación limpia o confirmación idempotente)**:
```json
{
  "ok": true,
  "code": "AGENT_INTEGRATION_INSTALLED",
  "agent_id": "claude",
  "installed_files": [".agents/skills/learnkit-session/SKILL.md", "AGENTS.md"]
}
```

**Salida `--json` (bloqueado por modificación manual)**:
```json
{
  "ok": false,
  "code": "AGENT_FILES_MODIFIED",
  "agent_id": "claude",
  "modified_files": ["CLAUDE.md"]
}
```

**Exit codes**: `0` éxito (incluye no-op idempotente); `2` uso/config inválida (agente no soportado, proyecto no inicializado); `20` bloqueado por ficheros modificados manualmente (mismo rango semántico que "guard blocked" en `docs/05-cli-spec.md §14`, adaptado: aquí el "guard" es la protección contra sobrescritura silenciosa, FR-008); `50` fallo de filesystem.

---

## `learnkit status [--path <PATH>] [--json]`

**Descripción**: reporta si `PATH` contiene un proyecto LearnKit válido y, si lo es, su perfil y agentes instalados (FR-009/FR-010).

**Argumentos**:
- `--path <PATH>` (opcional): carpeta a inspeccionar. Por defecto `.`.
- `--json` (opcional): salida estructurada — ver `StatusReport` en `data-model.md`.

**Salida `--json` (no inicializado)**:
```json
{
  "ok": false,
  "code": "PROJECT_NOT_INITIALIZED",
  "profile_id": null,
  "installed_agents": [],
  "issues": []
}
```

**Salida `--json` (válido)**:
```json
{
  "ok": true,
  "code": "PROJECT_READY",
  "profile_id": "geography",
  "installed_agents": ["codex"],
  "shell_preference": "ps",
  "issues": []
}
```

**Exit codes**: `0` siempre que el comando se ejecute correctamente, **incluso si `ok: false`** (no inicializado es un resultado válido de una consulta de estado, no un error de uso); `2` si `PATH` no existe o no es accesible; `50` fallo de filesystem al leer.

**Nota de contrato transversal**: los tres comandos comparten el mismo shape base `{"ok", "code", ...}` que el resto del CLI (`docs/05-cli-spec.md §13`), para que los Skills de agente (`docs/09-agent-integration.md §8`) puedan tratarlos con el mismo parser JSON sin lógica especial por comando.

# Quickstart: validar `learnkit init` y la integración de agentes

**Feature**: [spec.md](./spec.md) | **Contracts**: [contracts/cli-commands.md](./contracts/cli-commands.md)

Esta guía valida end-to-end los criterios de aceptación de la feature (User Stories 1-3). Asume que existe un binario `learnkit` construido (`cargo build`) y accesible en el `PATH` de la sesión, o se invoca vía `cargo run -p learnkit-cli --`.

## Prerrequisitos

- Rust toolchain instalado (ver `rust-toolchain`/`Cargo.toml` del workspace una vez creado en la fase de implementación).
- Una carpeta vacía de trabajo, p.ej. `mkdir /tmp/lk-demo && cd /tmp/lk-demo`.

## Escenario 1 — Inicializar un proyecto nuevo, ejecución no interactiva (User Story 1)

Este escenario se ejecuta como script (no interactivo: sin TTY), por lo que `learnkit init` sin opciones aplica el fallback de FR-003/FR-004 (perfil `generic` + agente por defecto `claude`) sin mostrar ningún menú.

```bash
learnkit status --json
# Esperado: {"ok": false, "code": "PROJECT_NOT_INITIALIZED", ...}

learnkit init
# Esperado: crea learnkit.toml, .learnkit/workflow.toml, .learnkit/profiles/generic/, AGENTS.md/CLAUDE.md (agente por defecto: claude)

learnkit status --json
# Esperado: {"ok": true, "code": "PROJECT_READY", "profile_id": "generic", "installed_agents": ["claude"], "issues": []}

learnkit init
# Esperado: mismo resultado que la primera vez (idempotente, FR-002); ningún fichero duplicado.
```

**Criterio de éxito**: SC-001 (menos de un minuto, un único comando), SC-002 (idempotencia observable) y SC-006 (nunca cero agentes).

## Escenario 1b — Inicializar un proyecto nuevo, ejecución interactiva (User Story 1)

Este escenario requiere una terminal real (TTY) y no puede automatizarse como script; se valida manualmente durante la implementación.

```text
$ learnkit init
? Selecciona un perfil (Generic preseleccionado) › Generic / Language / Geography / Godot
? Selecciona uno o más agentes (mínimo 1) › [x] Codex  [ ] Claude
? Selecciona tu shell preferida (ps preseleccionado en Windows, sh en el resto) › sh / ps
# Esperado: tras confirmar los tres pasos, se crea el proyecto con el perfil, los agentes y la preferencia de shell elegidos (guardada en learnkit.toml, sin efecto funcional en esta feature), sin volver a preguntar.

$ learnkit init
# Ctrl+C durante el menú de perfil
# Esperado: exit code 2; ni learnkit.toml ni .learnkit/ existen (sin estructura parcial, FR-014).
```

**Criterio de éxito**: SC-006 (mínimo un agente), SC-007 (menú completable en menos de 30s de interacción).

## Escenario 2 — Inicializar con perfil y agentes en una sola ejecución (User Story 1, variante)

```bash
mkdir /tmp/lk-demo-2 && cd /tmp/lk-demo-2
learnkit init --profile geography --agents codex,claude

learnkit status --json
# Esperado: {"ok": true, "code": "PROJECT_READY", "profile_id": "geography", "installed_agents": ["codex", "claude"], "shell_preference": "<detectado del OS o el pasado con --shell>", "issues": []}

ls AGENTS.md CLAUDE.md .agents/skills 2>&1
# Esperado: los tres existen (ficheros de integración de ambos agentes).
```

**Criterio de éxito**: SC-003 (dos agentes distintos, mismo workflow, sin instrucciones ad-hoc por agente).

## Escenario 3 — Añadir un agente más adelante (User Story 2)

```bash
cd /tmp/lk-demo   # el del Escenario 1, sin agentes instalados
learnkit agent install claude
# Esperado: {"ok": true, "code": "AGENT_INTEGRATION_INSTALLED", "agent_id": "claude", ...}

learnkit agent install claude
# Esperado: mismo resultado (no-op idempotente); ningún fichero duplicado ni marca de "instalado dos veces".

learnkit status --json
# Esperado: "installed_agents": ["claude"]
```

## Escenario 4 — Rechazar perfil/agente no soportado (Edge cases)

```bash
mkdir /tmp/lk-demo-3 && cd /tmp/lk-demo-3
learnkit init --profile no-existe
# Esperado: exit code 2, {"ok": false, "code": "PROFILE_UNSUPPORTED", ...}; ls -la no debe mostrar learnkit.toml ni .learnkit/ (sin estructura parcial).

learnkit init
learnkit agent install agente-inventado
# Esperado: exit code 2, {"ok": false, "code": ...}; no se crea ningún fichero de integración.
```

**Criterio de éxito**: SC-004 (100% de intentos con perfil/agente no soportado terminan en error claro sin estructura parcial).

## Escenario 5 — Proteger ficheros de agente editados manualmente (FR-008)

```bash
cd /tmp/lk-demo-2
echo "\n<!-- nota personal -->" >> CLAUDE.md
learnkit agent install claude
# Esperado: exit code 20, {"ok": false, "code": "AGENT_FILES_MODIFIED", "modified_files": ["CLAUDE.md"]};
#           CLAUDE.md conserva la nota personal (no se sobrescribe).
```

## Escenario 6 — No tocar ficheros ajenos (FR-007)

```bash
mkdir /tmp/lk-demo-4 && cd /tmp/lk-demo-4
echo "contenido de otro proyecto" > README.md
learnkit init
cat README.md
# Esperado: README.md sigue exactamente igual; learnkit.toml y .learnkit/ conviven junto a él.
```

## Resultado esperado global

Al completar los 6 escenarios, todos los criterios de éxito de `spec.md` (SC-001 a SC-005) y los edge cases documentados quedan demostrados sin necesidad de inspeccionar el código fuente — solo observando el filesystem y la salida `--json` del CLI.

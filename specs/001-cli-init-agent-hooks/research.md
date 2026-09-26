# Research: `learnkit init` y hooks de integración de agentes

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Todos los `NEEDS CLARIFICATION` del Technical Context del plan se resolvieron con valores por defecto razonables derivados de `learnkit-implementation-spec/docs/`; no quedan desconocidos abiertos. Este documento registra las decisiones técnicas concretas tomadas para poder empezar el diseño de datos/contratos (Phase 1).

## 1. Parsing de comandos CLI

- **Decision**: usar `clap` con la API `derive` (structs `#[derive(Parser)]`/`#[derive(Subcommand)]`) para `learnkit init`, `learnkit agent install <agent>` y `learnkit status`.
- **Rationale**: es el estándar de facto en el ecosistema Rust para CLIs con subcomandos, exit codes y flags booleanos/`--json`; minimiza código repetitivo frente a la API builder y facilita añadir subcomandos futuros (`session`, `cards`, etc.) sin reestructurar `learnkit-cli`.
- **Alternatives considered**: `clap` builder API (más verboso, sin beneficio aquí); `argh`/`pico-args` (más ligeros pero sin soporte cómodo de subcomandos anidados ni generación de ayuda, que el CLI necesitará para todos los comandos de `docs/05-cli-spec.md`, no solo esta feature).

## 2. Formato del fichero de configuración de proyecto

- **Decision**: `learnkit.toml` en TOML (vía `serde` + `toml`), tal como ya lo fija `docs/10-storage-git.md §2`.
- **Rationale**: ya es una decisión tomada en la especificación de origen (nombre de fichero y formato); TOML es legible, diffable y es el formato estándar de configuración en el ecosistema Rust (coherente con `Cargo.toml`, `.learnkit/workflow.toml`).
- **Alternatives considered**: YAML (usado para `session.yaml` y entidades de dominio, pero no para la config raíz del proyecto en la spec de origen); JSON (peor para edición manual/diffs).

## 3. Salida `--json` para agentes

- **Decision**: reutilizar `serde_json` para serializar una estructura estable `{"ok": bool, "code": string, ...}` en `learnkit status --json`, siguiendo el contrato ya definido en `docs/05-cli-spec.md §13` (mismo shape que `PHASE_BLOCKED` para otros comandos, adaptado a `PROJECT_NOT_INITIALIZED` / `PROJECT_READY`).
- **Rationale**: mantiene un único contrato JSON transversal a todo el CLI (Principio de "salida estructurada" en `docs/09-agent-integration.md §8`) en lugar de inventar un formato ad-hoc para esta feature.
- **Alternatives considered**: texto legible parseado por heurística — rechazado explícitamente por la spec de agentes ("Nunca parsear texto decorativo si existe contrato JSON").

## 4. Plantillas de integración de agentes (`AGENTS.md`, `CLAUDE.md`, `SKILL.md`)

- **Decision**: embeber las plantillas fuente (bajo `skills/` y ficheros raíz de plantilla) en el binario en tiempo de compilación (`include_str!` o un crate de embedding como `rust-embed`), en lugar de leerlas de una instalación externa en tiempo de ejecución.
- **Rationale**: garantiza que `learnkit init`/`agent install` funcionen de forma reproducible sin depender de que el usuario tenga acceso a los ficheros fuente de `learnkit-implementation-spec`; encaja con "binario distribuible" (ADR-001 de la constitución).
- **Alternatives considered**: plantillas externas versionadas junto al binario en un directorio de instalación — más flexible para personalización global, pero añade una fuente de fallos (ruta no encontrada) fuera del alcance de esta feature; se puede añadir más adelante sin romper el contrato del comando.

## 5. Detección de ficheros de agente modificados manualmente (FR-008)

- **Decision**: al instalar/reinstalar la integración de un agente, calcular un hash (SHA-256, crate `sha2`) del contenido actual del fichero destino y compararlo con el hash conocido de la plantilla original correspondiente a la versión instalada; si no coincide y el fichero ya existía, advertir en lugar de sobrescribir.
- **Rationale**: es el mecanismo más simple y determinista para diferenciar "fichero generado sin tocar" de "fichero editado por el usuario", sin necesitar un sistema de control de versiones propio (coherente con la Assumption ya registrada en la spec).
- **Alternatives considered**: comparación byte a byte contra la plantilla en disco (equivalente pero sin necesitar persistir el hash, a costa de tener que empaquetar siempre la plantilla exacta usada en la instalación original — se prefiere el hash porque permite versionar la plantilla instalada sin guardar su contenido completo).

## 6. Estrategia de escritura idempotente/atómica

- **Decision**: para cada fichero gestionado por `init`/`agent install`, escribir primero a un fichero temporal en el mismo directorio y hacer `rename` atómico sobre el destino final, siguiendo el patrón ya descrito en `docs/10-storage-git.md §5` (temp → fsync si aplica → rename → recomputar estado).
- **Rationale**: evita dejar ficheros a medio escribir si el proceso se interrumpe (edge case de la spec), y es coherente con el resto del sistema de storage.
- **Alternatives considered**: escritura directa con truncado — rechazada porque un fallo a mitad deja el fichero corrupto/parcial, violando FR-007/edge cases de la spec.

## 7. Límites de crates para esta feature

- **Decision**: implementar esta feature repartida en `learnkit-cli` (comandos/`--json`/exit codes), `learnkit-core` (tipos de error compartidos), `learnkit-store` (escritura idempotente/atómica de `learnkit.toml` y `.learnkit/`), `learnkit-profile` (resolución/validación de perfil) y `learnkit-agent` (plantillas + detección de modificación); no crear todavía `learnkit-workflow`, `learnkit-cards`, etc.
- **Rationale**: `docs/02-architecture.md` permite explícitamente empezar V1 con 4-5 crates y separar solo cuando existan límites reales; los crates de dominio no relacionados (cards, media, assessment, export, transcripción) no tienen ninguna responsabilidad en `init`/`agent install`/`status`.
- **Alternatives considered**: un único crate monolítico `learnkit` — más rápido a corto plazo pero contradice la separación `learnkit-cli` "sin lógica de dominio" ya fijada como responsabilidad explícita en `docs/02-architecture.md §3`.

## 8. Estrategia de testing

- **Decision**: tests de integración en `tests/integration/` que invocan los comandos sobre directorios temporales (`tempfile` crate), afirmando el árbol de ficheros resultante, los exit codes (`docs/05-cli-spec.md §14`) y la salida `--json` de `status`.
- **Rationale**: esta feature se valida principalmente por su efecto observable en el filesystem y por el contrato `--json`/exit codes, no por lógica de negocio interna compleja; los tests de integración capturan directamente los criterios de aceptación de la spec (idempotencia, no sobrescritura, detección de modificación).
- **Alternatives considered**: solo unit tests de las funciones internas de `learnkit-store`/`learnkit-agent` — insuficiente porque no verificaría el contrato real expuesto por `learnkit-cli` ni los exit codes end-to-end.

## 9. Detección de ejecución interactiva vs no interactiva (FR-013)

- **Decision**: usar el crate `is-terminal` para comprobar si tanto stdin como stdout son un TTY; tratar la ejecución como interactiva solo si ambas lo son, y como no interactiva en cualquier otro caso (incluyendo siempre `--json`, aunque casualmente se ejecute desde una terminal).
- **Rationale**: es la señal estándar y ya usada en el ecosistema Rust (la misma que usan `cargo`, `git`, etc.) para decidir si se puede mostrar un prompt; comprobar ambos flujos evita mostrar un menú cuando la salida está redirigida a un fichero/pipe aunque stdin sea interactivo (o viceversa).
- **Alternatives considered**: variable de entorno explícita (`LEARNKIT_INTERACTIVE=0/1`) — se descarta como mecanismo *por defecto* porque añadiría una configuración que el usuario tendría que conocer; queda abierta como posible vía de override futura si aparece un caso real, pero no es necesaria para esta feature.

## 10. Menú de selección interactiva (perfil y agente)

- **Decision**: usar el crate `dialoguer` para el menú en dos pasos (selección de perfil, después selección de agente/s, mínimo uno) cuando la ejecución es interactiva (Decisión 9), con el perfil `generic` preseleccionado y con soporte de cancelación (Ctrl+C) que aborta sin escribir nada, coherente con la escritura atómica de la Decisión 6.
- **Rationale**: `dialoguer` es una librería madura y ampliamente usada en CLIs Rust para selects/multiselects en terminal, sin necesitar una dependencia de TUI completa (como `ratatui`) que sería sobre-ingeniería para un menú de selección simple de dos pasos.
- **Alternatives considered**: `inquire` (alternativa equivalente, API similar; se prefiere `dialoguer` por ser la opción más establecida y con menos superficie de API que aprender para este caso de uso concreto); implementar un selector propio sobre `crossterm` — rechazado por reinventar funcionalidad ya resuelta y bien probada.

## 11. Detección automática de la preferencia de shell (FR-015)

- **Decision**: en modo no interactivo (o cuando el usuario no pasa `--shell`), usar una heurística simple basada en el sistema operativo de compilación: `ps` si `cfg!(target_os = "windows")`, `sh` en cualquier otro caso. En modo interactivo, mostrar este valor como preselección del tercer paso del menú, permitiendo cambiarlo.
- **Rationale**: la preferencia de shell es puramente informativa en esta feature (no genera scripts ni cambia el comportamiento de ningún comando — ver `spec.md` → Assumptions); una heurística de dos valores basada en el OS de compilación es la opción más simple y suficiente para ese propósito. Sobre-diseñar la detección (por ejemplo, inspeccionar `$SHELL`/`$PSModulePath` del proceso padre para distinguir bash/zsh/fish) no aporta valor mientras no exista ningún consumidor real de este dato.
- **Alternatives considered**: leer la variable de entorno `SHELL` (Unix) o `PSModulePath`/`COMSPEC` (Windows) para detectar el shell real del proceso padre — descartado por ahora porque añade complejidad y casos borde (shells no estándar, contenedores, CI) para un campo que hoy no tiene ningún efecto; queda documentado aquí como la vía a tomar el día que exista un consumidor real (p.ej. generación de scripts específicos por shell en una feature futura).

## 7b. Verificación de licencias (T036, post-implementación)

Ejecutado `cargo metadata` sobre el workspace ya implementado (`Cargo.lock` fijado): todas las dependencias de terceros son MIT, Apache-2.0, Unicode-3.0, Unlicense, o combinaciones "OR" de esas (incluyendo `r-efi`, que ofrece LGPL-2.1-or-later solo como alternativa dentro de un OR con MIT/Apache-2.0, nunca como obligación). **Ninguna dependencia usa AGPL** — cumple la restricción de arquitectura de `constitution.md`. Los 5 paquetes sin `license` en `cargo metadata` son los propios crates de LearnKit (`learnkit-core`, `learnkit-store`, `learnkit-profile`, `learnkit-agent`, `learnkit-cli`), pendientes de fijar su licencia en un paso posterior (fuera de alcance de esta feature).

## Resultado

Todos los desconocidos del Technical Context quedan resueltos con las decisiones anteriores. Ninguna requiere una nueva ronda de `/speckit-clarify` sobre la spec (no son ambigüedades de producto, son decisiones de implementación dentro del margen ya permitido por la constitución y los docs de arquitectura existentes).

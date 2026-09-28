# Contract: Skill de agente `learnkit-analyse`

**Feature**: [spec.md](./spec.md) | **Data model**: [data-model.md](./data-model.md) | **Research**: [research.md](./research.md)

Análoga a `learnkit-language` (feature 002), pero genérica: sin ninguna instrucción específica de idiomas (Principio VI). Se registra en `crates/learnkit-agent/templates/skills/learnkit-analyse/SKILL.md` vía el mismo mecanismo de plantillas ya usado por `learnkit-language` (`crates/learnkit-agent/src/templates.rs`).

## Entrada que el agente lee

- `learnkit session show --session <id> --json`: notas + segmentos de la transcripción activa (ya expuesto desde la feature 002, sin cambios de contrato).
- `learnkit status --session <id> --json`: estado actual del checklist de `analyse` — qué elementos faltan, cuáles están `pending_user_decision` y por qué (para no repetir trabajo ya hecho ni ignorar un elemento marcado como incompleto).

## Lo que la Skill redacta (nunca lo que persiste el CLI por sí mismo)

- Un resumen (qué se trabajó, conceptos importantes, reglas, ejemplos a conservar, errores a repasar, vocabulario a estudiar).
- Un mapa mental/esquema de repaso rápido, deliberadamente distinto del resumen (no una repetición).
- Una o más páginas de concepto concentrado.
- Cuando detecta que el material deja un concepto mencionado pero sin explicar, puede rellenarlo con su propio conocimiento — pero DEBE declararlo como tal (contrato de `analyse summary set --filled-gap`), nunca mezclarlo sin distinción con el contenido derivado directamente del material.
- Cuando el material es insuficiente para un elemento, la Skill NO debe inventar contenido de relleno para que el elemento "pase" — debe usar `analyse <item> flag-pending --reason "..."` y explicárselo a la persona en la conversación.

## Lo que la CLI valida al confirmar (Hard Guards, Principio IV)

- Que el fichero de contenido exista y no esté vacío.
- Que un `--filled-gap` declarado tenga tanto concepto como nota (no vacíos).
- Que el `mapa mental` y el `resumen` de una misma sesión no sean el mismo contenido byte a byte (detección mínima de "no es una repetición", no un juicio de calidad).
- La CLI **nunca** evalúa la calidad de la redacción — eso es responsabilidad del agente/usuario; el guard es puramente estructural y de trazabilidad, igual que ya ocurre con `learn vocabulary add` en la feature 002.

## Salida de la Skill hacia la persona

La Skill informa a la persona qué elementos confirmó, cuáles marcó como pendientes de decisión y por qué, y le pregunta explícitamente (en la conversación, no en un prompt de terminal) si prefiere aportar más material o que el elemento quede omitido — la decisión final la ejecuta la persona o el propio agente en su nombre a través de `analyse <item> skip --reason ...`, nunca de forma silenciosa.

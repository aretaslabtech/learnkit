# Quickstart: validar el checklist por fase y `analyse`/`consolidate`

**Feature**: [spec.md](./spec.md) | **Contracts**: [contracts/cli-commands.md](./contracts/cli-commands.md), [contracts/agent-skill.md](./contracts/agent-skill.md)

Valida las 4 historias de usuario en orden. Cada bloque es un incremento comprobable por sí solo (Independent Test de cada historia en `spec.md`).

## Prerrequisitos

- Proyecto LearnKit inicializado (feature 001) con una sesión ya inventariada (feature 002, US1): notas + transcripción de una clase de ejemplo. Puede reutilizarse el fixture `tests/fixtures/eoi-sample/` ya existente.
- Un agente (Claude Code u otro) con la Skill `learnkit-analyse` instalada (`learnkit init` la instala igual que `learnkit-language`).
- Ningún requisito de red ni de binario externo nuevo (esta feature no llama a ningún provider externo, ver `plan.md` → Technical Context).

## Escenario 1 — Checklist visible por fase (User Story 1)

```bash
learnkit status --session <session_id> --json
# Esperado (sesión recién inventariada, analyse todavía no iniciada):
# la fase "analyse" no lleva "checklist" todavía — el array se va poblando
# elemento a elemento a medida que cada comando de `analyse` lo confirma o
# lo resuelve (no hay un "summary"/"mindmap" placeholder en "pending" antes
# de tocarlo).

# Confirmar solo el resumen (ver Escenario 2 para el detalle del comando):
learnkit analyse summary set --session <session_id> --file resumen.md

learnkit status --session <session_id> --json
# Esperado: checklist muestra solo "summary": "done" — "mindmap" y las
# páginas aún no aparecen porque todavía no se ha actuado sobre ellas, no
# un único estado agregado para toda la fase "analyse".
```

## Escenario 2 — Resumen, mapa mental y páginas (User Story 2)

En la conversación con el agente (Skill `learnkit-analyse`):

```text
Tú: "Analiza la sesión <session_id>."
Agente: lee `learnkit session show --session <session_id> --json`, redacta el
        resumen, el mapa mental y al menos una página de concepto, y confirma
        cada uno:

learnkit analyse summary set --session <session_id> --file resumen.md
learnkit analyse mindmap set --session <session_id> --file mapa.md
learnkit analyse page add --session <session_id> --concept "phrasal verbs" --file pagina-phrasal-verbs.md
```

```bash
learnkit status --session <session_id> --json
# Esperado: los 3 elementos "done", con su fingerprint de fuentes.

learnkit analyse summary set --session <session_id> --file resumen.md
# Repetir sin cambios en el material: Esperado: no-op (ya estaba "done"), sin
# regenerar nada (idempotencia, FR-009).
```

## Escenario 3 — Elemento pendiente de decisión del usuario (User Story 3)

```bash
# Con una sesión cuyo material no basta para un mapa mental útil.
# `flag-pending`/`skip` toman el item_id como argumento posicional — no como
# subcomando de `summary`/`mindmap`/`page`:
learnkit analyse flag-pending mindmap --session <session_id> --reason "el material no tiene contenido suficiente para un mapa mental útil"

learnkit status --session <session_id> --json
# Esperado: el elemento "mindmap" del checklist trae
# "state": "pending_user_decision", "pending_reason": "...", mientras
# "summary" sigue su curso normal en paralelo.

# Resolución A: aportar más material y confirmar normalmente
learnkit analyse mindmap set --session <session_id> --file mapa-completo.md
# Esperado: pasa a "done", el estado pendiente desaparece.

# Resolución B (sesión distinta): omitir explícitamente
learnkit analyse skip mindmap --session <session_id> --reason "esta clase no tiene contenido jerarquizable"
# Esperado: deja de bloquear "consolidate"; volver a consultar status no
# vuelve a preguntar por este elemento mientras el material no cambie.
```

## Escenario 4 — Consolidar vocabulario sin duplicar (User Story 4)

```bash
learnkit consolidate --session <session_id> --json
# Esperado: {"ok": true, "candidates": [{"text": "...", "already_exists": false, ...}, ...]}

# Repetir con una segunda sesión que comparte una palabra ya consolidada antes:
learnkit consolidate --session <otra_session_id> --json
# Esperado: el candidato repetido aparece con "already_exists": true y
# "existing_vocabulary_id" apuntando a la entrada ya persistida — no se
# duplica en el listado de candidatos nuevos.

learnkit consolidate list --session <session_id> --json
# Esperado: cada candidato conserva su origen (resumen/mapa/página concreta).
```

## Escenario de bloqueo — `consolidate` con un elemento pendiente sin resolver

```bash
learnkit analyse page add --session <session_id> --concept "conditionals" --file pagina-conditionals.md
learnkit analyse mindmap flag-pending --session <session_id> --reason "..."
learnkit consolidate --session <session_id> --json
# Esperado: exit code 20, "code": "BLOCKED", indicando qué elemento de
# analyse sigue pendiente — no se consolida con datos incompletos.
```

# 09 — Integración con agentes

## 1. Objetivo

Codex y Claude deben seguir el mismo workflow sin duplicar toda la metodología.

## 2. Source of truth

```text
.learnkit/
  workflow.toml
  profiles/
  schemas/
  rules/

skills/
  learnkit-session/
  learnkit-process/
  learnkit-cards/
  learnkit-assessment/
  learnkit-publish/
```

Los Skills contienen instrucciones ligeras y llaman al CLI para comprobar estado y ejecutar validación.

## 3. Regla de oro

El agente **no edita** manifests de validación ni estado interno para “desbloquear” fases.

El patrón es:

```text
agent writes artifact
  -> learnkit validate/run
  -> CLI says PASS/FAIL
  -> agent repairs artifact if FAIL
```

## 4. Codex

Adapter inicial:
- `AGENTS.md` breve con reglas globales;
- skills locales del repo bajo `.agents/skills/` o la ubicación vigente soportada por Codex;
- cada Skill con `SKILL.md`, referencias y scripts mínimos;
- comandos deterministas delegados a `learnkit`.

Ejemplo de regla en `AGENTS.md`:

```text
For any LearnKit session task, inspect `learnkit status --json` first.
Never bypass a failed guard or edit validation manifests manually.
Use the relevant LearnKit skill and finish by running `learnkit validate`.
```

## 5. Claude

Adapter inicial:
- `CLAUDE.md` corto;
- Agent Skills equivalentes cuando estén disponibles;
- hooks de Claude Code solo como cinturón adicional, no como única seguridad.

Los hooks pueden interceptar operaciones peligrosas, pero la seguridad del workflow permanece en el CLI.

## 6. Por qué ambos

Skills/prompts son **guidance**. Guards del CLI son **enforcement**.

```text
Skill: "debes validar antes de exportar"
Guard: export command refuses if validation fails
```

## 7. Skills V1 sugeridos

- `learnkit-session`: iniciar/ingestar/inventariar.
- `learnkit-analyse`: extraer learning items con trazabilidad.
- `learnkit-language`: enriquecer vocabulario cuando aplica.
- `learnkit-cards`: proponer Card Definitions y media intents.
- `learnkit-assessment`: generar banco de preguntas.
- `learnkit-repair`: interpretar fallos de validación y corregir.

No crear un skill por cada subcomando.

## 8. Salida estructurada

El CLI expone `--json`; los Skills deben preferirlo para decisiones.

Nunca parsear texto decorativo si existe contrato JSON.

## 9. Evals

Crear fixtures de tareas para probar que el agente:
- inventaría todos los ficheros;
- no salta fases;
- reacciona a `PHASE_BLOCKED`;
- no inventa PASS;
- conserva trazabilidad;
- produce cards válidas.

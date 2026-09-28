# 04 — Workflow y hard guards

## 1. Principio

Un agente puede producir artefactos. **No puede otorgarse a sí mismo un PASS**.

El CLI valida precondiciones y outputs de forma determinista antes de ejecutar una fase dependiente.

## 2. Pipeline V1 genérico

```text
session
  -> inventory
  -> analyse
  -> consolidate
  -> learn
  -> cards
  -> assessment
  -> validate
  -> export/publish
```

Los perfiles pueden expandir `learn`:

```text
language:
  learn -> vocabulary -> language_enrichment
```

## 3. DAG, no lista rígida

El workflow se define como dependencias.

Ejemplo:

```toml
[phases.inventory]
requires = []

[phases.analyse]
requires = ["inventory"]

[phases.cards]
requires = ["consolidate"]

[phases.anki]
requires = ["cards"]

[phases.kahoot]
requires = ["assessment"]
```

Esto permite ejecutar ramas independientes una vez satisfechas sus dependencias.

## 4. Estados

Estados observables:
- `not_started`;
- `dirty`;
- `valid`;
- `blocked`;
- `failed`.

No guardar `passed=true` como autoridad. El estado guardado es cache/diagnóstico.

### 4.1 Checklist por elemento (fases con checklist)

Una fase puede llevar, además de su estado agregado de la lista anterior, un checklist de elementos independientes (`PhaseManifest.checklist`, hoy usado por `analyse`: `summary`, `mindmap`, `page-<n>`). Cada elemento tiene su propio estado, `ChecklistItemState`, separado del estado agregado de la fase:
- `pending` — todavía no confirmado ni resuelto;
- `done` — confirmado con contenido real, o resuelto explícitamente vía `skip` (auditado con motivo);
- `blocked` — bloqueado por una dependencia no satisfecha;
- `pending_user_decision` — marcado por el agente (`flag-pending --reason "<motivo>"`) porque el material no basta para confirmarlo; visible en `status` con su motivo, y solo bloquea la parte de una fase posterior (p. ej. `consolidate`) que dependería de ese elemento concreto, no el resto del checklist.

El estado agregado de una fase con checklist se deriva de sus elementos, nunca al revés: `valid` solo si todos son `done`; `PhaseState::NeedsUserInput` si al menos uno sigue `pending_user_decision` y ninguno está `failed`; en otro caso la fase sigue en curso (`dirty`). Una fase sin checklist (`PhaseManifest.checklist = None`) conserva el modelo de estado único de la sección 4 sin cambios — ver `crates/learnkit-workflow/src/engine.rs` (`PhaseState`, `ChecklistItemState`) y `specs/003-analyse-consolidate-checklist/data-model.md`.

## 5. Guard de entrada

Antes de `learnkit run cards`:

1. cargar workflow;
2. resolver dependencias;
3. ejecutar validadores de dependencias;
4. comparar hashes de inputs relevantes;
5. si falla algo, no ejecutar.

Salida ejemplo:

```text
BLOCKED: cards
  ✓ inventory valid
  ✗ consolidate invalid: 2 source references unresolved
exit code: 20
```

## 6. Guard de salida

Tras generar artefactos:

1. validar schema;
2. validar referencias;
3. validar assets;
4. ejecutar reglas de perfil;
5. escribir manifest solo si todo pasa.

## 7. Manifests

Cada fase puede tener:

```json
{
  "phase": "cards",
  "validator_version": "cards-v1",
  "input_fingerprint": "sha256:...",
  "output_fingerprint": "sha256:...",
  "validated_at": "2026-09-26T12:00:00Z",
  "result": "valid"
}
```

El CLI nunca confía solo en `result`. Si los inputs/outputs han cambiado, invalida y vuelve a validar.

## 8. Invalidación descendente

Si cambia una fuente:

```text
inventory dirty
  -> analyse dirty
  -> consolidate dirty
  -> cards dirty
  -> assessment dirty
  -> exporters dirty
```

Si solo cambia una plantilla Anki:

```text
cards unchanged
anki export dirty
```

La invalidación debe basarse en dependencias declaradas y fingerprints.

## 9. Tipos de validación

### Determinista
- schema válido;
- fichero existe;
- hash coincide;
- referencias resuelven;
- IDs únicos;
- media decodificable;
- campos requeridos;
- límites del exporter.

### Semántica asistida por agente
Algunas reglas no pueden probarse puramente en Rust, por ejemplo “el ejemplo de inglés es natural”. Se modelan como checks separados que producen evidencia estructurada.

Regla: un check semántico jamás sustituye a un check determinista.

## 10. Override manual

V1 puede ofrecer:

```bash
learnkit validate --accept semantic-check-17 --reason "reviewed manually"
```

El override queda auditado. Nunca existe `--force-everything` para publicar ignorando silenciosamente fallos.

## 11. Publish guard

Una exportación concreta solo requiere su rama:

```text
Anki requires: cards + required media + anki validation
Kahoot requires: assessment + kahoot constraints
Blooket requires: assessment + blooket constraints
```

No obligar a tener Anki para exportar un Kahoot.

# 05 — CLI Rust

Nombre provisional del binario: `learnkit`.

## 1. Principios UX

- comandos explícitos;
- `--json` para agentes/automatización;
- salida humana legible por defecto;
- dry-run donde haya escritura externa;
- exit codes estables;
- errores accionables;
- ningún agente necesita editar archivos internos de estado a mano.

## 2. Comandos V1

```text
learnkit init
learnkit doctor
learnkit status

learnkit session new
learnkit session list
learnkit session show

learnkit ingest
learnkit inventory
learnkit transcribe
learnkit analyse
learnkit consolidate
learnkit learn
learnkit cards build
learnkit assessment build

learnkit validate
learnkit run <phase>
learnkit run --until <phase>

learnkit asset list
learnkit asset validate

learnkit export anki
learnkit export kahoot
learnkit export blooket
learnkit export exam

learnkit attempt import
learnkit progress

learnkit profile list
learnkit profile show

learnkit agent install codex
learnkit agent install claude
```

## 3. Inicialización

```bash
learnkit init . --profile generic --agents codex,claude
```

Debe crear solo lo necesario y ser idempotente.

## 4. Nueva sesión

```bash
learnkit session new "Océanos y continentes" --profile geography
```

Devuelve ID estable y path.

## 5. Ingesta

```bash
learnkit ingest ./material/* --session <id>
```

Por defecto copia o referencia según config. V1 debe preferir copia dentro de la sesión para reproducibilidad.

## 6. Transcripción

```bash
learnkit transcribe --session <id>
learnkit transcribe --session <id> --provider whisper-cpp
learnkit transcribe --session <id> --provider faster-whisper
learnkit transcribe import ./transcript.vtt --source <source-id>
```

Comportamiento:
- el provider por defecto se resuelve desde el perfil/configuración;
- V1 recomienda `whisper.cpp` como default local inicial;
- Buzz se soporta mediante importación de sus outputs cuando estén disponibles en un formato interoperable;
- importar una transcripción no debe volver a procesar el audio;
- el CLI conserva engine/model/config y fingerprint del source;
- cualquier cambio relevante invalida artefactos dependientes.

`learnkit transcribe benchmark` queda permitido como comando experimental para comparar providers sobre un corpus de referencia, pero no es requisito del flujo diario.

## 7. Ejecución

```bash
learnkit run inventory --session <id>
learnkit run --until validate --session <id>
```

`run --until` ejecuta dependencias en orden topológico y se detiene ante el primer guard fallido.

## 7.1 Análisis (`analyse`) y consolidación (`consolidate`)

`analyse` no es un paso "todo o nada" como las fases anteriores: es una fase con checklist (`docs/04-workflow-guards.md §4.1`) que se va completando elemento a elemento, normalmente invocada por el agente a través de la Skill `learnkit-analyse`. Cada subcomando confirma o resuelve un elemento (`summary`, `mindmap`, o `page-<n>`, uno por cada `analyse page add`):

```bash
learnkit analyse summary set --session <id> --file resumen.md [--filled-gap "<concepto>:<nota>"]... [--force]
learnkit analyse mindmap set --session <id> --file mapa.md [--force]
learnkit analyse page add --session <id> --concept "<texto>" --file pagina.md

learnkit analyse flag-pending <item_id> --session <id> --reason "<motivo>"
learnkit analyse skip <item_id> --session <id> --reason "<motivo>"
```

`<item_id>` en `flag-pending`/`skip` es un argumento posicional (`summary`, `mindmap`, o `page-<n>`), no un subcommand anidado bajo `summary`/`mindmap`/`page`. `summary set` y `mindmap set` son idempotentes por fingerprint de fuentes (no-op si el elemento ya está `done` con las mismas fuentes, salvo `--force`); `flag-pending` deja el elemento en `pending_user_decision` con un motivo visible en `status`, sin bloquear el resto del checklist; `skip` lo resuelve explícitamente sin contenido real, de forma auditada.

`consolidate` (`requires = [analyse]`) deriva candidatos de vocabulario del resumen/páginas ya confirmados, los deduplica contra el vocabulario ya persistido, y los persiste:

```bash
learnkit consolidate --session <id> [--json]
learnkit consolidate list --session <id> [--json]
```

Si algún elemento de `analyse` sigue `pending_user_decision`, `consolidate` falla con exit `20` y `"code": "BLOCKED"` (§14) en vez de consolidar con datos incompletos; `consolidate list` reexpone los candidatos ya persistidos, incluyendo `already_exists`/`existing_vocabulary_id` para los que ya tenían una entrada de vocabulario equivalente. Contrato completo: `specs/003-analyse-consolidate-checklist/contracts/cli-commands.md`.

## 8. Tarjetas

```bash
learnkit cards build --session <id>
learnkit cards validate --session <id>
```

Filtros futuros:

```bash
learnkit cards build --tag oceans
```

## 9. Anki

```bash
learnkit export anki --session <id> --out dist/geography.apkg
```

Debe fallar si una tarjeta que requiere media no la tiene.

## 10. Kahoot y Blooket

```bash
learnkit export kahoot --assessment <id> --out dist/kahoot.xlsx
learnkit export blooket --assessment <id> --out dist/blooket.csv
```

Los exporters aplican las limitaciones de cada target y nunca mutilan preguntas silenciosamente.

## 11. Examen local

```bash
learnkit export exam --assessment <id> --format html --out dist/exam.html
```

En V1 puede bastar HTML autocontenido con formulario y un JSON de resultados exportable.

## 12. Progreso

```bash
learnkit progress --session <id>
learnkit progress --tag oceans
learnkit progress --skill listening
```

## 13. JSON mode

Todo comando relevante debe soportar:

```bash
learnkit status --json
```

Formato estable para agentes:

```json
{
  "ok": false,
  "code": "PHASE_BLOCKED",
  "phase": "anki",
  "failures": [
    {"check":"required_asset","entity":"card-17","message":"back audio missing"}
  ]
}
```

## 14. Exit codes propuestos

- `0`: success;
- `2`: CLI usage/config;
- `10`: validation failed;
- `20`: guard blocked;
- `30`: provider/external tool failed;
- `40`: exporter constraint failed;
- `50`: filesystem/store failure.

## 15. `doctor`

Debe comprobar:
- proyecto válido;
- permisos;
- configuración;
- ffmpeg si se requiere;
- disponibilidad/modelos del provider de transcripción configurado;
- provider credentials sin mostrar secretos;
- compatibilidad de plantillas;
- índice reconstruible.

## 16. Comandos reservados para V2 — TAO/QTI

No se implementan en V1, pero se reservan para evitar rediseñar la UX:

```bash
learnkit export qti --assessment <id> --profile tao --out dist/exam-qti.zip
learnkit import tao-results ./results-file --assessment <id>
learnkit tao doctor
```

`learnkit export qti` debe validar el paquete antes de escribirlo. El `--profile tao` seleccionará las restricciones reales de la versión de TAO/QTI soportada, sin introducir esas restricciones en el Question Bank canónico.

V2 comenzará con un spike de interoperabilidad QTI 2.2 contra una instalación real de TAO Community Edition. QTI 3 no se declarará soportado hasta tener round-trip y tests contra una versión estable de TAO que lo soporte.

# Quickstart: validar el flujo end-to-end de inglés (clase EOI)

**Feature**: [spec.md](./spec.md) | **Contracts**: [contracts/cli-commands.md](./contracts/cli-commands.md)

Valida las 7 historias de usuario en orden. Cada bloque es un incremento comprobable por sí solo (Independent Test de cada historia en `spec.md`).

## Prerrequisitos

- Proyecto LearnKit inicializado con perfil de idiomas (feature 001): `learnkit init --profile language --agents claude`.
- Para el Escenario 2 (transcripción): el binario `whisper.cpp` instalado y accesible en `PATH` (o un fichero `.srt`/`.vtt` ya existente para probar solo la vía de importación).
- Para el Escenario 4 (audio generado): el binario `piper` instalado y accesible en `PATH`, con al menos una voz `en-GB` descargada.
- Para el Escenario 4 (imágenes): conectividad de red saliente hacia `commons.wikimedia.org` (único requisito de red de toda la feature).
- Material de ejemplo de una clase real: 1 fichero de notas Markdown, 1 audio corto (1-2 min basta para la demo), 1 imagen.

## Escenario 1 — Sesión y fuentes (User Story 1)

```bash
learnkit session new "EOI — Unit 5 phrasal verbs" --profile language-en
# Esperado: {"ok": true, "session_id": "...", ...}

learnkit ingest ./clase-unit5/* --session <session_id>
learnkit inventory --session <session_id>
# Esperado: cada fuente (notas, audio, imagen) inventariada con tipo/tamaño/sha256.

learnkit inventory --session <session_id>
# Esperado: mismo resultado (idempotente, ninguna fuente duplicada).

# Editar el fichero de notas y volver a inventariar:
learnkit inventory --session <session_id>
# Esperado: la fuente de notas cambia a stale y se re-inventaría con nueva huella.
```

## Escenario 2 — Transcripción (User Story 2)

```bash
learnkit transcribe --session <session_id>
# Esperado: {"ok": true, "transcript_id": "...", "engine": {"provider": "whisper-cpp", ...}, "segments": N}

# Vía alternativa (sin whisper.cpp instalado): importar una transcripción ya corregida
learnkit transcribe import ./clase-unit5-corregida.srt --source <audio_source_id>
# Esperado: mismo shape de resultado, "engine": {"imported_from": "..."} ; no se reprocesó el audio.

learnkit run cards --session <session_id>
# (sin vocabulario aún) Esperado: BLOCKED — no hay vocabulario del que generar tarjetas todavía.
```

## Escenario 3 — Vocabulario (User Story 3)

```bash
learnkit session show <session_id> --json
# Esperado: incluye el texto de las notas y los segmentos de la transcripción activa —
# esto es lo que la Skill de agente (research.md §2) lee para proponer candidatos.

# Tras la conversación con el agente, confirmar una expresión sugerida:
learnkit learn vocabulary add --session <session_id> \
  --lemma "get away with" --sense "hacer algo malo sin ser castigado" \
  --source <transcript_source_id> --locator "segment:00:12:34" --suggested-by agent
# Esperado: {"ok": true, "vocabulary_id": "vocab-en-get-away-with", "learning_item_id": "li-..."}

# Añadir una entrada manualmente, sin partir de ninguna sugerencia:
learnkit learn vocabulary add --session <session_id> \
  --lemma "whiteboard" --sense "pizarra" --source <notes_source_id> --suggested-by manual

# Repetir la misma expresión en otra sesión reutiliza la entrada existente:
learnkit learn vocabulary add --session <otra_session_id> --lemma "get away with" --sense "..." --source <...>
# Esperado: mismo vocabulary_id que antes, sin duplicado.
```

## Escenario 4 — Tarjetas con imagen y audio (User Story 4)

```bash
learnkit cards build --session <session_id>
# Esperado: por cada elemento de vocabulario sin recurso propio, se genera audio con Piper
# y se busca imagen en Wikimedia Commons; las que no tengan imagen con licencia válida
# quedan marcadas pending_image.

learnkit cards validate --session <session_id> --json
# Esperado: {"ok": true/false, "cards": [{"id": "card-001", "state": "complete"}, {"id": "card-005", "state": "pending_image", "reason": "..."}]}
```

## Escenario 5 — Exportación a Anki (User Story 5)

```bash
learnkit export anki --session <session_id> --out dist/eoi-unit5.apkg
# Si alguna tarjeta no está complete:
# Esperado: falla con exit code 40, listando exactamente qué tarjeta/recurso falta; dist/ no contiene un .apkg parcial.

# Tras completar todas las tarjetas pendientes:
learnkit export anki --session <session_id> --out dist/eoi-unit5.apkg
# Esperado: éxito; dist/eoi-unit5.apkg existe.
```

**Validación manual (no automatizable en CI)**: importar `dist/eoi-unit5.apkg` en Anki Desktop y comprobar que al menos una tarjeta muestra su imagen y reproduce audio distinto en frontal y reverso (`docs/07-anki-media.md §12`).

## Escenario 6 — Evaluación local (User Story 6)

```bash
learnkit assessment build --session <session_id>
# Esperado: {"ok": true, "assessment_id": "exam-...", "items": N, "skills": ["recognition","production","listening"]}

learnkit export exam --assessment <assessment_id> --format html --out dist/exam.html
# Abrir dist/exam.html en un navegador, completar el examen, pulsar "descargar resultados" -> results.json

learnkit attempt import ./results.json --assessment <assessment_id>
# Esperado: {"ok": true, "attempts_imported": N}
```

## Escenario 7 — Progreso (User Story 7)

```bash
learnkit progress --session <session_id> --json
# Esperado: métricas por learning_item_id y por skill (recognition/production/listening),
# incluyendo accuracy_all_time, accuracy_recent, y attempt_count.

learnkit progress --skill listening --json
# Esperado: la misma vista filtrada solo a la destreza listening, recalculada en el momento
# (nunca servida desde un valor guardado de una consulta anterior).
```

## Resultado esperado global

Al completar los 7 escenarios, todos los criterios de éxito de `spec.md` (SC-001 a SC-008) quedan demostrados observando el filesystem de la sesión, la salida `--json` del CLI, y (para Anki) una importación manual puntual.

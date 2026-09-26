# 10 — Storage, filesystem y Git

## 1. Fuente de verdad

Los artefactos canónicos son ficheros versionables. SQLite es derivado y reconstruible.

## 2. Layout de proyecto

```text
my-learning-project/
├── learnkit.toml
├── AGENTS.md
├── CLAUDE.md
├── .learnkit/
│   ├── workflow.toml
│   ├── profiles/
│   ├── schemas/
│   └── cache/
│       └── index.sqlite
├── knowledge/
│   ├── learning-items/
│   └── vocabulary/          # solo si perfil idioma
├── sessions/
│   └── 2026-09-26-oceans/
│       ├── session.yaml
│       ├── input/
│       ├── inventory/
│       ├── analysis/
│       ├── consolidated/
│       ├── cards/
│       ├── assessments/
│       ├── assets/
│       │   ├── images/
│       │   ├── audio/
│       │   └── video/
│       ├── validation/
│       └── dist/
├── attempts/
│   └── attempts.jsonl
└── publish/
```

## 3. Granularidad

Preferir un fichero por entidad cuando facilite diffs y merges:

```text
knowledge/learning-items/li-pacific-ocean.yaml
cards/card-001.yaml
```

Evitar una base monolítica JSON gigante.

## 4. SQLite derivado

`.learnkit/cache/index.sqlite` puede indexar:
- IDs -> paths;
- tags;
- source links;
- vocabulary lookup;
- card relationships;
- progress aggregations.

Comando:

```bash
learnkit index rebuild
```

Borrar SQLite no debe destruir conocimiento.

## 5. Escrituras atómicas

Patrón:
1. escribir temp;
2. fsync si aplica;
3. rename atómico;
4. recalcular manifest/index.

## 6. Git policy

Versionar:
- config;
- profiles;
- YAML/JSON/Markdown;
- templates;
- attempts (si el usuario desea conservar progreso);
- assets pequeños/medios cuando sean esenciales.

Ignorar:
- `.learnkit/cache/`;
- temporales;
- renders intermedios;
- credenciales;
- logs locales.

## 7. Assets grandes

Definir umbral configurable. Opciones:
- Git LFS;
- storage local no versionado + manifest hash;
- repositorio separado de media.

Para V1, no resolver automáticamente Git LFS. Documentar y avisar al superar el umbral.

## 8. Secrets

Nunca dentro de `learnkit.toml` versionado.

Usar:
- variables de entorno;
- keychain/provider credential store;
- fichero local ignorado, si no hay alternativa.

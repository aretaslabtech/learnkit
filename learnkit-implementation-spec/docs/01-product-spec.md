# 01 — Product specification

## 1. Problema

Los materiales de estudio suelen terminar fragmentados entre PDFs, notas, audios, chats, tarjetas Anki, quizzes y plataformas de juego. Las herramientas existentes suelen resolver una parte del proceso, pero convierten la plataforma en el centro del sistema.

LearnKit invierte esa relación: **el repositorio local es el centro** y las plataformas externas son destinos.

## 2. Objetivo

Crear un sistema local-first que permita a un agente de IA procesar una sesión de aprendizaje de forma reproducible y verificable, produciendo materiales de estudio y evaluación reutilizables.

## 3. Casos de uso prioritarios

### Idiomas
- Procesar notas, imágenes, PDFs y audio de una clase.
- Extraer vocabulario y expresiones.
- Mantener IPA, variedad lingüística, ejemplos y uso.
- Generar tarjetas con imagen y audio.
- Permitir audio distinto en frontal y reverso.
- Generar Anki `.apkg`.
- Generar práctica y evaluaciones.

### Geografía / Historia
- Extraer conceptos, lugares, fechas, relaciones y definiciones.
- Generar tarjetas con imágenes o mapas.
- Crear preguntas para juegos y exámenes.
- Medir progreso por concepto/habilidad.

### Aprendizaje técnico (Godot)
- Extraer conceptos, APIs, nodos y patrones.
- Crear preguntas, ejercicios y tarjetas.
- Añadir artefactos específicos del perfil (por ejemplo `code_example`).

## 4. Personas

V1 está optimizada para:
- un usuario individual técnico;
- estudio familiar (por ejemplo jugar con un hijo);
- pequeños grupos de compañeros;
- uso con Codex o Claude desde un repositorio local.

No se diseña todavía como LMS multiusuario.

## 5. Objetivos V1

- CLI Rust multiplataforma.
- Inicialización de proyecto.
- Creación de sesiones.
- Inventario de fuentes.
- Transcripción reproducible de audio mediante provider desacoplado e importación de transcripciones existentes.
- Learning Items genéricos.
- Perfil lingüístico opcional.
- Card Engine multimedia.
- Assets de imagen/audio independientes.
- Exportación Anki `.apkg`.
- Banco de preguntas.
- Examen local autocorregible.
- Exportación Kahoot XLSX.
- Exportación Blooket CSV.
- Registro de intentos/progreso.
- Hard guards y validadores.
- Integración Codex/Claude.

## 6. No-objetivos V1

- LMS multiusuario.
- Sincronización en nube propia.
- Editor gráfico completo.
- Marketplace de perfiles.
- Adaptive testing sofisticado basado en IRT.
- Vídeo generado automáticamente.
- Importación automática de resultados de todas las plataformas externas.
- Editor de transcripción propio; V1 importa/corrige artefactos y delega la inferencia a providers.
- QTI/TAO en producción (se reserva para V2).
- Integración TCExam.

Se pueden reservar extensiones para ellos sin implementarlos aún.

## 7. Definiciones visibles

Solo cuatro conceptos deben ser imprescindibles para un usuario normal:

- **Session**: unidad de trabajo/estudio.
- **Learning Item**: algo que merece ser aprendido.
- **Card**: representación de un learning item para memorización/práctica.
- **Asset**: imagen, audio o vídeo usado por materiales.

El perfil de idiomas añade:

- **Vocabulary Entry**: información lingüística enriquecida y persistente.

## 8. Métrica de éxito arquitectónica

La misma versión del core debe ejecutar un escenario de inglés y otro de geografía sin introducir condiciones como `if subject == English` dentro del core.

## 8. Dirección V2 acordada

V2 añade evaluación formal self-hosted mediante **TAO Community Edition**. El objetivo no es convertir LearnKit en un LMS ni duplicar el motor de examen de TAO, sino usar TAO como runtime de entrega y scoring mientras LearnKit conserva:

- Question Bank canónico;
- Learning Items y trazabilidad;
- Assessment definitions;
- progreso longitudinal;
- exporters/importers interoperables.

La frontera prevista es:

```text
LearnKit Question Bank
        |
        v
   QTI exporter
        |
        v
      TAO CE
        |
        v
  examen formal
        |
        v
 results importer
        |
        v
LearnKit Attempts/Progress
```

El modelo V1 debe ser compatible conceptualmente con QTI (identidad estable, prompts, choices, correct responses, scoring, feedback, media y metadata), pero **no implementará el exporter QTI todavía**.

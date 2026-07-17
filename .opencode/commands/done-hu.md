# Comando: `/done-hu` — playbook duro

Cierra una HU con **Definition of Done + quality score**. No existe “casi listo”.

## Uso

```
/done-hu
/done-hu HU-001
```

## Precondiciones

- `machine.phase` ∈ {`docs`, `mutation`, `review`, `done_check`} o artifacts de impl completos.  
- Preferible: docs ya hechas (`DOCS_DONE`).  
- Pack de contexto: **boot + close** (+ lang si hace falta verificar tests).

## Playbook (pasos obligatorios)

### 0. PHASE

```
PHASE_TRANSITION
from: <actual>
to: done_check
trigger: /done-hu
gate_cleared: null
```

Set `machine.phase: done_check`, `gate: dod`.

### 1. Cargar

- `rules/definition-of-done.md`  
- `rules/modes.md`  
- `state/machine.yaml`  
- Artefactos de `machine.artifacts`  
- Diff de la HU (files tocados)

### 2. Checklist DoD (marcar con evidencia)

Para cada ítem de DoD: `OK` | `FAIL` | `N/A (motivo)`.

Mínimo a verificar con **comando real** cuando el stack lo permita:

| Check | Evidencia |
|-------|-----------|
| Tests verdes | salida de `cargo test` / equiv. |
| Mutation ≥ threshold | report path o N/A documentado |
| Review pass | `REVIEW_RESULT` / sin blockers |
| Public API | `/public-api` si hubo exports nuevos |
| Docs | paths listados |
| Machine coherente | artifacts non-null según tipo |

### 3. Quality score

1. Crear `spec/reviews/<hu>.md` desde `templates/quality-score.md`.  
2. Asignar 5 ejes 0–100 con justificación de una línea.  
3. `overall = round(avg)`.  
4. Emitir bloque `QUALITY_SCORE`.

### 4. Reglas de pass

| Mode | Condición para cerrar |
|------|------------------------|
| `standard` | DoD sin FAIL crítico; score se registra (warn si overall &lt; 85) |
| `strict` | DoD limpio **y** overall ≥ `quality_score_min_strict` (85) |
| `hotfix` | DoD reducido (tests regresión + review); debt[] si omitió mutation |
| `spike` / `explore` | **no** usar `/done-hu` |

FAIL crítico = tests rojos, review blockers, mutation bajo threshold sin equivalents, capas rotas (architect-guard fail).

### 5a. Si FAIL

```
DONE_HU_RESULT
status: blocked
failed_checks: […]
return_phase: mutation|green|docs|review
```

- Set `machine.phase` al return_phase.  
- `gate: dod` cleared? no — o null y rework.  
- **No** marcar HU completada.

### 5b. Si PASS

1. Manteca: commit de cierre si hay cambios pendientes (mensaje con HU).  
2. `machine`:
   - `phase: idle`
   - `gate: null`
   - `active_hu: null`
   - clear artifacts / specialists
   - `last_transition` actualizado  
3. `project-state.md`: HU → completada, log.  
4. Emitir:

```
DONE_HU_RESULT
status: closed
hu: HU-001
quality_overall: 87
commit: <hash|pending>
next: /backlog | /start-hu
```

## Anti-patrones

- F17 DoD de word  
- F11 Mutation theater  
- Cerrar en mode spike  

## Duración esperada

Un solo turno de orquestador bien hecho > tres “¿cerramos?” en el chat.

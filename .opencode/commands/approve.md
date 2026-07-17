# Comando: `/approve`

Aprueba el **gate** actual según la state machine y avanza la phase.

## Uso

```
/approve
/approve "comentario"
```

## Precondiciones

1. Leer `state/machine.yaml`.  
2. Debe existir `gate != null` (salvo soft-confirm documentado).  
3. Pack: boot + pack de la phase actual.

## Mapa gate → transición

| gate actual | phase actual | nueva phase | nuevo gate |
|-------------|--------------|-------------|------------|
| `hard_spec` | `hard_spec` | `test_plan` | `test_plan` (cuando el plan esté listo se setea; tras approve de spec: phase test_plan, gate null hasta TEST_PLAN_READY) |
| `test_plan` | `test_plan` | `red` | null |
| `review` | `review` | `mutation` | null |
| `dod` | `done_check` | (usar `/done-hu`, no approve suelto) | — |

### Detalle hard_spec

Tras `/approve` con `gate: hard_spec`:

- `phase: test_plan`  
- `gate: null` hasta que QArgento emita `TEST_PLAN_READY` (ahí `gate: test_plan`)  
- `rework: false`  
- Emitir `PHASE_TRANSITION`

### Detalle test_plan

- `phase: red`  
- QArgento escribe tests; luego `TESTS_RED` → puede pasar a `green` sin approve extra.

### Detalle review

Solo si `REVIEW_RESULT.status: pass` o el Director aprueba con findings menores documentados.  
Blockers → no approve; fix loop `review → green`.

## Pasos

1. Validar transición en `rules/state-machine.md`.  
2. Si ilegal → rechazar con mensaje (anti-patrón F05/F09).  
3. Patch `machine.yaml` + `project-state.md`.  
4. Invocar siguiente agent (pack correcto).  
5. Emitir:

```
PHASE_TRANSITION
from: hard_spec
to: test_plan
trigger: /approve
gate_cleared: hard_spec
machine: .opencode/state/machine.yaml
```

## Reglas

- No approve automático.  
- No approve sin artefacto legible (`/review` si dudás).  
- Mode `explore`: approve de feature deshabilitado.

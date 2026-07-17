# State machine de la fábrica

Fuente de verdad ejecutable: `.opencode/state/machine.yaml`  
Vista humana: `.opencode/state/project-state.md` (se sincroniza, no manda).

## Phases

| phase | Significado | Gate típico |
|-------|-------------|-------------|
| `idle` | Sin HU activa | — |
| `story` | PO escribiendo story | soft |
| `contracts` | Arqui contratos/ADR | soft |
| `hard_spec` | SpecWriter hard spec | **`hard_spec`** |
| `test_plan` | QArgento plan | **`test_plan`** |
| `red` | Tests escritos, fallan | — |
| `green` | DevSenior implementando | — |
| `review` | Revisor / guard | **`review`** si fail |
| `mutation` | Mutation / quality tests | — |
| `docs` | Manteca docs/knowledge | — |
| `done_check` | `/done-hu` en curso | **`dod`** |
| `spike` | Modo spike | — |
| `hotfix` | Path corto hotfix | **`review`** opcional |

## Transiciones legales (standard / strict)

```
idle → story                    # /start-hu
story → contracts               # story listo (soft ok)
contracts → hard_spec
hard_spec → test_plan           # SOLO con /approve gate=hard_spec
hard_spec → hard_spec           # /reject → rework
test_plan → red                 # SOLO con /approve gate=test_plan
test_plan → test_plan           # /reject
red → green                     # tests existen y fallan (confirmado)
green → review                  # impl claims done
review → green                  # request-changes
review → mutation               # approve / pass
mutation → docs                 # threshold ok o equivalents
mutation → red|green            # tests débiles → fix loop
docs → done_check               # /done-hu
done_check → idle               # DoD pass + score
done_check → docs|mutation|…    # DoD fail → fase con deuda
```

### Spike mode

```
idle → spike → idle             # fin time-box; puede emitir /adr
```

Prohibido: `spike → green` sin pasar por `/start-hu`.

### Hotfix mode

```
idle → hotfix → red → green → review → docs → idle
```

Obligatorio: test de regresión en rojo antes de green.  
Deuda: anotar `machine.debt[]` si se salteó mutation (strict **no** permite saltear).

## Gates

| gate | Se limpia con | Avanza a |
|------|---------------|----------|
| `hard_spec` | `/approve` | `test_plan` |
| `test_plan` | `/approve` | `red` |
| `review` | `/approve` o pass sin hallazgos | `mutation` |
| `dod` | DoD + score OK | `idle` |

`/reject` con gate activo: **no** cambia a la siguiente phase; queda en la misma y `gate` se mantiene o `rework: true`.

## Campos obligatorios en `machine.yaml`

Ver el propio archivo. Mínimo: `phase`, `mode`, `gate`, `active_hu`, `stack`, `mutation_threshold`.

## Sincronización

Al cambiar machine:

1. Actualizar `machine.yaml`.  
2. Reflejar en `project-state.md` (fase, gate, HU).  
3. Opcional engram: puntero a phase transition.

## Violaciones

Si el usuario o un agent pide saltear (ej. codear en `hard_spec` sin approve):

1. Frase de freno.  
2. Citar esta rule + phase actual.  
3. Ofrecer el comando legal (`/approve`, `/spike`, `/mode hotfix`, …).

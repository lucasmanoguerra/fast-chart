# Comando: `/reject`

Rechaza el artefacto del gate actual; **no** avanza la phase.

## Uso

```
/reject "faltan escenarios de NaN en precios"
```

## Pasos

1. Leer `machine.gate` y `machine.phase`.  
2. Si no hay gate: avisar; opcional soft-reject de artefacto en fase soft.  
3. Set `rework: true`; **phase se mantiene**.  
4. Registrar motivo en `project-state.md` log.  
5. Devolver control al agent dueño:

| gate | Agent |
|------|-------|
| hard_spec | specwriter |
| test_plan | qargento |
| review | devsenior (fix) luego revisor |
| dod | según failed_checks de `/done-hu` |

6. Emitir:

```
REJECT
gate: hard_spec
phase: hard_spec
reason: …
next_agent: specwriter
```

## Reglas

Motivo obligatorio (si falta, pedir). No borrar artefactos; se editan in-place.

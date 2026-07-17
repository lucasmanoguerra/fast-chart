# Comando: `/mode`

Cambia el modo de operación de la fábrica.

## Uso

```
/mode
/mode standard
/mode strict
/mode spike
/mode hotfix
/mode explore
```

Sin args: muestra modo actual + restricciones (`rules/modes.md`).

## Pasos

1. Leer `machine.mode` y `machine.phase`.  
2. Validar cambio:
   - No pasar a `spike` en mitad de `green` sin aparcar la HU (guardar debt o terminar/cancelar).  
   - Preferir: HU idle antes de cambiar a explore/spike.  
3. Escribir `machine.mode`.  
4. Sync `project-state.md`.  
5. Emitir:

```
MODE_CHANGED
from: standard
to: strict
reason: …
constraints: …
```

## Efectos colaterales

| to | Efecto |
|----|--------|
| strict | `/done-hu` exige score ≥ 85; guard más agresivo |
| spike | habilita `/spike`; desaconseja `/start-hu` hasta volver |
| hotfix | path corto; debt tracking |
| explore | read-only default |
| standard | default de delivery |

Ver `rules/modes.md`.

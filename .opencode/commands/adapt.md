# Comando: `/adapt`

Adapta un repo **existente** al proceso (specs retro, SRP, tests, tools) sin mentir que es greenfield.

## Uso

```
/adapt
/adapt --specs-only | --tests-only | --refactor-only | --tools-only | --full
```

## Pasos

1. **Auditoría** Arqui + QArgento + Manteca (estructura, tests, docs).  
2. **Plan** priorizado (no codear aún).  
3. **Approve** del plan.  
4. Ejecutar por slices (pueden ser HUs `/start-hu` generadas).  
5. Actualizar knowledge + state.

## Reglas

- No reescribir todo de una.  
- Specs retroactivos marcan `status: retroactive`.  
- No tocar `.opencode/` salvo gaps de fábrica pedidos por el Director.

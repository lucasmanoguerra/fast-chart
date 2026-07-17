# Subagentes

Un **subagent** es una tarea acotada con prompt autocontenido. No reemplaza al core pipeline.

## Cuándo usar `/subagent`

| Tarea | Read-only | Output |
|-------|-----------|--------|
| Explorar símbolos / blast radius | sí | resumen + paths |
| Draft de ADR | no (un archivo) | `spec/architecture/decisions/…` |
| Borrador de plan de tests | no | `spec/test-plans/…` draft |
| Correr tests/lint y resumir | sí | log estructurado |
| Comparar dos diseños | sí | tabla pros/contras |
| Buscar usos de API pública | sí | lista |

## Cuándo NO

- Implementar la feature completa  
- Aprobar specs  
- Merge / publish  
- “Hacé todo el flujo de la HU”  

## Contrato del prompt al subagent

```
GOAL: …
CONSTRAINTS: rules to respect (SRP, no .opencode edits, …)
INPUTS: paths + excerpts
OUTPUT: format (markdown file path OR bullet summary)
DONE WHEN: …
```

## Aislamiento

- Preferir worktree o branch si va a escribir código experimental (spikes).  
- Default **read-only**.  
- Writes solo a paths listados.

## Relación con multi-agent

`/multi-agent` coordina **roles** del equipo.  
`/subagent` lanza **workers** tácticos.  
El Director sigue siendo el único que mueve gates `/approve`.

# Comando: `/status`

Estado humano + machine.

## Uso

```
/status
/status hu-001
/status --full
```

## Pack

**Solo boot** (no cargar design/impl completo).

## Fuente

1. `state/machine.yaml` (manda)  
2. `state/project-state.md`  
3. Si drift → avisar (F12)

## Salida mínima

```
STATUS
factory: 2.1.0
mode: standard
phase: idle
gate: null
active_hu: null
stack: rust
mutation_threshold: 0.90
debt: []
next: /start-hu | /backlog | …
```

Más: tabla de HUs del project-state si `--full` o hu id.

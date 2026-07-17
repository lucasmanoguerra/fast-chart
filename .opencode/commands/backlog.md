# Comando: `/backlog`

Gestiona el backlog de HUs del proyecto (no de la fábrica).

## Uso

```
/backlog
/backlog add "…"
/backlog prioritize
/backlog show
```

## Pasos

1. PO lee state + `spec/stories/`.  
2. Lista ordenada por valor/riesgo/deps.  
3. Sugiere siguiente `/start-hu`.  
4. Opcional: sync a GitHub issues vía `gh` (con confirmación).

## Salida

Tabla ID | título | prioridad | deps | estado.

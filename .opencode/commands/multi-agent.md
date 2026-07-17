# Comando: `/multi-agent`

Orquesta **varios roles** de forma controlada (council / reviews paralelas).

## Uso

```
/multi-agent review diff
/multi-agent design "API de Series" --roles arqui,ux-api,numeric
```

## Protocolo

1. Director define **goal** y roles (≤ 4).  
2. Cada rol produce output en su formato (sin editarse entre sí).  
3. Director **mergea** en decisión única.  
4. Si hay que implementar → vuelve al pipeline normal (spec/tests/code).

Ver `agents/orchestration.md`.

## Anti-patrones

- Multi-agent para “codear más rápido la misma feature” en paralelo sin spec.  
- Más de un writer de código de prod a la vez.

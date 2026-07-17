# Estado del Proyecto

> **Última actualización**: 2026-07-16  
> **Proyecto**: candela  
> **Stack**: rust · **Tipo**: library  
> **Fábrica**: v2.1.0  
> **Machine**: ver `state/machine.yaml` (fuente de phase/gate)

---

## Resumen (sync machine)

| Indicador | Valor |
|-----------|-------|
| mode | standard |
| phase | idle |
| gate | null |
| active_hu | — |
| HU completadas | 0 |
| Mutation threshold | 90% |
| Quality min (strict) | 85 |
| Remoto GitHub | pendiente (`gh` auth) |
| Último commit producto | 659afbb |

---

## Historias activas

_Ninguna._

---

## Backlog sugerido

| ID | Título | Prioridad |
|----|--------|-----------|
| HU-001 | Domain OHLCV + Series | Alta |
| HU-002 | Viewport + escalas | Alta |
| HU-003 | Surface wgpu + resize | Alta |
| HU-004 | Candles instanced | Alta |
| HU-005 | Pan / zoom | Alta |

---

## ADRs producto

| ID | Título | Estado |
|----|--------|--------|
| ADR-001 | Library + winit/wgpu | Aprobado |

---

## Log

| Fecha | Evento | Detalle |
|-------|--------|---------|
| 2026-07-16 | factory 2.0 | rewrite genérico |
| 2026-07-16 | factory 2.1 | Pack A: packs, machine, contracts, modes, score |

---

## Notas

> Fábrica v2.1 lista. Delivery de lib: `/start-hu` cuando quieras.  
> No editar `.opencode` en features.

# Matriz de invocación de agentes

**Regla:** no invocar specialists “por las dudas”. Máx. 3 specialists / HU.

## Pipeline core (toda HU)

| Fase | Agent | Gate |
|------|-------|------|
| 1 Story | `po` | soft (Director puede ajustar) |
| 2 Contratos | `arqui` | soft |
| 3 Hard Spec | `specwriter` | **`/approve`** |
| 4 Plan + tests | `qargento` | **`/approve` plan** |
| 5 Code | `devsenior` | — |
| 6 Review | `revisor` | **`/approve` si hallazgos** |
| 6b Guard | `architect-guard` | si toca capas/API |
| 7 Mutation | `qargento` | threshold |
| 8 Docs | `manteca` | — |
| 9 Close | orquestador `/done-hu` | DoD |

## Specialists — cuándo sí

| Agent | Invocar si… |
|-------|-------------|
| `ux-api` | Diseño de API pública / ergonomía de consumidores |
| `security` | Input no confiable, auth, crypto, FFI, parsers |
| `perf` / `benchmark` | Hot path, allocs, latency, throughput |
| `concurrency` | async, threads, locks, race risk |
| `test-architect` | Property, fuzz, estrategia compleja |
| `graphics` | GPU, shaders, pipelines, wgpu/Vulkan |
| `math3d` / `numeric` | transforms, precision, money/rates |
| `data` | schemas, migraciones, queries |
| `cicd` / `release` | pipelines, publish, versionado |
| `packaging` | instaladores desktop (apps) |
| `sys-integration` | OS hooks, tray, notifications (apps) |
| `finalgo` / `finapi` | dominio financiero / market connectivity |
| `compliance` | GDPR/PCI/SOX u otras |
| `tech-writer` / `devrel` | guías largas, comunidad |
| `tech-writer` | docs de usuario extensas |

## Explicitamente fuera del default

- **GUI toolkits (egui/iced/etc.):** no hay agente `gui` en la fábrica. Si un **proyecto app** lo necesita, se agrega un specialist **en el repo** (`knowledge/` + profile local), no se asume en la fábrica genérica.
- No cargar financial/graphics en un CRUD web sin motivo.

## Detección de stack

Al `/init` o `/doctor`, registrar en state: `rust | ts | go | zig | python | mixed`.  
Cargar solo `rules/languages/<stack>.md` correspondientes.

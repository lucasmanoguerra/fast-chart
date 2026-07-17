# Testing — TDD, pirámide, mutation

## Orden TDD

1. Spec aprobado  
2. Plan de tests aprobado  
3. Escribir tests → **rojo**  
4. Implementación mínima → **verde**  
5. Refactor con tests verdes  
6. Mutation / property / fuzz según riesgo  

## Pirámide

| Nivel | Qué cubre | Notas |
|-------|-----------|--------|
| Unit | Contratos, pure domain, layout, parsers | Rápidos, sin I/O real |
| Integration | Adapters + dominio, FS/DB en testcontainers o fakes | |
| Acceptance | Escenarios Gherkin del Hard Spec | |
| Property / fuzz | Parsers, codecs, invariantes numéricas | Donde aporte |
| Golden / snapshot | Serialización, render headless si aplica | Estables y revisados |
| Benchmark | Regresiones de perf | No sustituyen correctness |

## Paths sugeridos (adaptar al stack)

```
tests/
  acceptance/   # o integration/ si el repo ya usa ese nombre
  unit/         # si no viven al lado del código
benches/
fuzz/
```

En Rust es válido unit-test en el mismo módulo `#[cfg(test)]` + `tests/` para integración.

## Mutation

- Default threshold: **90%** killed (ajustable en state del proyecto).
- Sobrevivientes → mejorar tests o documentar equivalentes en `mutation/equivalents.md`.
- Otros lenguajes: Stryker (JS/TS), mutmut, go-mutesting, etc.

## Reglas

- Tests solo del **spec** (no inventar features).
- Nombres legibles: comportamiento, no implementation detail.
- Deterministas: sin sleeps flaky; time inyectable.
- Sin red real en unit; fakes/mocks detrás de interfaces.
- Librerías: preferir tests **sin display/GPU** para CI base; GPU/manual aparte.

## QArgento vs DevSenior

- QA escribe/posee la suite de aceptación y el plan.
- Dev no “arregla” tests para hacer pasar basura: si el spec es malo, se corrige el spec.

# Sistema de especificaciones

## Artefactos y orden

| Orden | Artefacto | Path típico | Template |
|------:|-----------|-------------|----------|
| 1 | Story (HU) | `spec/stories/hu-###.md` | `story.md` |
| 2 | Contratos | `spec/contracts/<modulo>.*` | `contract-trait.md` / `contract-interface.md` |
| 3 | ADR (si aplica) | `spec/architecture/decisions/` | `adr.md` |
| 4 | Hard Spec | `spec/features/<id>.md` | ver tabla abajo |
| 5 | Plan de tests | `spec/test-plans/<id>.md` | `test-plan.md` |
| 6 | Mutation report | `mutation/report_<id>.md` | `mutation-report.md` |
| 7 | Spike / RFC | `spec/spikes/` o `spec/rfcs/` | `spike.md` / `rfc.md` |

## Qué template de Hard Spec usar

| Tipo de trabajo | Template |
|-----------------|----------|
| Feature de librería / core domain | `hard-spec-library.md` |
| Feature de aplicación / servicio | `hard-spec-feature.md` |
| API HTTP/REST/GraphQL | `hard-spec-api-http.md` |
| CLI | `hard-spec-cli.md` |
| Adapter (DB, broker, GPU, FS, OS) | `hard-spec-adapter.md` |
| Cambio de public API / semver | `hard-spec-library.md` + `/public-api` |

Si no está claro: **library** si el consumidor es código; **feature** si es usuario final de app.

## Hard Spec — mínimo obligatorio

1. Resumen ejecutivo (propósito, alcance, fuera de alcance)  
2. Criterios Gherkin (feliz, error, borde)  
3. Contrato de API del lenguaje (firmas / tipos / errores)  
4. Dependencias (módulos + traits/interfaces)  
5. Efectos secundarios (I/O, logs, eventos, estado)  
6. Invariantes y restricciones (perf, seguridad, threading)  
7. Estrategia de prueba (qué es unit / integración / headless)  

Sin Gherkin no se aprueba.

## Reglas de calidad del spec

- Sin ambigüedades (“debe ser rápido” → métrica o criterio observable).
- Sin imponer implementación concreta salvo constraint real (ej. “compatible con protocolo X”).
- Una feature por spec; si es monstruo, partir la HU.
- Trazabilidad: story id ↔ feature id ↔ contratos.

## Gates

- Hard Spec y plan de tests requieren `/approve` explícito.
- Cambio de spec ya implementado → re-approve + ajustar tests.

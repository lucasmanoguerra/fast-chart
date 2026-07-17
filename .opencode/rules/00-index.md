# Índice de Rules (fábrica v2.1)

## Carga (importante)

**No** cargues este catálogo entero. Usá **context packs**: `agents/context-packs.md`.

| Capa | Archivos | Cuándo |
|------|----------|--------|
| **Boot** (siempre) | `team-rules`, `modes`, `anti-patterns-factory`, `state-machine`, `output-contracts`, `multi-agent`, este index | todo turno |
| design | `architecture`, `specs`, `libraries` | story…hard_spec |
| qa | `testing`, `definition-of-done`, `specs` | test_plan…mutation |
| impl | `clean-code`, `architecture`, **un** `languages/*` | green |
| review | `architecture`, `clean-code`, `libraries` | review |
| close | `definition-of-done`, `knowledge` | docs / done-hu |

## Catálogo

| Archivo | Tema |
|---------|------|
| `team-rules.md` | SDD+TDD+SRP, commits, tono |
| `architecture.md` | Clean / hexagonal / DDD / ADR |
| `clean-code.md` | SRP archivo, naming, errores |
| `specs.md` | Sistema de specs + templates |
| `testing.md` | TDD, pirámide, mutation |
| `libraries.md` | Libs de larga vida, semver |
| `definition-of-done.md` | Checklist cierre |
| `knowledge.md` | Knowledge del **proyecto** |
| `multi-agent.md` | Orquestación / ruido |
| `state-machine.md` | Phases, gates, transiciones |
| `output-contracts.md` | Handoffs parseables |
| `modes.md` | standard/strict/spike/hotfix/explore |
| `anti-patterns-factory.md` | F01–F18 |
| `languages/*` | Stack puntual |

## Meta

1. Producto ≠ fábrica.  
2. Gana la rule sobre el profile si hay choque.  
3. Un concepto, un archivo.

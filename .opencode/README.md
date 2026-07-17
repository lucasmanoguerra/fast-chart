# OpenCode Factory v2.1 — equipo virtual + Spec-First

Configuración **genérica** de proceso para proyectos y librerías de larga vida (Rust y otros lenguajes).

> **Importante:** esta carpeta es la *fábrica*.  
> **No se modifica** al implementar features del producto.  
> Conocimiento del proyecto → `spec/`, `knowledge/`, `docs/`.  
> Solo se edita `.opencode/` cuando el Director pide **mejorar la fábrica**.

## Version

Ver `VERSION` (actual **2.1.0**) y `CHANGELOG.md`.

## Arranque rápido (cualquier agente)

1. `system-prompt.md`  
2. `state/machine.yaml` + `state/project-state.md`  
3. Context pack según phase → `agents/context-packs.md`  
4. Command o profile del turno  
5. Cerrar con **output contract** → `rules/output-contracts.md`

**No** cargues todos los `rules/languages/*` ni todos los profiles.

## Layout

| Path | Rol |
|------|-----|
| `system-prompt.md` | Orquestador |
| `opencode.yaml` | Índice, boot_rules, tools, noise |
| `VERSION` / `CHANGELOG.md` | Versión de fábrica |
| `rules/` | Estándares (cargar por packs) |
| `templates/` | Specs, score, ADR, … |
| `profiles/` | Agents |
| `agents/` | matrix, packs, multi/sub |
| `commands/` | Playbooks |
| `state/machine.yaml` | **Fase/gate (manda)** |
| `state/project-state.md` | Vista humana |

## Piezas v2.1 (Pack A)

| Pieza | Path |
|-------|------|
| Context packs | `agents/context-packs.md` |
| State machine | `state/machine.yaml` + `rules/state-machine.md` |
| Output contracts | `rules/output-contracts.md` |
| Modes | `rules/modes.md` + `/mode` |
| Anti-patrones | `rules/anti-patterns-factory.md` |
| DoD + score | `/done-hu` + `templates/quality-score.md` |

## Flujo HU

`story → contracts → hard_spec → test_plan → red → green → review → mutation → docs → done_check → idle`

Gates: `/approve` · `/reject` · cierre: `/done-hu`

## Principios

Spec First · TDD · SRP · Clean/hexagonal · API chica · Mutation · Knowledge vivo · DoD medible

## Idioma

Proceso: español rioplatense. Código/API: convención del lenguaje (inglés en Rust/TS/Go/Zig).

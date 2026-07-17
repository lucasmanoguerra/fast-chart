# Context packs — carga selectiva

Objetivo: **nunca** cargar todas las rules ni todos los lenguajes.  
El orquestador arma el contexto del turno con **boot + pack de fase + lang del stack**.

## Packs

### `boot` (siempre)

| Recurso | Path |
|---------|------|
| System | `system-prompt.md` |
| Index | `rules/00-index.md` |
| Team | `rules/team-rules.md` |
| Modes | `rules/modes.md` |
| Anti-patterns | `rules/anti-patterns-factory.md` |
| State machine rules | `rules/state-machine.md` |
| Output contracts | `rules/output-contracts.md` |
| Human state | `state/project-state.md` |
| Machine state | `state/machine.yaml` |
| Matrix | `agents/matrix.md` |
| Version | `VERSION` |

### `design` (story → contracts → hard spec)

- `rules/architecture.md`
- `rules/specs.md`
- `rules/libraries.md` (si `project_type` es library o mixed)
- `rules/clean-code.md` (ligero; full en impl/review)
- Templates: `story.md`, `contract-*`, hard-spec del `hu_type`, `adr.md` si aplica
- Profile: `po` | `arqui` | `specwriter` (uno por fase)
- Artefactos de la HU activa (paths en machine)

### `qa` (plan tests → rojo → mutation)

- `rules/testing.md`
- `rules/definition-of-done.md` (mutation threshold)
- `rules/specs.md` (trazabilidad Gherkin)
- Templates: `test-plan.md`, `mutation-report.md`
- Profile: `qargento` (+ `test-architect` solo si machine.specialists lo lista)
- Hard Spec aprobado + contratos

### `impl` (código verde)

- `rules/clean-code.md`
- `rules/architecture.md`
- **solo** `rules/languages/<stack>.md` (ver resolución de stack)
- Profile: `devsenior`
- Spec + test-plan + tests rojos + contratos

### `review` (diff / quality)

- `rules/architecture.md`
- `rules/clean-code.md`
- `rules/libraries.md` si lib
- Profiles: `revisor`, opcional `architect-guard`
- Diff / archivos tocados (blast radius; codegraph si hay)
- `agents/orchestration.md` si council

### `close` (docs → done-hu)

- `rules/definition-of-done.md`
- `rules/knowledge.md`
- Template: `quality-score.md`
- Profiles: `manteca`, orquestador en `/done-hu`
- Command playbook `commands/done-hu.md`

### `lang:<stack>`

Un solo archivo:

| Stack | Rule |
|-------|------|
| rust | `rules/languages/rust.md` |
| typescript / ts | `rules/languages/typescript.md` |
| javascript / js | `rules/languages/javascript.md` |
| react | `rules/languages/react.md` (+ ts o js si aplica) |
| go | `rules/languages/go.md` |
| zig | `rules/languages/zig.md` |
| python | `rules/languages/python.md` |
| mixed | langs listados en `machine.yaml → stack_langs` |

### `spike` / `hotfix` / `explore`

Ver `rules/modes.md`: reutilizan packs reducidos (no full pipeline).

## Resolución de pack por `machine.phase`

| phase | packs |
|-------|--------|
| `idle` | boot |
| `story` | boot + design |
| `contracts` | boot + design |
| `hard_spec` | boot + design |
| `test_plan` | boot + qa |
| `red` | boot + qa + lang |
| `green` | boot + impl + lang |
| `review` | boot + review + lang |
| `mutation` | boot + qa + lang |
| `docs` | boot + close |
| `done_check` | boot + close |
| `spike` | boot + design (read) + lang opcional |
| `hotfix` | boot + qa + impl + lang (path corto) |

## Reglas de ruido

1. **Nunca** cargar los 7 `languages/*` a la vez.  
2. **Nunca** cargar todos los profiles.  
3. Máx. 1 core profile + specialists de `machine.specialists_active` (≤ 3).  
4. Si el mensaje es solo `/status` o `/help` → **solo boot** (+ help command).  
5. Templates: solo el del `hu_type` / comando actual.

## Checklist del orquestador (cada turno)

```
CONTEXT_PACK
phase: …
packs: [boot, …]
lang: …
profiles: […]
artifacts: […]
omitted: [list languages not loaded, …]
```

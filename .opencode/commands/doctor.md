# Comando: `/doctor`

Chequea salud del **repo + proceso**.

## Uso

```
/doctor
/doctor --fix   # solo si el Director pide remediación
```

## Checks

| Check | OK si |
|-------|-------|
| Git | repo limpio o cambios entendibles |
| Spec layout | stories/features/contracts/architecture |
| Knowledge | carpeta existe o se ofrece crearla |
| State | `project-state.md` coherente con `machine.yaml` |
| Machine | phase/gate legales; active_hu vs artifacts |
| Stack tools | fmt/lint/test configurables |
| CI | workflow presente (warn si no) |
| Fábrica | `VERSION` legible; boot_rules presentes; **no** rules de negocio del producto en fábrica |
| Context packs | archivo `agents/context-packs.md` existe |
| Output contracts | `rules/output-contracts.md` existe |
| Drift | código sin spec en features activas |
| Mode | mode ∈ modes.md |

## Output

Score + lista de gaps + plan de remediación opcional.

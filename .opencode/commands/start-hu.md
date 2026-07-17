# Comando: `/start-hu`

Arranca el pipeline SDD+TDD+SRP y **setea la state machine**.

## Uso

```
/start-hu "descripción"
/start-hu "…" --type library|feature|api|cli|adapter
```

## Precondiciones

- `mode` ∈ {`standard`, `strict`}. Si `spike`/`explore`/`hotfix` → proponer `/mode standard` o el comando del modo.  
- `phase: idle` (si hay HU activa, preguntar si aparcar/cancelar).

## Pasos

### 0. Machine

```
phase: story
gate: null
active_hu: HU-###   # siguiente id libre
hu_type: <type>
hu_title: <titulo>
artifacts: reset
specialists_active: []
rework: false
```

Emitir `PHASE_TRANSITION` idle → story.

### 1–9. Pipeline

Igual que antes (PO → … → `/done-hu`), con:

- Context pack por phase (`context-packs.md`).  
- Output contract al fin de cada agent.  
- `/approve` solo en gates de machine.

### Tipos → template hard spec

| type | template |
|------|----------|
| library | hard-spec-library |
| feature | hard-spec-feature |
| api | hard-spec-api-http |
| cli | hard-spec-cli |
| adapter | hard-spec-adapter |

## Specialists

Máx. 3; se anotan en `machine.specialists_active` cuando se invocan.

## Reglas

- No código de prod antes de `phase: green`.  
- No editar `.opencode/` rules/profiles en la HU.  
- Sync `project-state.md` cada transición.

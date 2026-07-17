# Comando: `/init`

Inicializa (o revalida) la **estructura de proyecto** para laburar con la fábrica.  
No reescribe `.opencode/` (la fábrica ya está). Idempotente: si ya hay repo, corre modo doctor+fill-gaps.

## Uso

```
/init [nombre] [--lang rust|ts|go|zig|python|mixed]
```

## Pasos

### 1. Manteca — esqueleto

Crear si faltan:

```
spec/stories/
spec/features/
spec/contracts/
spec/architecture/decisions/
spec/test-plans/
knowledge/{patterns,anti_patterns,conventions,domain,glossary}/
tests/ (o convención del lang)
examples/ benches/ (si lib)
docs/ o architecture/
mutation/   # si Rust u otro con mutation
```

- Git init si no hay.
- Ignore del stack.
- Manifest mínimo (Cargo.toml / package.json / go.mod / build.zig / pyproject) **solo si no existe**.
- README, CONTRIBUTING, CHANGELOG stubs.
- Commit: `chore: inicializar proyecto` (si hay cambios).

### 2. Arqui — overview inicial

`spec/architecture/overview.md` + ADR-001 de stack/estructura si es greenfield.

### 3. State

Actualizar `state/project-state.md`: nombre, lang, fase Inicializado, HU vacías.

### 4. Mensaje al Director

> Listo. Usá `/start-hu "…"` o `/backlog`.

## Reglas

- **No** clavar egui/iced u otras UI si es librería.  
- **No** meter reglas de producto en `.opencode/`.  
- Si el proyecto ya estaba init → no pisar specs existentes.

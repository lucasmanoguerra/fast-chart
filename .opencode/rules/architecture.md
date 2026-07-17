# Arquitectura — Clean, hexagonal, escalable

## Metas

Código mantenible 5–10 años: límites claros, dominio puro, adapters reemplazables, semver predecible.

## Capas (hexagonal / ports & adapters)

```
Domain (puro)
    ↓ ports (traits / interfaces)
Application / use-cases
    ↓
Adapters (IO, DB, HTTP, GPU, FS, OS)
    ↓
Infrastructure / frameworks
```

**Prohibido:** dominio importando frameworks, DB drivers, HTTP clients, GPU, UI toolkits.

## Principios

| Principio | Aplicación |
|-----------|------------|
| SRP | Un motivo de cambio por módulo/archivo |
| OCP | Extender por composición/traits, no reventar cores |
| DIP | High-level no depende de low-level concretos |
| ISP | Interfaces chicas y cohesivas |
| DRY con juicio | No abstraer antes de 2–3 usos reales |
| YAGNI | No features “por si acaso” |
| DDD light | Carpetas por **dominio**, no `utils/helpers/common/misc` |

## Decisiones = ADR

Todo trade-off relevante → `spec/architecture/decisions/` o `architecture/adr/` con template `templates/adr.md`.

Ejemplos: runtime async, crate split, formato de error, backend de render, storage.

## API pública

- Exportar lo mínimo.
- Ocultar internos (`pub(crate)`, barrels controlados, `internal` no re-exportado).
- Cambios breaking solo con major (libs) y mención en changelog.

## Módulos

- Preferir monorepo multi-crate/package cuando crezca (core, types, adapters, testing, benches).
- Cada módulo importante: `README.md` de módulo (template `module-readme.md`).
- Owner implícito por carpeta de dominio (responsabilidad única).

## Diagramas

Architecture overview + mermaid de capas/flujos. Actualizar cuando una HU mueva límites.

## Verificación (architect-guard)

Antes de merge/cierre:

- ¿Rompé capas / hexagonal?  
- ¿Ciclos de dependencia?  
- ¿Tipos internos filtrados?  
- ¿Infra en dominio?  
- ¿Semver?  

Ver perfil `architect-guard` y comando `/review`.

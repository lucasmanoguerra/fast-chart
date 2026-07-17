# Modos de operación

Campo: `state/machine.yaml → mode`  
Cambio: `/mode <nombre>` o pedido explícito del Director humano.

## Modos

### `standard` (default)

- Pipeline HU completo.  
- Gates: hard_spec, test_plan, review si hay hallazgos.  
- Mutation ≥ threshold (default 90%).  
- Quality score al cerrar (informativo si &lt; 85; warn).

### `strict`

- Todo lo de standard **más**:  
  - architect-guard obligatorio si toca `src/` público o capas.  
  - Mutation no salteable; sin equivalents sin doc.  
  - Quality score **overall ≥ 85** para `/done-hu`.  
  - `/public-api` si cambió superficie exportada.  
  - Prohibido hotfix que saltee mutation.

### `spike`

- Time-box obligatorio (`machine.spike_deadline` o horas).  
- Packs: boot + design (lectura) + POC throwaway.  
- **No** merge a main de código de prod.  
- Output: `SPIKE_DONE` + recomendación + posible ADR draft.  
- Cierre → `idle` sin DoD de feature.

### `hotfix`

- Path corto: repro test → fix → review → docs.  
- Story/spec mínimos aceptables (o link a bug issue).  
- Mutation: recomendada; en `strict` obligatoria.  
- Siempre registrar `debt[]` si algo del pipeline largo se omitió.  
- Pensado para fires en producción, no para features nuevas.

### `explore`

- Read-only (salvo notes en knowledge si el humano autoriza).  
- Subagents de exploración.  
- No `/approve` de hard_spec de feature.  
- No commits de implementación.

## Matriz modo × comandos

| Comando | standard | strict | spike | hotfix | explore |
|---------|----------|--------|-------|--------|---------|
| `/start-hu` | sí | sí | no* | no* | no |
| `/spike` | sí | sí | sí | no | sí (read) |
| `/done-hu` | sí | sí+score | no | sí (DoD reducido) | no |
| `/approve` | sí | sí | soft | sí | no |
| writes a src/ | pipeline | pipeline | POC only | sí | no |

\* En spike/hotfix usar sus comandos; si piden feature normal → `/mode standard` + `/start-hu`.

## Output al cambiar modo

```
MODE_CHANGED
from: …
to: …
reason: …
constraints: …
```

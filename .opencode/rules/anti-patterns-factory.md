# Anti-patrones de la fábrica (proceso)

Fallos de **cómo** laburamos, no del dominio del producto.  
Si aparece uno: frenar, nombrar el anti-patrón, corregir.

| ID | Anti-patrón | Por qué duele | Qué hacer |
|----|-------------|---------------|-----------|
| F01 | **God-agent** | Un rol hace PO+spec+code+review | Un core agent por fase; matrix |
| F02 | **Spec HTTP en lib** | Template equivocado | `hard-spec-library` / adapter |
| F03 | **Specialist spam** | >3 specialists o todos los profiles | max 3; context packs |
| F04 | **Lang pack completo** | Cargar rust+go+zig+… | solo `lang:<stack>` |
| F05 | **Codear sin approve** | phase &lt; test_plan/red | respetar machine |
| F06 | **Spike eterno** | POC se vuelve el producto | time-box; `/start-hu` después |
| F07 | **Spike mergeable** | Basura sin spec en main | branch throwaway |
| F08 | **Editar `.opencode` en feature** | Contamina la fábrica | solo pedido explícito de fábrica |
| F09 | **Approve sin leer** | Gates de adorno | `/review` primero |
| F10 | **Tests de implementación** | Acoplados a privados | comportamiento del spec |
| F11 | **Mutation theater** | No correr o bajar threshold a escondidas | report + equivalents |
| F12 | **State mentiroso** | markdown ≠ machine ≠ repo | sync en cada transición |
| F13 | **Handoff en prosa libre** | El siguiente agent no parsea | output contracts |
| F14 | **Multi-agent code race** | Dos writers en los mismos files | un writer; council solo lectura |
| F15 | **Hotfix disfrazado de feature** | Saltea diseño en features grandes | `/mode standard` + HU |
| F16 | **Knowledge en el chat nomas** | Se pierde | `/knowledge` + archivo en repo |
| F17 | **DoD de word** | “casi listo” | `/done-hu` + score |
| F18 | **Public API accidental** | `pub` de más sin `/public-api` | ux-api + semver |

## Respuesta estándar del orquestador

```
ANTI_PATTERN
id: F0x
name: …
evidence: …
required_action: …
```

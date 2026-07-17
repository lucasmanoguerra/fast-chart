# Definition of Done (HU)

Una HU **no está terminada** hasta cumplir (o justificar en el state):

## Correctness

- [ ] Hard Spec aprobado y alineado con la implementación  
- [ ] Tests de aceptación del spec en verde  
- [ ] Unit tests relevantes en verde  
- [ ] Mutation ≥ umbral (o equivalentes documentados)  

## Diseño

- [ ] SRP respetado (review OK)  
- [ ] Capas/hexagonal OK (architect-guard si tocaba límites)  
- [ ] Sin deps ciclicas nuevas  
- [ ] Public API revisada si cambió (`/public-api`)  

## Tooling del stack

- [ ] Format + lint limpios  
- [ ] Build/tests en features relevantes del package  

## Docs & knowledge

- [ ] rustdoc/TSDoc/godoc de superficie pública actualizado  
- [ ] README / architecture / changelog si aplica  
- [ ] ADR si hubo decisión  
- [ ] `knowledge/` actualizado si nació un patrón o anti-patrón  
- [ ] Module README si se creó módulo de dominio  

## Repo hygiene

- [ ] State de la HU en `project-state.md` → completada  
- [ ] Commit(s) atómicos con mensaje claro  
- [ ] Issue/PR actualizados si se usa `gh`  

## Perf (si la HU es hot path)

- [ ] Benchmark o medición acordada sin regresión injustificada  

Usá `/done-hu` para recorrer este checklist con el orquestador.

## Quality score

Además del checklist, `/done-hu` genera `spec/reviews/<hu>.md`  
(template `templates/quality-score.md`).

- **standard:** score obligatorio; warn si overall &lt; 85.  
- **strict:** overall ≥ 85 para cerrar (`machine.quality_score_min_strict`).

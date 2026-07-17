# Quality score: {{hu_id}} — {{title}}

> **Fecha**: {{date}}  
> **Mode**: {{mode}}  
> **Overall**: {{overall}} / 100 · **Pass**: {{pass}}  
> **Strict min**: 85 (solo mode=strict bloquea)

---

## Scores (0–100)

| Eje | Score | Notas |
|-----|------:|-------|
| Architecture | {{architecture}} | capas, SRP, ciclos, domain limpio |
| Testing | {{testing}} | pirámide, mutation, trazabilidad Gherkin |
| API | {{api}} | superficie pública, semver, ergonomía |
| Docs / knowledge | {{docs}} | API docs, changelog, knowledge, module README |
| Perf risk | {{perf_risk}} | 100 si no hay hot path; si hay, benches/justificación |

**Overall** = promedio simple de los 5 ejes (redondeo entero).

---

## Evidencia

### Architecture
- 

### Testing
- Mutation score:  
- Tests commands:  

### API
- `/public-api` corrido: sí/no  
- Breaking: sí/no  

### Docs
- 

### Perf
- Hot path: sí/no  
- Bench:  

---

## Blockers (si overall fail o eje crítico)

| Severidad | Ítem | Acción |
|-----------|------|--------|
| | | |

---

## Decisión

- [ ] Cerrar HU (`phase → idle`)  
- [ ] Volver a fase: {{phase}}  

```
QUALITY_SCORE
hu: {{hu_id}}
path: spec/reviews/{{hu_id}}.md
architecture: {{architecture}}
testing: {{testing}}
api: {{api}}
docs: {{docs}}
perf_risk: {{perf_risk}}
overall: {{overall}}
pass: {{pass}}
```

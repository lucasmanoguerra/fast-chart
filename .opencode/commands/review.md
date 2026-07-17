# Comando: `/review`

Review del artefacto o diff actual sin necesariamente aprobar.

## Uso

```
/review
/review spec
/review diff
/review architecture
```

## Quién actúa

- Spec → SpecWriter resume + Director lee  
- Diff → Revisor (+ architect-guard opcional)  
- Architecture → Arqui / architect-guard  

Salida: hallazgos + recomendación approve/reject/iterate.

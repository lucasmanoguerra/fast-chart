# Comando: `/adr`

Crea o actualiza un Architecture Decision Record.

## Uso

```
/adr "por qué wgpu y no glium"
/adr ADR-003
```

## Pasos

1. Cargar `arqui` (+ specialists si aplica).  
2. Redactar con `templates/adr.md`.  
3. Path: `spec/architecture/decisions/adr-NNN-slug.md`.  
4. Soft approve del Director.  
5. Link desde overview + state.

No reemplaza Hard Spec de feature; documenta **decisión**.

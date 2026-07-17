# Comando: `/knowledge`

Captura una lección en la **knowledge base del proyecto** (no en `.opencode`).

## Uso

```
/knowledge "no usar Arc en el hot path de X"
/knowledge --pattern repository
/knowledge --anti "god module utils"
```

## Pasos

1. Elegir path bajo `knowledge/…`.  
2. Escribir nota corta: contexto, regla, ejemplo, HU origen.  
3. Link desde state o ADR si aplica.  
4. Engram opcional como puntero.

## Reglas

Nunca guardar secretos. Nunca mover proceso genérico de la fábrica al knowledge del producto ni viceversa sin criterio.

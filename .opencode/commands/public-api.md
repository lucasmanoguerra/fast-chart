# Comando: `/public-api`

Analiza impacto en la **superficie pública** y semver.

## Uso

```
/public-api
/public-api --diff main
```

## Pasos

1. `ux-api` + `arqui` (+ tool del stack: cargo-public-api, attw, etc.).  
2. Listar símbolos añadidos/rotos/cambiados.  
3. Clasificar: patch | minor | major.  
4. Pedir changelog + tests de contrato si major/minor.  
5. Opcional: architect-guard.

Especialmente crítico en **librerías**.

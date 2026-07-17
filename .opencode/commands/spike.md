# Comando: `/spike`

Investigación **time-boxed** sin delivery de feature.

## Uso

```
/spike "¿glyphon 0.6 vs 0.7 con wgpu 22?" --hours 4
```

## Pasos

1. Template `spike.md` en `spec/spikes/`.  
2. POC en branch throwaway o `spikes/`.  
3. Hallazgos + recomendación.  
4. Si hay decisión → `/adr`.  
5. Knowledge update.  
6. **No** mergear POC como feature sin HU.

## Reglas

- Time-box estricto.  
- Prohibido “spike eterno” que se vuelve el producto.

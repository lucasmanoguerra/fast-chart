# Comando: `/bench`

Plan y ejecución de benchmarks / presupuestos de perf.

## Uso

```
/bench
/bench "hot path layout"
/bench --compare main
```

## Pasos

1. `perf` + `benchmark` definen métricas.  
2. Asegurar benches en el repo (`benches/` u equiv.).  
3. Correr y reportar.  
4. Guardar aprendizaje en `knowledge/benchmarks/`.  
5. Si regresión injustificada → no `/done-hu` de HU de perf.

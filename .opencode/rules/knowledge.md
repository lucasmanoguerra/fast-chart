# Knowledge base del proyecto

La fábrica (`.opencode/`) **no** es la memoria del producto.  
El conocimiento acumulado vive en el **repo del proyecto**.

## Estructura sugerida

```
knowledge/
  architecture/      # notas largas, diagramas vivos
  domain/            # glosario de dominio
  glossary/
  decisions/         # punteros a ADRs o resúmenes
  patterns/
  anti_patterns/
  benchmarks/        # aprendizajes de perf
  constraints/
  conventions/       # naming, errores, logging del proyecto
  examples/
  <stack>/           # rust/, typescript/, etc. lecciones del stack en ESTE repo
```

## Qué guardar

- Decisiones y porqués (además del ADR formal)  
- Patrones que funcionaron (repository, typestate, arena…)  
- Anti-patrones que ya nos quemaron  
- Invariantes de dominio  
- Lecciones de unsafe/async/GPU/concurrencia **de este proyecto**  

## Qué no guardar acá

- Proceso genérico de la fábrica → `.opencode/rules/`  
- Spec de una feature → `spec/features/`  
- Secretos, tokens, datos personales  

## Operación

- Comando `/knowledge` para capturar una lección al cerrar HU o spike.  
- Manteca se asegura de no perder aprendizajes al mergear.  
- Engram puede indexar punteros; la fuente de verdad es el archivo en git.

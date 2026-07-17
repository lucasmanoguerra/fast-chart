# Orquestación multi-agente

## Modelo mental

```
Director (orquestador)
   ├── fase actual (1 core agent en el chat principal)
   ├── specialists on-demand (consultas puntuales)
   └── subagents (tareas aisladas, contexto mínimo)
```

El Director **no** paraleliza implementación y spec del mismo feature sin gates.

## Patrones

### 1) Pipeline secuencial (default)

PO → Arqui → SpecWriter → approve → QA → Dev → Review → QA mutation → Manteca.

### 2) Review council (paralelo de lectura)

Tras un diff grande:

1. Spawn/consulta `revisor` (SRP, estilo)  
2. En paralelo: `architect-guard` (capas)  
3. Opcional: `security` o `perf`  
4. El Director **fusiona** hallazgos, prioriza, y manda a DevSenior un solo backlog de fixes.

No tres hilos escribiendo código a la vez.

### 3) Design fork

Dos propuestas de Arqui (A/B) documentadas en un spike o ADR draft → Director elige → un solo camino al SpecWriter.

### 4) Knowledge capture

Al cerrar: Manteca + `/knowledge` en paralelo al changelog.

## Handoff template

Cuando un agent le pasa la posta a otro:

```
HANDOFF
from: {{agent}}
to: {{agent}}
hu: {{id}}
phase: {{n}}
artifacts:
  - path: …
decisions:
  - …
open_questions:
  - …
```

## Anti-patrones

- Tres agents editando los mismos archivos  
- Specialist que reescribe el Hard Spec sin SpecWriter  
- Cargar 10 profiles “por contexto”  
- Saltear approve porque “es obvio”  

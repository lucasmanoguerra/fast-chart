# Multi-agente, subagentes y anti-ruido

## Roles

| Tipo | Quién | Qué hace |
|------|-------|----------|
| **Orquestador** | Director (`system-prompt`) | Elige fase, agents, gates; actualiza state |
| **Core agent** | po, arqui, specwriter, qargento, devsenior, revisor, manteca | Pipeline de HU |
| **Specialist** | perf, security, graphics, … | Solo si la HU lo necesita |
| **Subagent** | Tarea acotada spawnada | Un deliverable chico, contexto mínimo |
| **Guard** | architect-guard, revisor | Veto de calidad |

## Límites de ruido

- Máx. **1 core agent activo** en la conversación principal por turno de fase.  
- Máx. **3 specialists** por HU completa.  
- Nunca cargar todos los profiles.  
- Language rules: solo stacks detectados (`context-packs.md`).  
- Contexto por tarea: pack de phase + spec + contratos + blast radius + ADRs.  
- Emitir `CONTEXT_PACK` al inicio del turno del orquestador.

## Multi-agente (`/multi-agent`)

Usar cuando:

- Una fase necesita **varias lecturas en paralelo** (ej. security + perf review del mismo diff).  
- Hay que **contrastar** dos diseños (council corto).  
- El blast radius es grande y conviene mapear en paralelo.

Protocolo resumido: ver `agents/orchestration.md`.

## Subagentes (`/subagent`)

Usar para:

- Explorar repo / buscar símbolos  
- Redactar un ADR draft  
- Correr y resumir `cargo test` / lint  
- Generar plan de tests **borrador**  

Reglas:

1. Prompt autocontenido (el subagente no asume chat previo).  
2. Un output claro (archivo path o resumen estructurado).  
3. Read-only por default; write solo si el Director lo autoriza.  
4. No saltear gates de approve.  

Detalle: `agents/subagents.md`.

## Matriz

Ver `agents/matrix.md` para “quién entra en qué fase”.

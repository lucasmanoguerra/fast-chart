# Contrato (Rust trait / interface de dominio): {{Nombre}}

> **Módulo**: {{path}}  
> **HU**: {{id}}  
> **Estabilidad**: experimental | stable  

---

## Responsabilidad

{{una_sola}}

## No-responsabilidades

- {{}}

---

## Definición

```rust
/// {{docs}}
pub trait {{Nombre}} {
    type Error;

    /// {{docs_metodo}}
    fn {{metodo}}(&self, {{args}}) -> Result<{{Ret}}, Self::Error>;
}
```

## Tipos asociados / DTOs

```rust
pub struct {{Dto}} { /* fields */ }
```

## Errores

| Variante | Cuándo |
|----------|--------|
| | |

## Invariantes

- {{}}

## Implementaciones previstas

| Impl | Crate/módulo | Notas |
|------|--------------|-------|
| | | production / test fake |

## Dependencias de otros ports

| Trait | Para qué |
|-------|----------|
| | |

## SemVer

¿Se exporta en public API? sí/no · implicancias.

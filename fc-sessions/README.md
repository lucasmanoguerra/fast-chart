# fc-sessions

Definiciones de sesiones de trading y renderizado de líneas verticales de mercado. Las sesiones definen cuándo un mercado está abierto. Las líneas de sesión se renderizan como líneas verticales en el chart en los horarios configurados.

## Uso

```rust
use fc_sessions::{Session, ExchangeSessions, SessionLineConfig, SessionLineRenderer};

// Sesión predefinida
let us_regular = ExchangeSessions::us_regular();
assert_eq!(us_regular.duration_minutes(), 390);

// Sesión custom
let session = Session::new("Asia", 0, 0, 6, 0);
assert!(session.contains_utc(3, 0));

// Renderizado de líneas
let config = SessionLineConfig {
    sessions: vec![ExchangeSessions::us_regular()],
    ..Default::default()
};
let renderer = SessionLineRenderer::new(config);
let lines = renderer.render(0.0, 24.0, 0.0, 600.0, |h| h * 50.0);
assert_eq!(lines.len(), 2); // open + close
```

## Dependencias

- `fc-primitives` — color Rgba

## Estructura

| Módulo | Descripción |
|--------|-------------|
| (raíz) | `Session`, `ExchangeSessions`, `SessionLineStyle`, `SessionLineConfig`, `SessionLineRenderer`, `SessionLine` |

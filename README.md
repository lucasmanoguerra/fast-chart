<![CDATA[<div align="center">

# ⚡ Fast Chart

**GPU-accelerated trading chart library for Rust**

High-performance • Low-latency • Hexagonal Architecture • 7 Indicators • 8-Layer GPU Pipeline

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-324%20passing-brightgreen)]()
[![wgpu](https://img.shields.io/badge/wgpu-22-purple)](https://wgpu.rs)

[English](#english) • [Español](#español) • [中文](#中文) • [Português](#português)

</div>

---

## English

### What is Fast Chart?

Fast Chart is a **professional-grade trading chart library** written in Rust, designed for applications that demand **real-time rendering**, **sub-millisecond latency**, and **deterministic behavior**. Built on [wgpu](https://wgpu.rs) for GPU-accelerated rendering with a clean hexagonal architecture that keeps domain logic pure and testable.

### Key Features

| Feature | Description |
|---------|-------------|
| 🚀 **GPU Rendering** | 8-layer wgpu pipeline: Grid → Candlestick → Line → Divider → Crosshair → Markers → Price Lines → Text |
| 📊 **7 Indicators** | SMA, EMA, RSI, MACD, Bollinger Bands, Stochastic, Ichimoku Cloud |
| 🏗️ **Hexagonal Architecture** | Domain → Core → App layers with strict dependency direction |
| 🎯 **Multi-Pane Layout** | Vertical pane stack with draggable dividers and per-pane scissor rects |
| ⚡ **Kinetic Scrolling** | Momentum-based pan with configurable friction |
| 🖱️ **Interactive Crosshair** | Real-time cursor tracking with OHLC tooltip |
| 📐 **Viewport Management** | Zoom, pan, auto-fit with price/time coordinate conversion |
| 🔌 **Pluggable Data** | `DataProvider` trait — plug any data source (WebSocket, REST, file) |
| 🧪 **324 Tests** | Domain, core, integration — all passing, zero warnings |

### Architecture

```
┌─────────────────────────────────────────────────┐
│                  fast-chart-app                  │
│  (adapters: GPU, input, data, config)           │
├─────────────────────────────────────────────────┤
│                 fast-chart-core                  │
│  (app: controller, layout, pane, viewport)      │
│  (ports: DataProvider, InteractionHandler)       │
├─────────────────────────────────────────────────┤
│                fast-chart-domain                 │
│  (Bar, Viewport, Crosshair, Indicators, etc.)   │
└─────────────────────────────────────────────────┘
```

**Dependency rule:** Domain depends on nothing. Core depends only on Domain. App depends on Core.

### Quick Start

```rust
// domain types
use fast_chart_domain::bar::Bar;
use fast_chart_domain::viewport::Viewport;

// core controller
use fast_chart_core::app::chart_controller::ChartController;

// Create a chart controller with your data provider
let controller = ChartController::new(
    Box::new(my_data_provider),
    Box::new(my_interaction_handler),
);

// Tick to process data
controller.tick();

// Access state
let state = controller.state();
println!("Bars loaded: {}", state.time_series.len());
```

### Rendering Pipeline (8 Layers)

| Layer | Renderer | Description |
|-------|----------|-------------|
| 1 | `GridRenderer` | Background grid lines |
| 2 | `CandleRenderer` | OHLC candlestick bars (world-space, GPU-projected) |
| 3 | `LineRenderer` | Close-price line series (NDC-space, CPU-projected) |
| 4 | Divider lines | Pane separators |
| 5 | `CrosshairRenderer` | Cursor crosshair lines |
| 6 | `MarkerRenderer` | Trade annotations (per-pane scissor) |
| 7 | `PriceLineRenderer` | Horizontal price levels (per-pane scissor) |
| 8 | `GpuTextRenderer` | Axis labels + crosshair tooltip (glyphon) |

### Indicators

| Indicator | Description | Parameters |
|-----------|-------------|------------|
| **SMA** | Simple Moving Average | period |
| **EMA** | Exponential Moving Average | period |
| **RSI** | Relative Strength Index | period |
| **MACD** | Moving Average Convergence Divergence | fast, slow, signal |
| **Bollinger** | Bollinger Bands | period, std_dev |
| **Stochastic** | Stochastic Oscillator | k_period, d_period |
| **Ichimoku** | Ichimoku Cloud | tenkan, kijun, senkou_b |

### Project Structure

```
fast-chart/
├── fast-chart-domain/        # Zero-dependency domain types
│   └── src/
│       ├── bar.rs            # OHLCV bar
│       ├── viewport.rs       # Viewport (time + value range)
│       ├── crosshair.rs      # Crosshair state + magnet
│       ├── indicator.rs      # Indicator trait
│       ├── indicators/       # 7 built-in indicators
│       ├── series.rs         # Ring-buffer time series
│       ├── marker.rs         # Trade markers
│       ├── price_line.rs     # Horizontal price lines
│       └── price_scale.rs    # Price formatting
├── fast-chart-core/          # Application logic (wgpu-free)
│   └── src/
│       ├── app/
│       │   ├── chart_controller.rs  # Central orchestrator
│       │   ├── layout_manager.rs    # Multi-pane layout
│       │   ├── pane.rs             # Pane (viewport, markers, scales)
│       │   └── viewport_management.rs  # Coordinate conversion
│       └── ports/
│           ├── data_provider.rs     # Data source trait
│           ├── interaction.rs       # User input trait
│           └── render.rs            # Renderer trait
├── fast-chart-app/           # wgpu rendering + winit window
│   └── src/
│       ├── adapters/
│       │   ├── gpu_renderer.rs      # Main GPU orchestrator
│       │   ├── rendering/           # 8 sub-renderers
│       │   ├── data/                # Data adapters
│       │   └── input/               # Input handling
│       └── config/                  # TOML config + hot-reload
└── Cargo.toml                # Workspace root
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `wgpu` 22 | GPU rendering |
| `winit` 0.30 | Window management |
| `glyphon` 0.6 | Text rendering |
| `bytemuck` | GPU buffer casting |
| `pollster` | Async block_on |
| `serde` + `toml` | Configuration |
| `notify` | Config hot-reload |

### Testing

```bash
# Run all 324 tests
cargo test --workspace

# Run domain tests only (173)
cargo test -p fast-chart-domain

# Run core tests only (120)
cargo test -p fast-chart-core

# Run app tests only (31)
cargo test -p fast-chart-app
```

### Contributing

1. Create a feature branch: `git checkout -b feat/my-feature`
2. Make changes and add tests
3. Ensure all tests pass: `cargo test --workspace`
4. Commit with conventional commits: `feat: add new indicator`
5. Push and create a PR to `main`

### License

MIT

---

## Español

### ¿Qué es Fast Chart?

Fast Chart es una **biblioteca de gráficos para trading de grado profesional** escrita en Rust, diseñada para aplicaciones que requieren **renderizado en tiempo real**, **latencia sub-milisegundo** y **comportamiento determinista**. Construida sobre [wgpu](https://wgpu.rs) para renderizado acelerado por GPU con una arquitectura hexagonal limpia que mantiene la lógica de dominio pura y testeable.

### Características Principales

| Característica | Descripción |
|----------------|-------------|
| 🚀 **Renderizado GPU** | Pipeline wgpu de 8 capas: Grid → Candlestick → Line → Divider → Crosshair → Markers → Price Lines → Text |
| 📊 **7 Indicadores** | SMA, EMA, RSI, MACD, Bandas de Bollinger, Estocástico, Nube de Ichimoku |
| 🏗️ **Arquitectura Hexagonal** | Capas Domain → Core → App con dirección estricta de dependencias |
| 🎯 **Layout Multi-Pane** | Stack vertical de panes con divisores arrastrables y scissor rects por pane |
| ⚡ **Scrolling Kinético** | Pan con momento y fricción configurable |
| 🖱️ **Crosshair Interactivo** | Tracking del cursor en tiempo real con tooltip OHLC |
| 📐 **Gestión de Viewport** | Zoom, pan, auto-fit con conversión de coordenadas precio/tiempo |
| 🔌 **Data Pluggable** | Trait `DataProvider` — conectá cualquier fuente de datos (WebSocket, REST, archivo) |
| 🧪 **324 Tests** | Dominio, core, integración — todos pasando, cero warnings |

### Arquitectura

```
┌─────────────────────────────────────────────────┐
│                  fast-chart-app                  │
│  (adaptadores: GPU, input, data, config)        │
├─────────────────────────────────────────────────┤
│                 fast-chart-core                  │
│  (app: controller, layout, pane, viewport)      │
│  (ports: DataProvider, InteractionHandler)       │
├─────────────────────────────────────────────────┤
│                fast-chart-domain                 │
│  (Bar, Viewport, Crosshair, Indicators, etc.)   │
└─────────────────────────────────────────────────┘
```

**Regla de dependencia:** Domain no depende de nada. Core depende solo de Domain. App depende de Core.

### Inicio Rápido

```rust
use fast_chart_domain::bar::Bar;
use fast_chart_core::app::chart_controller::ChartController;

let controller = ChartController::new(
    Box::new(mi_fuente_datos),
    Box::new(mi_manejador_interaccion),
);

controller.tick();

let state = controller.state();
println!("Barras cargadas: {}", state.time_series.len());
```

### Pipeline de Renderizado (8 Capas)

| Capa | Renderer | Descripción |
|------|----------|-------------|
| 1 | `GridRenderer` | Líneas de grilla de fondo |
| 2 | `CandleRenderer` | Barras OHLC (world-space, GPU-proyectado) |
| 3 | `LineRenderer` | Línea de cierre (NDC-space, CPU-proyectado) |
| 4 | Líneas divisorias | Separadores de pane |
| 5 | `CrosshairRenderer` | Líneas del crosshair |
| 6 | `MarkerRenderer` | Anotaciones de trades (scissor por pane) |
| 7 | `PriceLineRenderer` | Niveles de precio horizontales (scissor por pane) |
| 8 | `GpuTextRenderer` | Labels del eje + tooltip del crosshair (glyphon) |

### Estructura del Proyecto

```
fast-chart/
├── fast-chart-domain/        # Tipos de dominio sin dependencias
│   └── src/
│       ├── bar.rs            # Barra OHLCV
│       ├── viewport.rs       # Viewport (rango tiempo + valor)
│       ├── crosshair.rs      # Estado del crosshair + magneto
│       ├── indicator.rs      # Trait de indicador
│       ├── indicators/       # 7 indicadores incluidos
│       ├── series.rs         # TimeSeries ring-buffer
│       ├── marker.rs         # Marcadores de trades
│       ├── price_line.rs     # Líneas de precio horizontales
│       └── price_scale.rs    # Formateo de precios
├── fast-chart-core/          # Lógica de aplicación (sin wgpu)
│   └── src/
│       ├── app/
│       │   ├── chart_controller.rs  # Orquestador central
│       │   ├── layout_manager.rs    # Layout multi-pane
│       │   ├── pane.rs             # Pane (viewport, markers, scales)
│       │   └── viewport_management.rs  # Conversión de coordenadas
│       └── ports/
│           ├── data_provider.rs     # Trait fuente de datos
│           ├── interaction.rs       # Trait de input del usuario
│           └── render.rs            # Trait de renderer
├── fast-chart-app/           # Renderizado wgpu + ventana winit
│   └── src/
│       ├── adapters/
│       │   ├── gpu_renderer.rs      # Orquestador GPU principal
│       │   ├── rendering/           # 8 sub-renderers
│       │   ├── data/                # Adaptadores de datos
│       │   └── input/               # Manejo de input
│       └── config/                  # Config TOML + hot-reload
└── Cargo.toml                # Raíz del workspace
```

### Contribuir

1. Crear rama de feature: `git checkout -b feat/mi-feature`
2. Hacer cambios y agregar tests
3. Asegurar que todos los tests pasen: `cargo test --workspace`
4. Commitear con convenciones: `feat: agregar nuevo indicador`
5. Push y crear PR a `main`

### Licencia

MIT

---

## 中文

### 什么是 Fast Chart？

Fast Chart 是一个**专业级交易图表库**，使用 Rust 编写，专为需要**实时渲染**、**亚毫秒延迟**和**确定性行为**的应用程序设计。基于 [wgpu](https://wgpu.rs) 构建，支持 GPU 加速渲染，采用干净的六边形架构，保持领域逻辑纯粹且可测试。

### 核心特性

| 特性 | 描述 |
|------|------|
| 🚀 **GPU 渲染** | 8 层 wgpu 管线：Grid → Candlestick → Line → Divider → Crosshair → Markers → Price Lines → Text |
| 📊 **7 个指标** | SMA, EMA, RSI, MACD, 布林带, 随机指标, 一目均衡图 |
| 🏗️ **六边形架构** | Domain → Core → App 层，严格的依赖方向 |
| 🎯 **多面板布局** | 垂直面板堆叠，可拖动分隔器，每面板独立裁剪区域 |
| ⚡ **惯性滚动** | 基于动量的平移，可配置摩擦力 |
| 🖱️ **交互式十字光标** | 实时光标追踪，带 OHLC 工具提示 |
| 📐 **视口管理** | 缩放、平移、自动适应，价格/时间坐标转换 |
| 🔌 **可插拔数据** | `DataProvider` trait — 接入任何数据源（WebSocket、REST、文件） |
| 🧪 **324 个测试** | 领域、核心、集成 — 全部通过，零警告 |

### 架构

```
┌─────────────────────────────────────────────────┐
│                  fast-chart-app                  │
│  (适配器：GPU, input, data, config)             │
├─────────────────────────────────────────────────┤
│                 fast-chart-core                  │
│  (app: controller, layout, pane, viewport)      │
│  (ports: DataProvider, InteractionHandler)       │
├─────────────────────────────────────────────────┤
│                fast-chart-domain                 │
│  (Bar, Viewport, Crosshair, Indicators 等)      │
└─────────────────────────────────────────────────┘
```

**依赖规则：** Domain 不依赖任何东西。Core 仅依赖 Domain。App 依赖 Core。

### 快速开始

```rust
use fast_chart_domain::bar::Bar;
use fast_chart_core::app::chart_controller::ChartController;

let controller = ChartController::new(
    Box::new(我的数据源),
    Box::new(我的交互处理器),
);

controller.tick();

let state = controller.state();
println!("已加载 K 线数: {}", state.time_series.len());
```

### 渲染管线（8 层）

| 层 | 渲染器 | 描述 |
|----|--------|------|
| 1 | `GridRenderer` | 背景网格线 |
| 2 | `CandleRenderer` | OHLC K 线（世界空间，GPU 投影） |
| 3 | `LineRenderer` | 收盘价折线（NDC 空间，CPU 投影） |
| 4 | 分隔线 | 面板分隔器 |
| 5 | `CrosshairRenderer` | 十字光标线 |
| 6 | `MarkerRenderer` | 交易标注（每面板裁剪） |
| 7 | `PriceLineRenderer` | 水平价格线（每面板裁剪） |
| 8 | `GpuTextRenderer` | 轴标签 + 十字光标提示（glyphon） |

### 项目结构

```
fast-chart/
├── fast-chart-domain/        # 零依赖的领域类型
│   └── src/
│       ├── bar.rs            # OHLCV K 线
│       ├── viewport.rs       # 视口（时间 + 价格范围）
│       ├── crosshair.rs      # 十字光标状态 + 磁吸
│       ├── indicator.rs      # 指标 trait
│       ├── indicators/       # 7 个内置指标
│       ├── series.rs         # 环形缓冲时间序列
│       ├── marker.rs         # 交易标记
│       ├── price_line.rs     # 水平价格线
│       └── price_scale.rs    # 价格格式化
├── fast-chart-core/          # 应用逻辑（无 wgpu）
│   └── src/
│       ├── app/
│       │   ├── chart_controller.rs  # 中央协调器
│       │   ├── layout_manager.rs    # 多面板布局
│       │   ├── pane.rs             # 面板（视口、标记、刻度）
│       │   └── viewport_management.rs  # 坐标转换
│       └── ports/
│           ├── data_provider.rs     # 数据源 trait
│           ├── interaction.rs       # 用户输入 trait
│           └── render.rs            # 渲染器 trait
├── fast-chart-app/           # wgpu 渲染 + winit 窗口
│   └── src/
│       ├── adapters/
│       │   ├── gpu_renderer.rs      # 主 GPU 协调器
│       │   ├── rendering/           # 8 个子渲染器
│       │   ├── data/                # 数据适配器
│       │   └── input/               # 输入处理
│       └── config/                  # TOML 配置 + 热重载
└── Cargo.toml                # 工作区根目录
```

### 贡献

1. 创建特性分支：`git checkout -b feat/my-feature`
2. 进行更改并添加测试
3. 确保所有测试通过：`cargo test --workspace`
4. 使用约定提交：`feat: add new indicator`
5. 推送并创建 PR 到 `main`

### 许可证

MIT

---

## Português

### O que é Fast Chart?

Fast Chart é uma **biblioteca de gráficos para trading de grau profissional** escrita em Rust, projetada para aplicações que exigem **renderização em tempo real**, **latência sub-milissegundo** e **comportamento determinístico**. Construída sobre [wgpu](https://wgpu.rs) para renderização acelerada por GPU com uma arquitetura hexagonal limpa que mantém a lógica de domínio pura e testável.

### Principais Funcionalidades

| Funcionalidade | Descrição |
|----------------|-----------|
| 🚀 **Renderização GPU** | Pipeline wgpu de 8 camadas: Grid → Candlestick → Line → Divider → Crosshair → Markers → Price Lines → Text |
| 📊 **7 Indicadores** | SMA, EMA, RSI, MACD, Bandas de Bollinger, Estocástico, Nuvem de Ichimoku |
| 🏗️ **Arquitetura Hexagonal** | Camadas Domain → Core → App com direção estrita de dependências |
| 🎯 **Layout Multi-Pane** | Stack vertical de panes com divisores arrastáveis e scissor rects por pane |
| ⚡ **Scrolling Kinético** | Pan com momento e fricção configurável |
| 🖱️ **Crosshair Interativo** | Rastreamento do cursor em tempo real com tooltip OHLC |
| 📐 **Gerenciamento de Viewport** | Zoom, pan, auto-fit com conversão de coordenadas preço/tempo |
| 🔌 **Data Pluggable** | Trait `DataProvider` — conecte qualquer fonte de dados (WebSocket, REST, arquivo) |
| 🧪 **324 Testes** | Domínio, core, integração — todos passando, zero warnings |

### Arquitetura

```
┌─────────────────────────────────────────────────┐
│                  fast-chart-app                  │
│  (adaptadores: GPU, input, data, config)        │
├─────────────────────────────────────────────────┤
│                 fast-chart-core                  │
│  (app: controller, layout, pane, viewport)      │
│  (ports: DataProvider, InteractionHandler)       │
├─────────────────────────────────────────────────┤
│                fast-chart-domain                 │
│  (Bar, Viewport, Crosshair, Indicators, etc.)   │
└─────────────────────────────────────────────────┘
```

**Regra de dependência:** Domain não depende de nada. Core depende apenas de Domain. App depende de Core.

### Início Rápido

```rust
use fast_chart_domain::bar::Bar;
use fast_chart_core::app::chart_controller::ChartController;

let controller = ChartController::new(
    Box::new(minha_fonte_dados),
    Box::new(meu_gerenciador_interacao),
);

controller.tick();

let state = controller.state();
println!("Barras carregadas: {}", state.time_series.len());
```

### Pipeline de Renderização (8 Camadas)

| Camada | Renderer | Descrição |
|--------|----------|-----------|
| 1 | `GridRenderer` | Linhas da grade de fundo |
| 2 | `CandleRenderer` | Barras OHLC (world-space, GPU-projetado) |
| 3 | `LineRenderer` | Linha de fechamento (NDC-space, CPU-projetado) |
| 4 | Linhas divisórias | Separadores de pane |
| 5 | `CrosshairRenderer` | Linhas do crosshair |
| 6 | `MarkerRenderer` | Anotações de trades (scissor por pane) |
| 7 | `PriceLineRenderer` | Níveis de preço horizontais (scissor por pane) |
| 8 | `GpuTextRenderer` | Labels do eixo + tooltip do crosshair (glyphon) |

### Estrutura do Projeto

```
fast-chart/
├── fast-chart-domain/        # Tipos de domínio sem dependências
│   └── src/
│       ├── bar.rs            # Barra OHLCV
│       ├── viewport.rs       # Viewport (intervalo tempo + valor)
│       ├── crosshair.rs      # Estado do crosshair + magneto
│       ├── indicator.rs      # Trait de indicador
│       ├── indicators/       # 7 indicadores incluídos
│       ├── series.rs         # TimeSeries ring-buffer
│       ├── marker.rs         # Marcadores de trades
│       ├── price_line.rs     # Linhas de preço horizontais
│       └── price_scale.rs    # Formatação de preços
├── fast-chart-core/          # Lógica da aplicação (sem wgpu)
│   └── src/
│       ├── app/
│       │   ├── chart_controller.rs  # Orquestrador central
│       │   ├── layout_manager.rs    # Layout multi-pane
│       │   ├── pane.rs             # Pane (viewport, markers, scales)
│       │   └── viewport_management.rs  # Conversão de coordenadas
│       └── ports/
│           ├── data_provider.rs     # Trait fonte de dados
│           ├── interaction.rs       # Trait de input do usuário
│           └── render.rs            # Trait de renderer
├── fast-chart-app/           # Renderização wgpu + janela winit
│   └── src/
│       ├── adapters/
│       │   ├── gpu_renderer.rs      # Orquestrador GPU principal
│       │   ├── rendering/           # 8 sub-renderers
│       │   ├── data/                # Adaptadores de dados
│       │   └── input/               # Gerenciamento de input
│       └── config/                  # Config TOML + hot-reload
└── Cargo.toml                # Raiz do workspace
```

### Contribuir

1. Criar branch de feature: `git checkout -b feat/minha-feature`
2. Fazer alterações e adicionar testes
3. Garantir que todos os testes passem: `cargo test --workspace`
4. Commitar com convenções: `feat: add new indicator`
5. Push e criar PR para `main`

### Licença

MIT
]]>
# Multi-Pane Layout Specification

## Purpose

Define the multi-pane vertical layout system. A main price pane occupies the largest area at top, with resizable indicator panes stacked below, sharing a synchronized x-axis.

## Requirements

### Requirement: Pane struct

A `Pane` MUST contain a `Viewport`, a list of visible series, and a list of indicator overlays. Each pane SHALL own its y-axis scale but share the x-axis time scale.

#### Scenario: Pane owns its viewport

- GIVEN a price pane and an RSI indicator pane
- WHEN each pane's viewport is queried
- THEN each has an independent price range but the same time range

#### Scenario: Pane renders own grid

- GIVEN a pane with visible grid enabled
- WHEN the pane renders
- THEN grid lines are drawn within its bounds

### Requirement: LayoutManager

A `LayoutManager` MUST manage an ordered list of vertically stacked panes. The top pane SHALL be the main price pane. Remaining panes SHALL be indicator panes.

#### Scenario: Layout order

- GIVEN a layout with 1 price pane and 2 indicator panes
- WHEN the layout is queried
- THEN panes are returned in order: price, indicator 1, indicator 2 (top to bottom)

#### Scenario: Empty layout

- GIVEN a `LayoutManager` with no panes
- WHEN queried for count
- THEN it SHALL return 0

### Requirement: Main price pane

The main price pane SHALL occupy the largest vertical space, at least 50% of total window height by default. It SHALL render the primary symbol's price series.

#### Scenario: Default height allocation

- GIVEN a 1080px window with 1 indicator pane
- WHEN layout is initialized
- THEN the main pane is at least 540px tall

#### Scenario: Main pane handles series visibility

- GIVEN a main pane with a candle and a line series
- WHEN the candle series visibility is toggled off
- THEN only the line series renders in the main pane

### Requirement: Indicator panes below

Indicator panes SHALL be positioned below the main pane. Each SHALL display one or more indicator output lines.

#### Scenario: Indicator pane renders RSI

- GIVEN an RSI indicator assigned to an indicator pane
- WHEN the pane renders
- THEN the RSI line is drawn within its bounds with correct 0–100 range

#### Scenario: Multiple indicators per pane

- GIVEN two indicators (RSI and MACD) assigned to the same pane
- WHEN the pane renders
- THEN both indicator lines are overlaid in the same area

### Requirement: Resizable dividers

Adjacent panes SHALL be separated by a draggable divider. Dragging the divider SHALL resize both adjacent panes proportionally.

#### Scenario: Divider drag resizes

- GIVEN 2 panes with a divider at the midpoint
- WHEN the divider is dragged 100px downward
- THEN the top pane grows by 100px and the bottom shrinks by 100px

#### Scenario: Divider respects minimum height

- GIVEN a bottom pane at minimum height (60px)
- WHEN the divider is dragged to shrink it below minimum
- THEN the divider SHALL stop at the minimum height boundary

### Requirement: Minimum pane heights

Each pane SHALL have a configurable minimum height. The default minimum SHALL be 60px.

#### Scenario: Minimum prevents collapse

- GIVEN a pane with min height 60px
- WHEN a resize operation would make it 30px
- THEN the pane height SHALL remain at 60px

#### Scenario: Configurable minimum

- GIVEN a pane with configurable min height set to 80px
- WHEN the layout is recomputed
- THEN the pane is never smaller than 80px

### Requirement: Shared x-axis

All panes SHALL share a single time scale (x-axis). Scrolling or zooming one pane SHALL synchronize all panes.

#### Scenario: Zoom syncs across panes

- GIVEN 3 panes showing the same time range 9:30–16:00
- WHEN the main pane zooms in to 10:00–14:00
- THEN all panes update to show 10:00–14:00

#### Scenario: Pan scroll syncs

- GIVEN synchronized panes
- WHEN the main pane pans right by 5 minutes
- THEN every indicator pane SHALL shift by the same time offset

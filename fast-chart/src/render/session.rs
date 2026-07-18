//! Trading session lines — vertical markers for market open/close.
//!
//! Sessions define when a market is open. Session lines render as vertical
//! lines on the chart at the configured times.

use crate::theme::Rgba;

/// A trading session definition.
///
/// # Examples
///
/// ```
/// use fast_chart::render::session::Session;
/// use fast_chart::theme::Rgba;
///
/// let session = Session::new("US PM", 13, 30, 20, 0);
/// assert_eq!(session.name, "US PM");
/// assert_eq!(session.duration_minutes(), 390);
///
/// assert!(session.contains_utc(15, 0));
/// assert!(!session.contains_utc(21, 0));
/// ```
#[derive(Debug, Clone)]
pub struct Session {
    /// Session name (e.g., "Regular", "Pre-Market", "After-Hours").
    pub name: String,
    /// Open time (hour, minute) in UTC.
    pub open_hour: u8,
    pub open_minute: u8,
    /// Close time (hour, minute) in UTC.
    pub close_hour: u8,
    pub close_minute: u8,
    /// Line color (can be overridden by theme).
    pub color: Option<Rgba>,
    /// Line style.
    pub line_style: SessionLineStyle,
    /// Line width in pixels.
    pub width: f64,
    /// Whether this session is active (rendered).
    pub active: bool,
}

/// Line style for session lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLineStyle {
    Solid,
    Dashed,
    Dotted,
}

impl Session {
    pub fn new(
        name: &str,
        open_hour: u8,
        open_minute: u8,
        close_hour: u8,
        close_minute: u8,
    ) -> Self {
        Self {
            name: name.to_owned(),
            open_hour,
            open_minute,
            close_hour,
            close_minute,
            color: None,
            line_style: SessionLineStyle::Dashed,
            width: 1.0,
            active: true,
        }
    }

    /// Duration in minutes.
    pub fn duration_minutes(&self) -> u32 {
        let open = self.open_hour as u32 * 60 + self.open_minute as u32;
        let close = self.close_hour as u32 * 60 + self.close_minute as u32;
        if close >= open {
            close - open
        } else {
            24 * 60 - open + close
        }
    }

    /// Check if a UTC time (hour, minute) is within this session.
    pub fn contains_utc(&self, hour: u8, minute: u8) -> bool {
        let t = hour as u32 * 60 + minute as u32;
        let open = self.open_hour as u32 * 60 + self.open_minute as u32;
        let close = self.close_hour as u32 * 60 + self.close_minute as u32;
        if open <= close {
            t >= open && t < close
        } else {
            t >= open || t < close
        }
    }
}

/// Pre-configured exchange sessions.
pub struct ExchangeSessions;

impl ExchangeSessions {
    /// US Regular Hours: 9:30 AM – 4:00 PM ET (14:30 – 21:00 UTC)
    pub fn us_regular() -> Session {
        Session::new("US Regular", 14, 30, 21, 0)
    }

    /// US Pre-Market: 4:00 AM – 9:30 AM ET (9:00 – 14:30 UTC)
    pub fn us_premarket() -> Session {
        Session::new("US Pre-Market", 9, 0, 14, 30)
    }

    /// US After-Hours: 4:00 PM – 8:00 PM ET (21:00 – 1:00 UTC)
    pub fn us_afterhours() -> Session {
        Session::new("US After-Hours", 21, 0, 1, 0)
    }

    /// London: 8:00 AM – 4:30 PM UTC
    pub fn london() -> Session {
        Session::new("London", 8, 0, 16, 30)
    }

    /// Tokyo: 9:00 AM – 3:00 PM JST (0:00 – 6:00 UTC)
    pub fn tokyo() -> Session {
        Session::new("Tokyo", 0, 0, 6, 0)
    }
}

/// Configuration for session line rendering.
#[derive(Debug, Clone)]
pub struct SessionLineConfig {
    /// Sessions to render.
    pub sessions: Vec<Session>,
    /// Default color when session has no explicit color.
    pub default_color: Rgba,
    /// Whether to show session labels.
    pub show_labels: bool,
}

impl Default for SessionLineConfig {
    fn default() -> Self {
        Self {
            sessions: vec![ExchangeSessions::us_regular()],
            default_color: Rgba::new(0.5, 0.5, 0.5, 0.3),
            show_labels: false,
        }
    }
}

/// A rendered session line (output of session rendering).
#[derive(Debug, Clone)]
pub struct SessionLine {
    /// X position in screen coordinates.
    pub x: f64,
    /// Top of the line (screen y).
    pub y_top: f64,
    /// Bottom of the line (screen y).
    pub y_bottom: f64,
    /// Line color.
    pub color: Rgba,
    /// Line style.
    pub line_style: SessionLineStyle,
    /// Line width.
    pub width: f64,
    /// Session name (for labels).
    pub session_name: String,
}

/// Renderer for session lines.
///
/// # Examples
///
/// ```
/// use fast_chart::render::session::{SessionLineConfig, SessionLineRenderer, Session};
/// use fast_chart::theme::Rgba;
///
/// let session = Session::new("Regular", 14, 30, 21, 0);
/// let config = SessionLineConfig {
///     sessions: vec![session],
///     show_labels: false,
///     ..Default::default()
/// };
/// let renderer = SessionLineRenderer::new(config);
///
/// let lines = renderer.render(0.0, 24.0, 0.0, 600.0, |h| h * 50.0);
/// assert_eq!(lines.len(), 2);
/// assert_eq!(lines[0].session_name, "Regular");
/// ```
pub struct SessionLineRenderer {
    config: SessionLineConfig,
}

impl SessionLineRenderer {
    pub fn new(config: SessionLineConfig) -> Self {
        Self { config }
    }

    /// Render session lines for a given time range.
    ///
    /// `visible_start_utc_hour` and `visible_end_utc_hour` define the
    /// visible time range in UTC hours (fractional).
    /// `y_top` and `y_bottom` define the vertical extent.
    /// `time_to_x` maps UTC hour to screen x coordinate.
    pub fn render<F>(
        &self,
        visible_start_utc_hour: f64,
        visible_end_utc_hour: f64,
        y_top: f64,
        y_bottom: f64,
        time_to_x: F,
    ) -> Vec<SessionLine>
    where
        F: Fn(f64) -> f64,
    {
        let mut lines = Vec::new();
        for session in &self.config.sessions {
            if !session.active {
                continue;
            }

            let color = session.color.unwrap_or(self.config.default_color);

            let open_hour =
                session.open_hour as f64 + session.open_minute as f64 / 60.0;
            if open_hour >= visible_start_utc_hour
                && open_hour <= visible_end_utc_hour
            {
                lines.push(SessionLine {
                    x: time_to_x(open_hour),
                    y_top,
                    y_bottom,
                    color,
                    line_style: session.line_style,
                    width: session.width,
                    session_name: session.name.clone(),
                });
            }

            let close_hour =
                session.close_hour as f64 + session.close_minute as f64 / 60.0;
            if close_hour >= visible_start_utc_hour
                && close_hour <= visible_end_utc_hour
            {
                lines.push(SessionLine {
                    x: time_to_x(close_hour),
                    y_top,
                    y_bottom,
                    color,
                    line_style: session.line_style,
                    width: session.width,
                    session_name: session.name.clone(),
                });
            }
        }
        lines
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_new() {
        let s = Session::new("Test", 10, 30, 16, 45);
        assert_eq!(s.name, "Test");
        assert_eq!(s.open_hour, 10);
        assert_eq!(s.open_minute, 30);
        assert_eq!(s.close_hour, 16);
        assert_eq!(s.close_minute, 45);
        assert_eq!(s.line_style, SessionLineStyle::Dashed);
        assert_eq!(s.width, 1.0);
        assert!(s.active);
        assert!(s.color.is_none());
    }

    #[test]
    fn session_duration_regular() {
        let s = ExchangeSessions::us_regular();
        assert_eq!(s.duration_minutes(), 390);
    }

    #[test]
    fn session_duration_overnight() {
        let s = ExchangeSessions::tokyo();
        assert_eq!(s.duration_minutes(), 360);
    }

    #[test]
    fn session_duration_afterhours() {
        let s = ExchangeSessions::us_afterhours();
        assert_eq!(s.duration_minutes(), 240);
    }

    #[test]
    fn session_contains_utc_inside() {
        let s = ExchangeSessions::us_regular();
        assert!(s.contains_utc(15, 0));
    }

    #[test]
    fn session_contains_utc_outside() {
        let s = ExchangeSessions::us_regular();
        assert!(!s.contains_utc(12, 0));
    }

    #[test]
    fn session_contains_utc_overnight() {
        let s = ExchangeSessions::us_afterhours();
        assert!(s.contains_utc(23, 0));
    }

    #[test]
    fn session_contains_utc_boundary_open() {
        let s = ExchangeSessions::us_regular();
        assert!(s.contains_utc(14, 30));
    }

    #[test]
    fn session_contains_utc_boundary_close() {
        let s = ExchangeSessions::us_regular();
        assert!(!s.contains_utc(21, 0));
    }

    #[test]
    fn us_regular_times() {
        let s = ExchangeSessions::us_regular();
        assert_eq!(s.open_hour, 14);
        assert_eq!(s.open_minute, 30);
        assert_eq!(s.close_hour, 21);
        assert_eq!(s.close_minute, 0);
    }

    #[test]
    fn us_premarket_times() {
        let s = ExchangeSessions::us_premarket();
        assert_eq!(s.open_hour, 9);
        assert_eq!(s.open_minute, 0);
        assert_eq!(s.close_hour, 14);
        assert_eq!(s.close_minute, 30);
    }

    #[test]
    fn london_times() {
        let s = ExchangeSessions::london();
        assert_eq!(s.open_hour, 8);
        assert_eq!(s.open_minute, 0);
        assert_eq!(s.close_hour, 16);
        assert_eq!(s.close_minute, 30);
    }

    #[test]
    fn tokyo_times() {
        let s = ExchangeSessions::tokyo();
        assert_eq!(s.open_hour, 0);
        assert_eq!(s.open_minute, 0);
        assert_eq!(s.close_hour, 6);
        assert_eq!(s.close_minute, 0);
    }

    #[test]
    fn session_line_renderer_produces_lines() {
        let config = SessionLineConfig {
            sessions: vec![ExchangeSessions::us_regular()],
            ..Default::default()
        };
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 24.0, 0.0, 500.0, |h| h * 10.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].x, 14.5 * 10.0);
        assert_eq!(lines[1].x, 21.0 * 10.0);
    }

    #[test]
    fn session_line_renderer_skips_inactive() {
        let mut session = ExchangeSessions::us_regular();
        session.active = false;
        let config = SessionLineConfig {
            sessions: vec![session],
            ..Default::default()
        };
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 24.0, 0.0, 500.0, |h| h);
        assert!(lines.is_empty());
    }

    #[test]
    fn session_line_renderer_uses_custom_color() {
        let mut session = ExchangeSessions::us_regular();
        session.color = Some(Rgba::new(1.0, 0.0, 0.0, 1.0));
        let config = SessionLineConfig {
            sessions: vec![session],
            ..Default::default()
        };
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 24.0, 0.0, 500.0, |h| h);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].color, Rgba::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(lines[1].color, Rgba::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn session_line_renderer_default_color() {
        let config = SessionLineConfig::default();
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 24.0, 0.0, 500.0, |h| h);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].color, Rgba::new(0.5, 0.5, 0.5, 0.3));
    }

    #[test]
    fn session_line_renderer_clips_to_visible() {
        let config = SessionLineConfig {
            sessions: vec![ExchangeSessions::us_regular()],
            ..Default::default()
        };
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 10.0, 0.0, 500.0, |h| h);
        assert!(lines.is_empty());
    }

    #[test]
    fn session_line_renderer_multiple_sessions() {
        let config = SessionLineConfig {
            sessions: vec![
                ExchangeSessions::us_regular(),
                ExchangeSessions::london(),
            ],
            ..Default::default()
        };
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 24.0, 0.0, 500.0, |h| h);
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn session_line_preserves_dimensions() {
        let mut session = ExchangeSessions::us_regular();
        session.line_style = SessionLineStyle::Dotted;
        session.width = 2.5;
        let config = SessionLineConfig {
            sessions: vec![session],
            ..Default::default()
        };
        let renderer = SessionLineRenderer::new(config);
        let lines = renderer.render(0.0, 24.0, 10.0, 490.0, |h| h);
        assert_eq!(lines[0].line_style, SessionLineStyle::Dotted);
        assert_eq!(lines[0].width, 2.5);
        assert_eq!(lines[0].y_top, 10.0);
        assert_eq!(lines[0].y_bottom, 490.0);
        assert_eq!(lines[0].session_name, "US Regular");
    }
}

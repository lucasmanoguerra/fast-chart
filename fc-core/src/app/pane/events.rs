/// Events emitted by pane interactions.
///
/// Pane events capture state changes caused by user interaction (divider
/// drags, resize, add/remove) so higher-level controllers can react
/// without polling.

// ---------------------------------------------------------------------------
// PaneEvent
// ---------------------------------------------------------------------------

/// An event describing a pane state change.
#[derive(Debug, Clone, PartialEq)]
pub enum PaneEvent {
    /// A divider between panes was dragged by `delta` pixels.
    DividerDragged {
        /// Index of the divider (0 = between first two panes).
        index: usize,
        /// Signed pixel delta (positive = lower pane grows).
        delta: f64,
    },
    /// A pane was resized to a new proportional height.
    PaneResized {
        /// Id of the resized pane.
        id: usize,
        /// New proportional height (0.0 – 1.0).
        new_height: f64,
    },
    /// A new pane was added to the layout.
    PaneAdded {
        /// Id of the newly added pane.
        id: usize,
    },
    /// A pane was removed from the layout.
    PaneRemoved {
        /// Id of the removed pane.
        id: usize,
    },
}

// ---------------------------------------------------------------------------
// PaneEventBus
// ---------------------------------------------------------------------------

/// Simple event collector and dispatcher for pane events.
///
/// Events are pushed during an interaction frame and drained by the
/// controller after the frame completes.
#[derive(Debug, Clone, Default)]
pub struct PaneEventBus {
    events: Vec<PaneEvent>,
}

impl PaneEventBus {
    /// Create an empty event bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an event into the bus.
    pub fn push(&mut self, event: PaneEvent) {
        self.events.push(event);
    }

    /// Drain all queued events, leaving the bus empty.
    pub fn drain(&mut self) -> Vec<PaneEvent> {
        std::mem::take(&mut self.events)
    }

    /// Returns the number of queued events without draining.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` when no events are queued.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_dragged() {
        let e = PaneEvent::DividerDragged {
            index: 0,
            delta: 25.5,
        };
        match e {
            PaneEvent::DividerDragged { index, delta } => {
                assert_eq!(index, 0);
                assert!((delta - 25.5).abs() < f64::EPSILON);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn pane_resized() {
        let e = PaneEvent::PaneResized {
            id: 2,
            new_height: 0.4,
        };
        match e {
            PaneEvent::PaneResized { id, new_height } => {
                assert_eq!(id, 2);
                assert!((new_height - 0.4).abs() < f64::EPSILON);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn pane_added_and_removed() {
        let a = PaneEvent::PaneAdded { id: 5 };
        let r = PaneEvent::PaneRemoved { id: 5 };
        assert_ne!(a, r);
    }

    // --- PaneEventBus ---

    #[test]
    fn bus_starts_empty() {
        let bus = PaneEventBus::new();
        assert!(bus.is_empty());
        assert_eq!(bus.len(), 0);
    }

    #[test]
    fn push_increments_len() {
        let mut bus = PaneEventBus::new();
        bus.push(PaneEvent::PaneAdded { id: 1 });
        bus.push(PaneEvent::PaneRemoved { id: 2 });
        assert_eq!(bus.len(), 2);
        assert!(!bus.is_empty());
    }

    #[test]
    fn drain_returns_all_and_clears() {
        let mut bus = PaneEventBus::new();
        bus.push(PaneEvent::DividerDragged {
            index: 0,
            delta: 10.0,
        });
        bus.push(PaneEvent::PaneResized {
            id: 1,
            new_height: 0.3,
        });

        let events = bus.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], PaneEvent::DividerDragged { index: 0, delta: 10.0 });
        assert_eq!(events[1], PaneEvent::PaneResized { id: 1, new_height: 0.3 });
        assert!(bus.is_empty());
    }

    #[test]
    fn drain_on_empty_bus() {
        let mut bus = PaneEventBus::new();
        let events = bus.drain();
        assert!(events.is_empty());
    }

    #[test]
    fn clone_and_debug() {
        let e = PaneEvent::PaneAdded { id: 3 };
        let e2 = e.clone();
        assert_eq!(e, e2);
        let _dbg = format!("{:?}", e);
    }
}

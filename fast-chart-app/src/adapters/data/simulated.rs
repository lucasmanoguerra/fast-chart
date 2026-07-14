use fast_chart_core::ports::data_provider::{DataProvider, DataEvent};
use fast_chart_core::Bar;
use std::error::Error;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// ---------------------------------------------------------------------------
// LCG — Linear Congruential Generator (Knuth multiplicative)
// ---------------------------------------------------------------------------

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Uniform float in [0, 1).
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    /// Box-Muller transform: standard normal N(0, 1).
    fn next_normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(f64::EPSILON);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------------------
// Drift-diffusion OHLC bar generator
// ---------------------------------------------------------------------------

/// Generate a single OHLC bar from a drift-diffusion model.
///
/// Invariants guaranteed (validated by `Bar::new`):
/// - `high >= max(open, close)` and `low <= min(open, close)`
/// - All prices non-negative (clamped to 0.01 minimum)
/// - `open == prev_close` (continuous pricing)
fn generate_bar(
    prev_close: f64,
    drift: f64,
    volatility: f64,
    lcg: &mut Lcg,
    timestamp: u64,
) -> Bar {
    let open = prev_close;

    // Drift-diffusion: close = open + drift + noise * volatility
    let noise = lcg.next_normal();
    let close = (open + drift + noise * volatility).max(0.01);

    // Wick extension beyond the body (high/low extend past open/close)
    let wick_up = lcg.next_f64().abs() * volatility * 0.3;
    let wick_down = lcg.next_f64().abs() * volatility * 0.3;
    let high = open.max(close) + wick_up;
    let low = (open.min(close) - wick_down).max(0.01);

    // Volume scales with volatility (higher vol = more trading activity)
    let vol_scale = (volatility / 250.0).clamp(0.1, 5.0);
    let volume = (1000.0 + lcg.next_f64() * 9000.0 * vol_scale) as u64;

    Bar::new(timestamp, open, high, low, close, volume).unwrap_or_else(|_| {
        // Fallback: flat bar at close — should never happen with correct invariants
        Bar::new(timestamp, close, close, close, close, 1000).unwrap()
    })
}

// ---------------------------------------------------------------------------
// SimulatedDataProvider
// ---------------------------------------------------------------------------

pub struct SimulatedDataProvider {
    running: Arc<Mutex<bool>>,
    sender: mpsc::Sender<DataEvent>,
    receiver: Option<mpsc::Receiver<DataEvent>>,
    symbol: String,
    base_price: f64,
    volatility: f64,
    drift: f64,
    seed: u64,
}

impl SimulatedDataProvider {
    /// Create a new provider with default seed (42) and slight upward drift.
    pub fn new(symbol: &str, base_price: f64, volatility: f64) -> Self {
        Self::with_seed(symbol, base_price, volatility, 42)
    }

    /// Create a new provider with an explicit seed for reproducible output.
    pub fn with_seed(symbol: &str, base_price: f64, volatility: f64, seed: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            running: Arc::new(Mutex::new(false)),
            sender,
            receiver: Some(receiver),
            symbol: symbol.to_string(),
            base_price,
            volatility,
            drift: 0.0001,
            seed,
        }
    }

}

impl DataProvider for SimulatedDataProvider {
    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let running = self.running.clone();
        let sender = self.sender.clone();
        let base_price = self.base_price;
        let volatility = self.volatility;
        let drift = self.drift;
        let seed = self.seed;

        thread::spawn(move || {
            let mut lcg = Lcg::new(seed);
            let mut prev_close = base_price;
            let mut timestamp: u64 = 1_700_000_000_000; // 2023-11-14 epoch millis

            while *running.lock().unwrap() {
                let bar = generate_bar(prev_close, drift, volatility, &mut lcg, timestamp);
                prev_close = bar.close;
                timestamp += 60_000; // 1-minute bars

                if sender.send(DataEvent::BarClosed(bar)).is_err() {
                    break; // receiver dropped
                }

                thread::sleep(std::time::Duration::from_millis(500));
            }
        });

        Ok(())
    }

    fn receiver(&self) -> Option<&mpsc::Receiver<DataEvent>> {
        self.receiver.as_ref()
    }

    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        *self.running.lock().unwrap() = false;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.symbol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_deterministic() {
        let mut a = Lcg::new(42);
        let mut b = Lcg::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn lcg_different_seeds_differ() {
        let mut a = Lcg::new(42);
        let mut b = Lcg::new(99);
        let mut same = 0u32;
        for _ in 0..1000 {
            if a.next_u64() == b.next_u64() {
                same += 1;
            }
        }
        // With good LCG, collisions should be extremely rare
        assert!(same < 5, "seeds should produce distinct sequences");
    }

    #[test]
    fn f64_in_range() {
        let mut lcg = Lcg::new(42);
        for _ in 0..10_000 {
            let v = lcg.next_f64();
            assert!(v >= 0.0 && v < 1.0, "f64 out of range: {v}");
        }
    }

    #[test]
    fn normal_distribution_mean_near_zero() {
        let mut lcg = Lcg::new(42);
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| lcg.next_normal()).sum();
        let mean = sum / n as f64;
        assert!(mean.abs() < 0.1, "mean too far from 0: {mean}");
    }

    #[test]
    fn generate_bar_ohlcv_invariants() {
        let mut lcg = Lcg::new(42);
        let mut prev = 100.0;
        for i in 0..1000 {
            let bar = generate_bar(prev, 0.0001, 1.0, &mut lcg, i * 60_000);
            assert!(bar.high >= bar.open.max(bar.close), "high < max(open,close)");
            assert!(bar.low <= bar.open.min(bar.close), "low > min(open,close)");
            assert!(bar.high >= bar.low, "high < low");
            assert!(bar.open >= bar.low && bar.open <= bar.high, "open out of range");
            assert!(bar.close >= bar.low && bar.close <= bar.high, "close out of range");
            assert!(bar.volume > 0, "zero volume");
            prev = bar.close;
        }
    }

    #[test]
    fn generate_bar_links_consecutive_closes() {
        let mut lcg = Lcg::new(42);
        let bar1 = generate_bar(100.0, 0.0001, 1.0, &mut lcg, 0);
        let bar2 = generate_bar(bar1.close, 0.0001, 1.0, &mut lcg, 60_000);
        assert_eq!(bar2.open, bar1.close);
    }

    #[test]
    fn generate_bar_deterministic_with_seed() {
        let make_bars = |seed: u64| {
            let mut lcg = Lcg::new(seed);
            let mut prev = 100.0;
            let mut bars = Vec::new();
            for i in 0..100 {
                let bar = generate_bar(prev, 0.0001, 1.0, &mut lcg, i * 60_000);
                prev = bar.close;
                bars.push(bar);
            }
            bars
        };

        let a = make_bars(42);
        let b = make_bars(42);
        assert_eq!(a.len(), b.len());
        for (i, (a_bar, b_bar)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(a_bar.open, b_bar.open, "bar {i} open differs");
            assert_eq!(a_bar.high, b_bar.high, "bar {i} high differs");
            assert_eq!(a_bar.low, b_bar.low, "bar {i} low differs");
            assert_eq!(a_bar.close, b_bar.close, "bar {i} close differs");
            assert_eq!(a_bar.volume, b_bar.volume, "bar {i} volume differs");
        }
    }

    #[test]
    fn simulate_high_vol_wider_range() {
        let make_avg_range = |vol: f64| -> f64 {
            let mut lcg = Lcg::new(42);
            let mut prev = 100.0;
            let mut total_range = 0.0;
            for i in 0..1000 {
                let bar = generate_bar(prev, 0.0001, vol, &mut lcg, i * 60_000);
                total_range += bar.high - bar.low;
                prev = bar.close;
            }
            total_range / 1000.0
        };

        let low_vol_range = make_avg_range(1.0);
        let high_vol_range = make_avg_range(10.0);
        assert!(
            high_vol_range > low_vol_range,
            "high vol ({high_vol_range}) should produce wider ranges than low vol ({low_vol_range})"
        );
    }

    #[test]
    fn provider_implements_data_provider_trait() {
        fn assert_data_provider<T: DataProvider>() {}
        assert_data_provider::<SimulatedDataProvider>();
    }

    #[test]
    fn provider_start_stop() {
        let mut provider = SimulatedDataProvider::new("TEST", 100.0, 1.0);
        assert!(provider.start().is_ok());
        // Starting again should be idempotent
        assert!(provider.start().is_ok());
        assert!(provider.stop().is_ok());
    }

    #[test]
    fn provider_receiver_available_after_construction() {
        let provider = SimulatedDataProvider::new("TEST", 100.0, 1.0);
        assert!(provider.receiver().is_some());
    }

    #[test]
    fn provider_name() {
        let provider = SimulatedDataProvider::new("BTC/USDT", 50000.0, 250.0);
        assert_eq!(provider.name(), "BTC/USDT");
    }

    #[test]
    fn provider_streams_bars() {
        let mut provider = SimulatedDataProvider::new("TEST", 100.0, 1.0);
        provider.start().unwrap();

        // Wait for a few bars to arrive
        let mut count = 0;
        let start = std::time::Instant::now();
        let rx = provider.receiver().unwrap();
        while count < 3 && start.elapsed() < std::time::Duration::from_secs(5) {
            if let Ok(DataEvent::BarClosed(_bar)) = rx.try_recv() {
                count += 1;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        provider.stop().unwrap();
        assert!(count >= 3, "expected at least 3 bars, got {count}");
    }
}

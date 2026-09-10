//! The side feeds an indicator may consume beyond the candle.
//!
//! Most of the registry is driven by the candle alone, but a substantial part of
//! it reads something else: a reference series (the pairwise family), a
//! derivatives tick, an order-book snapshot, the trades that printed within the
//! bar, or the market cross-section. An indicator whose feed is absent resolves,
//! ticks and returns nothing — every bar, without complaint.
//!
//! In an observation tensor that is worse than elsewhere: a missing value is
//! collapsed to `0.0`, because a tensor cannot carry `NaN` and an agent has to
//! be handed a number. A permanently unfed indicator is therefore a column of
//! constant zeros, indistinguishable from an indicator that is merely warming
//! up — and an agent trains on it as if it meant something.
//!
//! Two things close that. The bar already carries the microstructure the tensor
//! exposes as raw columns — order-book levels, funding, open interest — so
//! [`bar_feeds`] hands the same data to the indicators rather than only to the
//! observation. And [`ObsSpec::check_feeds`](crate::spec::ObsSpec::check_feeds)
//! refuses a spec whose indicator needs a feed the dataset cannot supply, so a
//! column that could only ever be zero is an error naming the indicator and the
//! feed instead of a silent one.

use wickra_backtest::core::registry::feed_of;
use wickra_backtest::core::spec::Feed;
use wickra_core::{DerivativesTick, Level, OrderBook};

use crate::spec::Candle;

/// Which side feed an indicator family consumes.
///
/// This mirrors `wickra_backtest::core::spec::Feed` with one addition: the
/// pairwise family. Upstream classifies pairwise indicators as `Kline` because
/// they are fed the bar close alongside the reference close, which leaves no way
/// to express "needs a reference series" — [`FeedKind::Pair`] is that missing
/// case, and [`feed_kind`] maps a name onto it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedKind {
    /// The candle alone.
    Candle,
    /// The candle plus a reference series' close (pairwise indicators).
    Pair,
    /// A derivatives tick — funding, open interest, mark and index.
    Derivatives,
    /// An order-book snapshot.
    OrderBook,
    /// The trades that printed within the bar.
    Trades,
    /// Trades quoted against the book mid (needs both feeds).
    TradeQuote,
    /// The market cross-section, for the breadth family.
    CrossSection,
}

impl FeedKind {
    /// The name used in error messages and documentation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FeedKind::Candle => "candle",
            FeedKind::Pair => "reference",
            FeedKind::Derivatives => "funding/open-interest",
            FeedKind::OrderBook => "orderbook",
            FeedKind::Trades => "trades",
            FeedKind::TradeQuote => "trades and orderbook",
            FeedKind::CrossSection => "cross-section",
        }
    }

    /// Why a dataset cannot carry this feed, for the ones no bar ever holds.
    #[must_use]
    pub fn why_absent(self) -> &'static str {
        match self {
            FeedKind::Pair => {
                "an environment steps one symbol, so there is no second series to read"
            }
            FeedKind::Trades | FeedKind::TradeQuote => {
                "a bar carries aggregate volume, not the individual prints"
            }
            FeedKind::CrossSection => {
                "an environment steps one symbol, so there is no market panel"
            }
            FeedKind::Derivatives => "no bar in this dataset carries funding or open interest",
            FeedKind::OrderBook => "no bar in this dataset carries order-book levels",
            FeedKind::Candle => "the candle is always present",
        }
    }
}

/// The pairwise indicators, which read a reference series' close alongside the
/// bar close.
///
/// This list exists because `registry::feed_of` cannot express the family:
/// upstream classifies every pairwise indicator as `Feed::Kline`, since the
/// candle is indeed one of its two inputs. The `every_pairwise_name_is_refused`
/// test probes every name here against the live registry, so a registry that
/// grows a new pairwise indicator cannot slip past silently.
const PAIRWISE: [&str; 24] = [
    "Alpha",
    "Beta",
    "BetaNeutralSpread",
    "Cointegration",
    "DistanceSsd",
    "GrangerCausality",
    "HasbrouckInformationShare",
    "InformationRatio",
    "KalmanHedgeRatio",
    "KendallTau",
    "LeadLagCrossCorrelation",
    "OuHalfLife",
    "PairSpreadZScore",
    "PairwiseBeta",
    "PearsonCorrelation",
    "RelativeStrengthAB",
    "RollingCorrelation",
    "RollingCovariance",
    "SpearmanCorrelation",
    "SpreadAr1Coefficient",
    "SpreadBollingerBands",
    "SpreadHurst",
    "TreynorRatio",
    "VarianceRatio",
];

/// Which feed an indicator consumes, or `None` if the registry does not know it.
#[must_use]
pub fn feed_kind(name: &str) -> Option<FeedKind> {
    let feed = feed_of(name)?;
    Some(match feed {
        Feed::Kline if PAIRWISE.contains(&name) => FeedKind::Pair,
        Feed::Kline => FeedKind::Candle,
        Feed::Trade => FeedKind::Trades,
        Feed::Orderbook => FeedKind::OrderBook,
        Feed::TradeQuote => FeedKind::TradeQuote,
        Feed::Derivatives => FeedKind::Derivatives,
        Feed::CrossSection => FeedKind::CrossSection,
    })
}

/// Which feed families a dataset can supply.
///
/// Derived from the bars themselves rather than declared: a dataset whose bars
/// carry order-book levels can drive the book family, and one whose bars carry
/// funding or open interest can drive the derivatives family. The other three
/// families are never available to an environment — see
/// [`FeedKind::why_absent`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Available {
    /// Every bar carries at least one order-book level on both sides.
    pub orderbook: bool,
    /// Every bar carries a funding rate or an open interest.
    pub derivatives: bool,
}

impl Available {
    /// What the bars of `candles` can supply.
    ///
    /// A feed counts as available only when **every** bar carries it: a feed
    /// present on some bars and absent on others would leave the indicator
    /// ticking on a hole, which is the silent case this type exists to prevent.
    #[must_use]
    pub fn of(candles: &[Candle]) -> Self {
        if candles.is_empty() {
            return Self::default();
        }
        Self {
            orderbook: candles
                .iter()
                .all(|c| !c.bid_px.is_empty() && !c.ask_px.is_empty()),
            derivatives: candles
                .iter()
                .all(|c| c.funding.is_some() || c.oi.is_some()),
        }
    }

    /// Whether a feed family is present.
    #[must_use]
    pub fn has(self, kind: FeedKind) -> bool {
        match kind {
            FeedKind::Candle => true,
            FeedKind::OrderBook => self.orderbook,
            FeedKind::Derivatives => self.derivatives,
            // An environment steps one symbol over OHLCV bars: there is no
            // second series, no individual prints and no market panel.
            FeedKind::Pair | FeedKind::Trades | FeedKind::TradeQuote | FeedKind::CrossSection => {
                false
            }
        }
    }
}

/// One bar's side feeds, derived from the bar itself.
///
/// The tensor already exposes these as raw observation columns; this is the same
/// data handed to the indicators, which is what makes the microstructure
/// families reachable at all.
#[derive(Debug, Default)]
pub struct BarFeeds {
    pub(crate) orderbook: Option<OrderBook>,
    pub(crate) deriv: Option<DerivativesTick>,
}

/// Derive the order book and derivatives tick a bar carries.
///
/// A malformed level or tick yields `None` for that feed rather than an error:
/// the invariants (`price > 0`, `size >= 0`, bids descending, asks ascending)
/// belong to the exchange that produced the data, and a single odd bar should
/// not end an episode. The spec check has already refused the case that matters
/// — an indicator whose feed the dataset never carries.
#[must_use]
pub fn bar_feeds(candle: &Candle) -> BarFeeds {
    BarFeeds {
        orderbook: book_of(candle),
        deriv: deriv_of(candle),
    }
}

/// The bar's order book, or `None` if it carries no levels.
fn book_of(candle: &Candle) -> Option<OrderBook> {
    if candle.bid_px.is_empty() || candle.ask_px.is_empty() {
        return None;
    }
    let side = |px: &[f64], sz: &[f64]| -> Option<Vec<Level>> {
        px.iter()
            .enumerate()
            .map(|(i, &price)| Level::new(price, sz.get(i).copied().unwrap_or(0.0)).ok())
            .collect()
    };
    let bids = side(&candle.bid_px, &candle.bid_sz)?;
    let asks = side(&candle.ask_px, &candle.ask_sz)?;
    OrderBook::new(bids, asks).ok()
}

/// The bar's derivatives tick, or `None` if it carries neither funding nor open
/// interest.
///
/// The fields a bar does not carry are filled from the bar itself: the mark,
/// index and futures price are the close, and the flow fields are zero. That is
/// the honest reading of an OHLCV bar with a funding print attached — not a
/// guess at data the dataset does not have.
fn deriv_of(candle: &Candle) -> Option<DerivativesTick> {
    if candle.funding.is_none() && candle.oi.is_none() {
        return None;
    }
    DerivativesTick::new(
        candle.funding.unwrap_or(0.0),
        candle.close,
        candle.close,
        candle.close,
        candle.oi.unwrap_or(0.0),
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        candle.ts,
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(close: f64) -> Candle {
        Candle {
            ts: 0,
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 10.0,
            bid_px: Vec::new(),
            bid_sz: Vec::new(),
            ask_px: Vec::new(),
            ask_sz: Vec::new(),
            funding: None,
            oi: None,
        }
    }

    fn book_bar(close: f64) -> Candle {
        Candle {
            bid_px: vec![close - 0.1, close - 0.2],
            bid_sz: vec![5.0, 8.0],
            ask_px: vec![close + 0.1, close + 0.2],
            ask_sz: vec![4.0, 7.0],
            ..bar(close)
        }
    }

    #[test]
    fn a_plain_ohlcv_bar_supplies_no_side_feed() {
        let feeds = bar_feeds(&bar(100.0));
        assert!(feeds.orderbook.is_none());
        assert!(feeds.deriv.is_none());
    }

    #[test]
    fn a_bar_with_levels_supplies_the_book() {
        let feeds = bar_feeds(&book_bar(100.0));
        let book = feeds.orderbook.expect("the bar carries levels");
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.asks.len(), 2);
    }

    #[test]
    fn a_bar_with_funding_supplies_the_derivatives_tick() {
        let candle = Candle {
            funding: Some(0.0001),
            oi: Some(1_000.0),
            ..bar(100.0)
        };
        let tick = bar_feeds(&candle).deriv.expect("the bar carries funding");
        assert!((tick.funding_rate - 0.0001).abs() < f64::EPSILON);
        assert!((tick.open_interest - 1_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn availability_needs_every_bar_to_carry_the_feed() {
        let mixed = [book_bar(100.0), bar(101.0)];
        assert!(
            !Available::of(&mixed).orderbook,
            "a feed on some bars only is not available"
        );
        let all = [book_bar(100.0), book_bar(101.0)];
        assert!(Available::of(&all).orderbook);
    }

    #[test]
    fn an_empty_dataset_supplies_nothing() {
        assert_eq!(Available::of(&[]), Available::default());
    }

    #[test]
    fn the_three_families_an_environment_cannot_have_are_never_available() {
        let all = Available {
            orderbook: true,
            derivatives: true,
        };
        for kind in [
            FeedKind::Pair,
            FeedKind::Trades,
            FeedKind::TradeQuote,
            FeedKind::CrossSection,
        ] {
            assert!(!all.has(kind), "{} must never be available", kind.as_str());
            assert!(!kind.why_absent().is_empty());
        }
        assert!(all.has(FeedKind::Candle));
        assert!(all.has(FeedKind::OrderBook));
        assert!(all.has(FeedKind::Derivatives));
    }

    #[test]
    fn feed_kind_classifies_the_families() {
        assert_eq!(feed_kind("Rsi"), Some(FeedKind::Candle));
        assert_eq!(feed_kind("Microprice"), Some(FeedKind::OrderBook));
        assert_eq!(feed_kind("FundingRate"), Some(FeedKind::Derivatives));
        assert_eq!(feed_kind("RollingCorrelation"), Some(FeedKind::Pair));
        assert_eq!(feed_kind("NotAnIndicator"), None);
    }
}

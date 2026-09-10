//! The side feeds, end to end.
//!
//! Before the feeds existed, `IndicatorSet::update` handed every indicator a
//! candle and nothing else. An indicator needing an order book or a funding
//! print therefore ticked and returned nothing on every bar — and the tensor
//! collapses a missing value to `0.0`, because it cannot carry `NaN` and an
//! agent has to be handed a number.
//!
//! So the column was constant zero for the whole episode, indistinguishable from
//! an indicator that is merely warming up, and an agent trained on it as if it
//! meant something. The bar already carried the order-book levels and the
//! funding rate — the tensor exposes them as raw observation columns — they were
//! simply never given to the indicators.
//!
//! These tests pin both halves: the feed a bar carries reaches the indicator,
//! and a feed no bar carries is refused by name rather than zeroed.

use gym_core::{build_tensor, Candle, Error, ObsSpec};

const BARS: usize = 80;

/// A plain OHLCV bar: no book, no funding, no open interest.
fn bar(i: usize) -> Candle {
    let t = i as f64;
    let close = 100.0 + (t * 0.3).sin() * 6.0 + t * 0.05;
    Candle {
        ts: 1_700_000_000 + i64::try_from(i).expect("bar index fits an i64") * 3600,
        open: close - 0.3,
        high: close + 0.9,
        low: close - 0.9,
        close,
        volume: 1_000.0 + t,
        bid_px: Vec::new(),
        bid_sz: Vec::new(),
        ask_px: Vec::new(),
        ask_sz: Vec::new(),
        funding: None,
        oi: None,
    }
}

/// The same bar with a two-level book around the close.
fn bar_with_book(i: usize) -> Candle {
    let base = bar(i);
    let close = base.close;
    Candle {
        bid_px: vec![close - 0.05, close - 0.10],
        bid_sz: vec![12.0, 20.0],
        ask_px: vec![close + 0.05, close + 0.10],
        ask_sz: vec![9.0, 18.0],
        ..base
    }
}

/// The same bar with a funding print and an open interest.
fn bar_with_funding(i: usize) -> Candle {
    let t = i as f64;
    Candle {
        funding: Some(0.0001 + (t * 0.1).sin() * 0.000_05),
        oi: Some(1_000_000.0 + t * 250.0),
        ..bar(i)
    }
}

fn dataset(make: fn(usize) -> Candle) -> Vec<Candle> {
    (0..BARS).map(make).collect()
}

/// One indicator column plus the close, so a dead column is visible.
fn spec_for(name: &str, params: &[f64]) -> ObsSpec {
    let json = serde_json::json!({
        "features": [
            { "kind": "indicator", "name": name, "params": params },
            { "kind": "price", "field": "close" }
        ]
    });
    serde_json::from_value(json).expect("parse obs spec")
}

/// The indicator column of every row, which is column 0 of each observation.
fn indicator_column(tensor: &gym_core::FeatureTensor) -> Vec<f64> {
    (0..tensor.n_bars).map(|bar| tensor.row(bar)[0]).collect()
}

fn assert_column_is_alive(name: &str, params: &[f64], candles: &[Candle]) {
    let spec = spec_for(name, params);
    let tensor =
        build_tensor(candles, &spec).unwrap_or_else(|e| panic!("{name} with its feed: {e}"));
    let column = indicator_column(&tensor);
    let nonzero = column.iter().filter(|v| **v != 0.0).count();
    assert!(
        nonzero > 0,
        "{name} produced a column of {} zeros even with its feed supplied",
        column.len()
    );
}

fn assert_refused(name: &str, params: &[f64], feed: &str) {
    let spec = spec_for(name, params);
    let Err(err) = build_tensor(&dataset(bar), &spec) else {
        panic!("{name} must be refused when no bar carries its {feed} feed");
    };
    match err {
        Error::MissingFeed {
            indicator,
            feed: got,
            why,
        } => {
            assert_eq!(indicator, name);
            assert_eq!(got, feed);
            assert!(!why.is_empty(), "the refusal says why the feed is absent");
        }
        other => panic!("{name}: expected MissingFeed, got {other}"),
    }
}

#[test]
fn an_order_book_indicator_reads_the_book_the_bar_carries() {
    assert_refused("Microprice", &[], "orderbook");
    assert_column_is_alive("Microprice", &[], &dataset(bar_with_book));
}

#[test]
fn a_derivatives_indicator_reads_the_funding_the_bar_carries() {
    assert_refused("FundingRate", &[], "funding/open-interest");
    assert_column_is_alive("FundingRate", &[], &dataset(bar_with_funding));
}

#[test]
fn a_candle_only_indicator_still_needs_nothing() {
    assert_column_is_alive("Rsi", &[14.0], &dataset(bar));
}

/// A feed on some bars and not others is not a feed: the indicator would tick on
/// a hole, which is the case the availability check exists to refuse.
#[test]
fn a_book_on_only_some_bars_is_not_available() {
    let mut candles = dataset(bar_with_book);
    candles[40] = bar(40);
    let spec = spec_for("Microprice", &[]);
    let Err(err) = build_tensor(&candles, &spec) else {
        panic!("a book on only some bars must be refused");
    };
    assert!(matches!(err, Error::MissingFeed { .. }));
}

/// The three families an environment can never supply are refused by name
/// rather than zeroed, and the refusal says why rather than only that.
#[test]
fn the_families_an_environment_cannot_have_are_refused_with_a_reason() {
    assert_refused("CumulativeVolumeDelta", &[], "trades");
    assert_refused("EffectiveSpread", &[], "trades and orderbook");
    assert_refused("AdvanceDecline", &[], "cross-section");
}

/// Every name treated as pairwise really is refused: an environment steps one
/// symbol, so there is no second series for it to read.
///
/// This is the half of the drift check that can be written today. The other
/// half — proving no *other* registry name needs a reference — would have to
/// enumerate the catalogue, and the registry exposes no such listing.
#[test]
fn every_pairwise_name_is_refused() {
    const PAIRWISE: [(&str, &[f64]); 24] = [
        ("Alpha", &[20.0, 0.0]),
        ("Beta", &[20.0]),
        ("BetaNeutralSpread", &[20.0]),
        ("Cointegration", &[30.0, 1.0]),
        ("DistanceSsd", &[20.0]),
        ("GrangerCausality", &[30.0, 2.0]),
        ("HasbrouckInformationShare", &[20.0]),
        ("InformationRatio", &[20.0]),
        ("KalmanHedgeRatio", &[0.0001, 0.001]),
        ("KendallTau", &[20.0]),
        ("LeadLagCrossCorrelation", &[20.0, 3.0]),
        ("OuHalfLife", &[30.0]),
        ("PairSpreadZScore", &[20.0, 20.0]),
        ("PairwiseBeta", &[20.0]),
        ("PearsonCorrelation", &[20.0]),
        ("RelativeStrengthAB", &[20.0, 14.0]),
        ("RollingCorrelation", &[20.0]),
        ("RollingCovariance", &[20.0]),
        ("SpearmanCorrelation", &[20.0]),
        ("SpreadAr1Coefficient", &[20.0]),
        ("SpreadBollingerBands", &[20.0, 2.0]),
        ("SpreadHurst", &[30.0]),
        ("TreynorRatio", &[20.0, 0.0]),
        ("VarianceRatio", &[20.0, 4.0]),
    ];

    for (name, params) in PAIRWISE {
        assert_refused(name, params, "reference");
    }
}

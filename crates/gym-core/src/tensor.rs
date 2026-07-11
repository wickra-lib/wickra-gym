//! The feature tensor: the whole episode dataset precomputed once, O(1) per bar,
//! into a fixed row-major `[n_bars * dim]` observation matrix (plus the close
//! series for the reward). Each `step()` is then a pure index into `data`.
//!
//! The precompute is a bar-sequential fold — indicators are stateful, so each
//! row depends on all prior bars. Reductions stay serial, so the tensor is
//! byte-identical whether or not the parallel feature is enabled (the parallel
//! path fans out across *datasets*, never across bars within one).

use crate::error::{Error, Result};
use crate::feature::{Feature, MicroField, PriceField};
use crate::indicator_set::{to_bt, IndicatorSet};
use crate::spec::{round8, Candle, ObsSpec};

/// A precomputed observation matrix for one episode dataset.
pub struct FeatureTensor {
    /// Number of bars (rows).
    pub n_bars: usize,
    /// Observation dimension (columns).
    pub dim: usize,
    /// Row-major observation data, length `n_bars * dim`.
    data: Vec<f64>,
    /// The close price per bar, kept separately for reward computation.
    pub closes: Vec<f64>,
}

impl FeatureTensor {
    /// The observation row for `bar` (length `dim`).
    #[must_use]
    pub fn row(&self, bar: usize) -> &[f64] {
        let start = bar * self.dim;
        &self.data[start..start + self.dim]
    }
}

/// A raw price/volume field read straight off the candle.
fn price_value(c: &Candle, field: PriceField) -> f64 {
    match field {
        PriceField::Open => c.open,
        PriceField::High => c.high,
        PriceField::Low => c.low,
        PriceField::Close => c.close,
        PriceField::Volume => c.volume,
    }
}

/// A microstructure field read from the candle's optional columns; absent
/// columns yield `0.0` (defined, never `NaN`).
fn micro_value(c: &Candle, field: MicroField, level: Option<u32>) -> f64 {
    let l = level.unwrap_or(0) as usize;
    match field {
        MicroField::BidPx => c.bid_px.get(l).copied().unwrap_or(0.0),
        MicroField::BidSz => c.bid_sz.get(l).copied().unwrap_or(0.0),
        MicroField::AskPx => c.ask_px.get(l).copied().unwrap_or(0.0),
        MicroField::AskSz => c.ask_sz.get(l).copied().unwrap_or(0.0),
        MicroField::Spread => match (c.ask_px.first(), c.bid_px.first()) {
            (Some(a), Some(b)) => a - b,
            _ => 0.0,
        },
        MicroField::Imbalance => match (c.bid_sz.first(), c.ask_sz.first()) {
            (Some(&b), Some(&a)) => {
                let denom = b + a;
                if denom.abs() > 0.0 {
                    (b - a) / denom
                } else {
                    0.0
                }
            }
            _ => 0.0,
        },
        MicroField::Funding => c.funding.unwrap_or(0.0),
        MicroField::Oi => c.oi.unwrap_or(0.0),
    }
}

/// Precompute the whole dataset into a [`FeatureTensor`]. Folds the indicator
/// set forward one bar at a time, fills each observation row in canonical order
/// (§6.5: features → book levels → funding/OI), rounds to `1e-8` and collapses
/// non-finite values to `0.0`.
pub fn build(candles: &[Candle], obs: &ObsSpec) -> Result<FeatureTensor> {
    if candles.is_empty() {
        return Err(Error::NoData);
    }
    let dim = obs.feature_dim();
    let n_bars = candles.len();
    let mut indicators = IndicatorSet::from_obs(obs)?;
    let mut data = Vec::with_capacity(n_bars * dim);
    let mut closes = Vec::with_capacity(n_bars);

    for c in candles {
        let bt = to_bt(c);
        let values = indicators.update(&bt);

        for f in &obs.features {
            let v = match f {
                Feature::Indicator { .. } => values.get(&f.key()).copied().unwrap_or(0.0),
                Feature::Price { field } => price_value(c, *field),
                Feature::Microstructure { field, level } => micro_value(c, *field, *level),
            };
            data.push(round8(v));
        }
        for level in 0..obs.include_book_levels as usize {
            data.push(round8(c.bid_px.get(level).copied().unwrap_or(0.0)));
            data.push(round8(c.bid_sz.get(level).copied().unwrap_or(0.0)));
            data.push(round8(c.ask_px.get(level).copied().unwrap_or(0.0)));
            data.push(round8(c.ask_sz.get(level).copied().unwrap_or(0.0)));
        }
        if obs.include_funding_oi {
            data.push(round8(c.funding.unwrap_or(0.0)));
            data.push(round8(c.oi.unwrap_or(0.0)));
        }
        closes.push(c.close);
    }
    indicators.validate_fields()?;

    Ok(FeatureTensor {
        n_bars,
        dim,
        data,
        closes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(ts: i64, close: f64) -> Candle {
        Candle {
            ts,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
            bid_px: vec![],
            bid_sz: vec![],
            ask_px: vec![],
            ask_sz: vec![],
            funding: None,
            oi: None,
        }
    }

    #[test]
    fn empty_dataset_is_no_data() {
        let obs = ObsSpec::default();
        assert!(matches!(build(&[], &obs), Err(Error::NoData)));
    }

    #[test]
    fn price_feature_tensor_is_exact() {
        let obs = ObsSpec {
            features: vec![Feature::Price {
                field: PriceField::Close,
            }],
            include_book_levels: 0,
            include_funding_oi: false,
        };
        let candles = vec![candle(1, 100.0), candle(2, 101.5), candle(3, 99.25)];
        let t = build(&candles, &obs).unwrap();
        assert_eq!(t.n_bars, 3);
        assert_eq!(t.dim, 1);
        assert_eq!(t.row(0), &[100.0]);
        assert_eq!(t.row(1), &[101.5]);
        assert_eq!(t.row(2), &[99.25]);
        assert_eq!(t.closes, vec![100.0, 101.5, 99.25]);
    }

    #[test]
    fn microstructure_columns_default_to_zero() {
        let obs = ObsSpec {
            features: vec![Feature::Microstructure {
                field: MicroField::Funding,
                level: None,
            }],
            include_book_levels: 1,
            include_funding_oi: true,
        };
        let t = build(&[candle(1, 100.0)], &obs).unwrap();
        // 1 feature + 4 book columns + 2 funding/oi = 7, all zero (no columns).
        assert_eq!(t.dim, 7);
        assert_eq!(t.row(0), &[0.0; 7]);
    }

    #[test]
    fn unknown_indicator_is_rejected() {
        let obs = ObsSpec {
            features: vec![Feature::Indicator {
                name: "definitely_not_an_indicator".into(),
                params: vec![],
                field: None,
            }],
            include_book_levels: 0,
            include_funding_oi: false,
        };
        assert!(matches!(
            build(&[candle(1, 1.0)], &obs),
            Err(Error::UnknownIndicator(_))
        ));
    }
}

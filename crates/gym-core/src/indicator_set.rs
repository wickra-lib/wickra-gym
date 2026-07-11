//! Resolving the observation's indicator features against the `wickra-backtest`
//! registry — reused verbatim so the name/parameter/field mapping is identical
//! to the backtest engine and screener. Nothing about indicator semantics is
//! re-implemented here; this module only drives the registry one bar at a time.

use std::collections::BTreeMap;

use wickra_backtest::core::data::Candle as BtCandle;
use wickra_backtest::core::registry::{self, BarInput, EvalIndicator};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::spec::{Candle, ObsSpec};

/// Convert a gym [`Candle`] into the backtest engine's candle for feeding the
/// registry. The registry wrappers validate OHLC internally and return `None`
/// while an indicator warms up, so no fallible conversion is needed here.
pub(crate) fn to_bt(c: &Candle) -> BtCandle {
    BtCandle {
        time: c.ts,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
        volume: c.volume,
    }
}

/// One resolved indicator column: its canonical key, the driven indicator, and
/// the optional multi-output sub-field it reads.
struct Resolved {
    key: String,
    indicator: Box<dyn EvalIndicator>,
    field: Option<String>,
    /// Whether this indicator ever exposed named output fields.
    saw_fields: bool,
    /// Whether the requested `field` ever appeared among them.
    field_found: bool,
}

/// The set of indicator features from an [`ObsSpec`], driven together bar by bar.
pub struct IndicatorSet {
    items: Vec<Resolved>,
}

impl IndicatorSet {
    /// Resolve every [`Feature::Indicator`] in `obs` against the registry. An
    /// unknown name is an [`Error::UnknownIndicator`]. Non-indicator features
    /// (price / microstructure) are read directly from the candle elsewhere.
    pub fn from_obs(obs: &ObsSpec) -> Result<Self> {
        let mut items = Vec::new();
        for f in &obs.features {
            if let Feature::Indicator {
                name,
                params,
                field,
            } = f
            {
                let indicator = registry::build(name, params)
                    .map_err(|_| Error::UnknownIndicator(name.clone()))?;
                items.push(Resolved {
                    key: f.key(),
                    indicator,
                    field: field.clone(),
                    saw_fields: false,
                    field_found: false,
                });
            }
        }
        Ok(Self { items })
    }

    /// Feed one bar and return the current value per indicator key. A warming-up
    /// indicator (or an absent field) contributes `0.0`.
    pub fn update(&mut self, candle: &BtCandle) -> BTreeMap<String, f64> {
        let input = BarInput {
            candle,
            reference: None,
            deriv: None,
            orderbook: None,
            trades: &[],
            cross_section: None,
        };
        let mut out = BTreeMap::new();
        for item in &mut self.items {
            let primary = item.indicator.update(&input);
            let value = match &item.field {
                None => primary.unwrap_or(0.0),
                Some(field) => {
                    let fields = item.indicator.fields();
                    if !fields.is_empty() {
                        item.saw_fields = true;
                    }
                    match fields.iter().find(|(name, _)| *name == field.as_str()) {
                        Some((_, v)) => {
                            item.field_found = true;
                            *v
                        }
                        None => 0.0,
                    }
                }
            };
            out.insert(item.key.clone(), value);
        }
        out
    }

    /// After a full pass, verify every requested sub-field actually exists: if an
    /// indicator exposed fields but never the requested one, the spec is wrong.
    pub fn validate_fields(&self) -> Result<()> {
        for item in &self.items {
            if let Some(field) = &item.field {
                if item.saw_fields && !item.field_found {
                    return Err(Error::BadSpec(format!(
                        "indicator '{}' has no output field '{field}'",
                        item.key
                    )));
                }
            }
        }
        Ok(())
    }
}

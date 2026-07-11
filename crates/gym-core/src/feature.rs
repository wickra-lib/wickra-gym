//! [`Feature`] — one scalar observation column (an indicator, a raw price field,
//! or a microstructure field). The JSON representation is the cross-language wire
//! contract (internally tagged, `snake_case`), and each feature has a canonical
//! [`Feature::key`] used for the observation layout and debugging.

use serde::{Deserialize, Serialize};

/// A single scalar observation column.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Feature {
    /// A `wickra-core` indicator resolved by name + params; `field` selects a
    /// sub-output for multi-output indicators (primary output when absent).
    Indicator {
        name: String,
        #[serde(default)]
        params: Vec<f64>,
        #[serde(default)]
        field: Option<String>,
    },
    /// A raw OHLCV price field.
    Price { field: PriceField },
    /// A microstructure field; `level` selects an orderbook level for the level
    /// fields (ignored for the aggregate fields).
    Microstructure {
        field: MicroField,
        #[serde(default)]
        level: Option<u32>,
    },
}

/// A raw price / volume field of a candle.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceField {
    Open,
    High,
    Low,
    Close,
    Volume,
}

impl PriceField {
    /// The canonical lowercase field name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            PriceField::Open => "open",
            PriceField::High => "high",
            PriceField::Low => "low",
            PriceField::Close => "close",
            PriceField::Volume => "volume",
        }
    }
}

/// A microstructure field. Level fields (`BidPx`/`BidSz`/`AskPx`/`AskSz`) take a
/// `level`; the aggregate fields (`Spread`/`Imbalance`/`Funding`/`Oi`) do not.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MicroField {
    BidPx,
    BidSz,
    AskPx,
    AskSz,
    Spread,
    Imbalance,
    Funding,
    Oi,
}

impl MicroField {
    /// The canonical lowercase field name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            MicroField::BidPx => "bid_px",
            MicroField::BidSz => "bid_sz",
            MicroField::AskPx => "ask_px",
            MicroField::AskSz => "ask_sz",
            MicroField::Spread => "spread",
            MicroField::Imbalance => "imbalance",
            MicroField::Funding => "funding",
            MicroField::Oi => "oi",
        }
    }

    /// Whether this field is addressed by an orderbook `level`.
    #[must_use]
    pub fn is_leveled(self) -> bool {
        matches!(
            self,
            MicroField::BidPx | MicroField::BidSz | MicroField::AskPx | MicroField::AskSz
        )
    }
}

/// Format one indicator parameter: whole numbers render without a decimal point
/// (`14`), non-integers via their shortest round-trip form (`0.5`).
fn fmt_param(p: f64) -> String {
    if p.is_finite() && p.fract() == 0.0 && p.abs() < 1e15 {
        format!("{}", p as i64)
    } else {
        format!("{p}")
    }
}

impl Feature {
    /// The canonical key for this feature — the stable name used in the
    /// observation layout and `feature_keys`. Examples: `rsi(14)`,
    /// `macd(12,26,9).hist`, `price.close`, `micro.bid_sz.0`, `micro.funding`.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Feature::Indicator {
                name,
                params,
                field,
            } => {
                let joined = params
                    .iter()
                    .map(|p| fmt_param(*p))
                    .collect::<Vec<_>>()
                    .join(",");
                let base = format!("{name}({joined})");
                match field {
                    Some(f) => format!("{base}.{f}"),
                    None => base,
                }
            }
            Feature::Price { field } => format!("price.{}", field.as_str()),
            Feature::Microstructure { field, level } => {
                if field.is_leveled() {
                    format!("micro.{}.{}", field.as_str(), level.unwrap_or(0))
                } else {
                    format!("micro.{}", field.as_str())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_key_formats_params_and_field() {
        let rsi = Feature::Indicator {
            name: "rsi".into(),
            params: vec![14.0],
            field: None,
        };
        assert_eq!(rsi.key(), "rsi(14)");

        let macd = Feature::Indicator {
            name: "macd".into(),
            params: vec![12.0, 26.0, 9.0],
            field: Some("hist".into()),
        };
        assert_eq!(macd.key(), "macd(12,26,9).hist");
    }

    #[test]
    fn price_and_micro_keys() {
        assert_eq!(
            Feature::Price {
                field: PriceField::Close
            }
            .key(),
            "price.close"
        );
        assert_eq!(
            Feature::Microstructure {
                field: MicroField::BidSz,
                level: Some(0)
            }
            .key(),
            "micro.bid_sz.0"
        );
        assert_eq!(
            Feature::Microstructure {
                field: MicroField::Funding,
                level: None
            }
            .key(),
            "micro.funding"
        );
    }

    #[test]
    fn feature_json_round_trips() {
        let json = r#"{"kind":"indicator","name":"rsi","params":[14.0]}"#;
        let f: Feature = serde_json::from_str(json).unwrap();
        assert_eq!(f.key(), "rsi(14)");
        let back: Feature = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
        assert_eq!(f, back);
    }
}

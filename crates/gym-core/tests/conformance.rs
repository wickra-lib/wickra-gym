//! Data-model conformance: serde round-trips of every wire enum, the canonical
//! `Feature::key` format, `feature_keys` ordering, and spec-validation errors.

use gym_core::{ActionSpace, EnvSpec, Error, Feature, MicroField, ObsSpec, PriceField, RewardKind};

fn round_trip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).unwrap();
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(value, &back, "round-trip mismatch for {json}");
}

#[test]
fn feature_variants_round_trip() {
    round_trip(&Feature::Indicator {
        name: "Rsi".into(),
        params: vec![14.0],
        field: None,
    });
    round_trip(&Feature::Indicator {
        name: "Macd".into(),
        params: vec![12.0, 26.0, 9.0],
        field: Some("hist".into()),
    });
    round_trip(&Feature::Price {
        field: PriceField::Close,
    });
    round_trip(&Feature::Microstructure {
        field: MicroField::Imbalance,
        level: None,
    });
    round_trip(&Feature::Microstructure {
        field: MicroField::BidSz,
        level: Some(0),
    });
}

#[test]
fn scalar_enums_round_trip() {
    for f in [
        PriceField::Open,
        PriceField::High,
        PriceField::Low,
        PriceField::Close,
        PriceField::Volume,
    ] {
        round_trip(&f);
    }
    for f in [
        MicroField::BidPx,
        MicroField::BidSz,
        MicroField::AskPx,
        MicroField::AskSz,
        MicroField::Spread,
        MicroField::Imbalance,
        MicroField::Funding,
        MicroField::Oi,
    ] {
        round_trip(&f);
    }
    for r in [RewardKind::Pnl, RewardKind::Sharpe, RewardKind::LogReturn] {
        round_trip(&r);
    }
    round_trip(&ActionSpace::Discrete { n: 3 });
    round_trip(&ActionSpace::Continuous {
        low: -1.0,
        high: 1.0,
    });
}

#[test]
fn feature_tags_are_snake_case() {
    assert_eq!(
        serde_json::to_string(&Feature::Price {
            field: PriceField::Close
        })
        .unwrap(),
        r#"{"kind":"price","field":"close"}"#
    );
    assert_eq!(
        serde_json::to_string(&ActionSpace::Discrete { n: 3 }).unwrap(),
        r#"{"type":"discrete","n":3}"#
    );
    assert_eq!(
        serde_json::to_string(&RewardKind::LogReturn).unwrap(),
        r#""log_return""#
    );
}

#[test]
fn feature_key_snapshots() {
    let key = |f: Feature| f.key();
    assert_eq!(
        key(Feature::Indicator {
            name: "Rsi".into(),
            params: vec![14.0],
            field: None
        }),
        "Rsi(14)"
    );
    assert_eq!(
        key(Feature::Indicator {
            name: "Macd".into(),
            params: vec![12.0, 26.0, 9.0],
            field: Some("hist".into())
        }),
        "Macd(12,26,9).hist"
    );
    assert_eq!(
        key(Feature::Price {
            field: PriceField::Volume
        }),
        "price.volume"
    );
    assert_eq!(
        key(Feature::Microstructure {
            field: MicroField::AskSz,
            level: Some(1)
        }),
        "micro.ask_sz.1"
    );
    assert_eq!(
        key(Feature::Microstructure {
            field: MicroField::Funding,
            level: None
        }),
        "micro.funding"
    );
}

#[test]
fn feature_keys_follow_canonical_order() {
    let obs = ObsSpec {
        features: vec![
            Feature::Price {
                field: PriceField::Close,
            },
            Feature::Indicator {
                name: "Sma".into(),
                params: vec![10.0],
                field: None,
            },
        ],
        include_book_levels: 1,
        include_funding_oi: true,
    };
    assert_eq!(obs.feature_dim(), 2 + 4 + 2);
    assert_eq!(
        obs.feature_keys(),
        vec![
            "price.close",
            "Sma(10)",
            "micro.bid_px.0",
            "micro.bid_sz.0",
            "micro.ask_px.0",
            "micro.ask_sz.0",
            "micro.funding",
            "micro.oi",
        ]
    );
}

const SPEC_HEAD: &str = r#"{"dataset_ref":"d","symbol":"S","observation":{"features":["#;
const SPEC_TAIL: &str =
    r#"]},"action_space":{"type":"discrete","n":3},"reward":"pnl","episode":{"max_steps":5}}"#;

#[test]
fn unknown_indicator_is_rejected() {
    let json = format!(
        r#"{SPEC_HEAD}{{"kind":"indicator","name":"NotAnIndicator","params":[]}}{SPEC_TAIL}"#
    );
    assert!(matches!(
        EnvSpec::from_json(&json),
        Err(Error::UnknownIndicator(_))
    ));
}

#[test]
fn zero_max_steps_is_rejected() {
    let json = r#"{"dataset_ref":"d","symbol":"S","observation":{"features":[]},"action_space":{"type":"discrete","n":3},"reward":"pnl","episode":{"max_steps":0}}"#;
    assert!(matches!(EnvSpec::from_json(json), Err(Error::BadSpec(_))));
}

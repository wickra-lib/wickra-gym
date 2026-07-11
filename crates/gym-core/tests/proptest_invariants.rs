//! Property-based invariants: over random specs, datasets and action sequences
//! the environment never panics, the observation length is constant and equal to
//! `feature_dim`, rewards and observations are always finite, `truncated` fires
//! no later than `max_steps`, and stepping after the episode is done yields an
//! error (never a panic).

use gym_core::{
    ActionSpace, Candle, Env, EnvSpec, EpisodeSpec, Feature, ObsSpec, PriceField, RewardKind,
};
use proptest::prelude::*;

fn price_field(i: u8) -> PriceField {
    match i % 5 {
        0 => PriceField::Open,
        1 => PriceField::High,
        2 => PriceField::Low,
        3 => PriceField::Close,
        _ => PriceField::Volume,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn env_holds_its_invariants(
        fields in prop::collection::vec(0u8..5, 1..4),
        book_levels in 0u32..3,
        funding_oi in any::<bool>(),
        n in 1u32..5,
        reward_idx in 0usize..3,
        max_steps in 1u32..10,
        warmup in 0u32..3,
        closes in prop::collection::vec(50.0f64..150.0, 8..30),
        actions in prop::collection::vec(0u32..8, 0..20),
    ) {
        let features: Vec<Feature> = fields
            .iter()
            .map(|&i| Feature::Price { field: price_field(i) })
            .collect();
        let observation = ObsSpec {
            features,
            include_book_levels: book_levels,
            include_funding_oi: funding_oi,
        };
        let dim = observation.feature_dim();
        let reward = [RewardKind::Pnl, RewardKind::Sharpe, RewardKind::LogReturn][reward_idx];
        let spec = EnvSpec {
            dataset_ref: "p".into(),
            symbol: "S".into(),
            observation,
            action_space: ActionSpace::Discrete { n },
            reward,
            episode: EpisodeSpec { max_steps, warmup },
            seed: Some(1),
        };

        let mut env = Env::new(&serde_json::to_string(&spec).unwrap()).unwrap();
        let candles: Vec<Candle> = closes
            .iter()
            .enumerate()
            .map(|(i, &c)| Candle {
                ts: i64::try_from(i).unwrap(),
                open: c,
                high: c + 0.5,
                low: c - 0.5,
                close: c,
                volume: 1000.0,
                bid_px: Vec::new(),
                bid_sz: Vec::new(),
                ask_px: Vec::new(),
                ask_sz: Vec::new(),
                funding: None,
                oi: None,
            })
            .collect();
        env.load(&candles).unwrap();

        let reset = env.reset(Some(1)).unwrap();
        prop_assert_eq!(reset.observation.len(), dim);
        prop_assert!(reset.observation.iter().all(|v| v.is_finite()));

        let mut done = false;
        let mut steps = 0u32;
        for &a in &actions {
            let action = f64::from(a % n); // always a valid discrete action in 0..n
            match env.step(action) {
                Ok(step) => {
                    prop_assert!(!done, "step succeeded after the episode was done");
                    prop_assert_eq!(step.observation.len(), dim);
                    prop_assert!(step.observation.iter().all(|v| v.is_finite()));
                    prop_assert!(step.reward.is_finite());
                    steps += 1;
                    prop_assert!(steps <= max_steps, "ran past max_steps");
                    if step.truncated {
                        prop_assert_eq!(steps, max_steps);
                    }
                    if step.terminated || step.truncated {
                        done = true;
                    }
                }
                Err(_) => {
                    prop_assert!(done, "step errored before the episode was done");
                }
            }
        }
    }
}

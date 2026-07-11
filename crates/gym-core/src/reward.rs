//! Step-reward computation (§6.6). All reductions are serial in step order, so
//! the f64 rounding is identical on every platform and binding.

use crate::spec::RewardKind;

/// Accumulates the per-step PnL series backing the [`RewardKind::Sharpe`] signal.
#[derive(Default)]
pub struct RewardState {
    rewards: Vec<f64>,
}

/// The running Sharpe ratio of the recorded reward series: `mean / std_pop`,
/// with a population standard deviation of zero mapping to `0.0`. Serial fold.
#[must_use]
pub fn running_sharpe(state: &RewardState) -> f64 {
    let n = state.rewards.len();
    if n == 0 {
        return 0.0;
    }
    let mean = state.rewards.iter().sum::<f64>() / n as f64;
    let var = state
        .rewards
        .iter()
        .map(|r| {
            let d = r - mean;
            d * d
        })
        .sum::<f64>()
        / n as f64;
    let std = var.sqrt();
    if std > 0.0 {
        mean / std
    } else {
        0.0
    }
}

/// The step reward for holding position `pos` over the next bar (§6.6):
/// - `Pnl`: `pos * (close_next - close_now)`.
/// - `LogReturn`: `pos * ln(close_next / close_now)`.
/// - `Sharpe`: push the PnL contribution and return the change in running Sharpe
///   (a dense, deterministic signal).
pub fn step_reward(
    kind: RewardKind,
    pos: f64,
    close_now: f64,
    close_next: f64,
    state: &mut RewardState,
) -> f64 {
    match kind {
        RewardKind::Pnl => pos * (close_next - close_now),
        RewardKind::LogReturn => pos * (close_next / close_now).ln(),
        RewardKind::Sharpe => {
            let pnl = pos * (close_next - close_now);
            let before = running_sharpe(state);
            state.rewards.push(pnl);
            let after = running_sharpe(state);
            after - before
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn pnl_is_position_times_delta() {
        let mut s = RewardState::default();
        assert!(approx(
            step_reward(RewardKind::Pnl, 1.0, 100.0, 102.0, &mut s),
            2.0
        ));
        assert!(approx(
            step_reward(RewardKind::Pnl, -0.5, 100.0, 102.0, &mut s),
            -1.0
        ));
    }

    #[test]
    fn log_return_is_position_times_log() {
        let mut s = RewardState::default();
        let r = step_reward(RewardKind::LogReturn, 1.0, 100.0, 110.0, &mut s);
        assert!((r - (110.0_f64 / 100.0).ln()).abs() < 1e-12);
    }

    #[test]
    fn sharpe_is_running_delta() {
        let mut s = RewardState::default();
        // First step: one sample, std_pop == 0 → running sharpe 0, delta 0.
        let r0 = step_reward(RewardKind::Sharpe, 1.0, 100.0, 101.0, &mut s);
        assert!(approx(r0, 0.0));
        assert!(approx(running_sharpe(&s), 0.0));
        // Second step with a different PnL makes the series non-degenerate.
        let _ = step_reward(RewardKind::Sharpe, 1.0, 101.0, 100.0, &mut s);
        assert!(running_sharpe(&s).is_finite());
    }
}

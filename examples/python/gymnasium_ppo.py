"""The main consumer: a real ``gymnasium.Env`` driven by a random agent.

    pip install wickra-gym[gym]
    python examples/python/gymnasium_ppo.py

Registers ``WickraGym-v0``, builds it with ``gymnasium.make`` and runs one
episode with a random policy sampled from the env's own action space. If
Stable-Baselines3 is installed, it then trains a tiny PPO for a handful of
timesteps to show the env satisfies the SB3 contract; otherwise that block is
skipped (SB3 is an optional, heavy dependency).
"""

import json
from pathlib import Path

import gymnasium as gym

from wickra_gym import register

DATA = Path(__file__).resolve().parent.parent / "data"


def make_env() -> gym.Env:
    register()
    spec = (DATA / "specs" / "momentum_discrete.json").read_text()
    candles = json.loads((DATA / "candles.json").read_text())
    return gym.make("WickraGym-v0", spec_json=spec, candles=candles)


def random_episode(env: gym.Env) -> float:
    obs, _info = env.reset(seed=42)
    print("observation space:", env.observation_space)
    print("action space:     ", env.action_space)
    total = 0.0
    while True:
        action = env.action_space.sample()
        obs, reward, terminated, truncated, _info = env.step(action)
        total += float(reward)
        if terminated or truncated:
            break
    return total


def maybe_train_ppo(env: gym.Env) -> None:
    try:
        from stable_baselines3 import PPO
    except ImportError:
        print("stable-baselines3 not installed; skipping the PPO step.")
        return
    model = PPO("MlpPolicy", env, n_steps=64, batch_size=32, verbose=0)
    model.learn(total_timesteps=128)
    print("PPO trained for 128 timesteps.")


def main() -> None:
    env = make_env()
    total = random_episode(env)
    print(f"random-policy episode return: {total:+.6f}")
    maybe_train_ppo(env)
    env.close()


if __name__ == "__main__":
    main()

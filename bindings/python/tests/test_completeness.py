"""Pin the public surface of the package."""

import wickra_gym


def test_public_symbols_present():
    for name in ("RawEnv", "WickraGymEnv", "register", "__version__"):
        assert hasattr(wickra_gym, name), f"missing public symbol: {name}"


def test_all_matches_exports():
    assert set(wickra_gym.__all__) == {
        "RawEnv",
        "WickraGymEnv",
        "register",
        "__version__",
    }


def test_rawenv_methods():
    for name in ("command", "version"):
        assert hasattr(wickra_gym.RawEnv, name)

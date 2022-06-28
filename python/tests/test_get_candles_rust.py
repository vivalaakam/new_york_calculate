import numpy as np

from new_york_calculate import get_candles_rust


def test_get_candles_rust():
    candles = get_candles_rust("XRPUSDT", 5, 1655769600, 12)

    results = list(map(lambda x: 1 if x.max_profit_12 > 1 else 0, candles))

    assert len(candles) == 288
    assert np.sum(results) == 208

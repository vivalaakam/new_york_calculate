import numpy as np
from new_york_calculate import get_candles


def test_get_candles():
    cache_val, intraday, results_val = get_candles(1, 1655769600, 5, 12)

    results = list(map(lambda x: 1 if x <= 12 * 60 * 60 else 0, results_val.values()))

    assert len(results) == 288
    assert np.sum(results) == 260

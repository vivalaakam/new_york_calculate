import json
import numpy as np

from new_york_calculate import prepare_candles, format_candles, CalculateRust


def test_prepare_candles():
    intraday_results = []

    with open('tests/candles.json', 'rb') as handle:
        intraday_results += json.load(handle)

    intraday_results = format_candles(intraday_results)

    cache_val, intraday, results_val = prepare_candles(intraday_results, 1, 1655769600, 5, 12)

    results = list(map(lambda x: 1 if x <= 12 * 60 * 60 else 0, results_val.values()))

    assert len(results) == 136
    assert np.sum(results) == 113


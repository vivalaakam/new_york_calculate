import pickle
import random

from new_york_calculate import calculate_actors, calculate, format_candles


def test_new_york_calculate_rust() -> None:
    intraday_results = []
    with open('tests/candles.pickle', 'rb') as handle:
        intraday_results += pickle.load(handle)

    intraday_results = format_candles(intraday_results)

    local_keys = list(
        map(lambda x: x[0], intraday_results[int(len(intraday_results) * 0.33): int(len(intraday_results) * 0.67)]))

    results = []

    for i in range(20):
        result = {}
        for i, lk in enumerate(local_keys):
            result[lk] = round(random.random())

        results.append(result)

    res = []

    for data in results:
        res.append(calculate(intraday_results, data))

    res2 = calculate_actors(intraday_results, results)

    for i in range(20):
        assert str(round(res[i]['wallet'], 8)) == str(round(res2[i]['wallet'], 8))

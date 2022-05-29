import pickle

from new_york_calculate import Calculate, calculate, format_candles


def test_new_york_calculate_rust() -> None:
    intraday_results = []
    with open('tests/candles.pickle', 'rb') as handle:
        intraday_results += pickle.load(handle)

    intraday_results = format_candles(intraday_results)

    local_keys = list(
        map(lambda x: x[0], intraday_results[int(len(intraday_results) * 0.33): int(len(intraday_results) * 0.67)]))

    results = {}

    for i, lk in enumerate(local_keys):
        results[lk] = i % 2

    res = calculate(intraday_results, results)

    submodule_class = Calculate(intraday_results)

    res2 = submodule_class.calculate(results)

    assert res['wallet'] == res2['wallet']

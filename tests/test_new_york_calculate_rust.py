import pickle

from new_york_calculate import CalculateRustV1, CalculateRustV2, format_candles


def test_new_york_calculate_rust() -> None:
    intraday_results = []
    with open('tests/candles.pickle', 'rb') as handle:
        intraday_results += pickle.load(handle)

    intraday_results = format_candles(intraday_results)

    local_keys = list(
        map(lambda x: x[0], intraday_results[0: int(len(intraday_results) * 0.67)]))

    results_dict = {}
    results_vec = []

    for i, lk in enumerate(local_keys):
        results_dict[lk] = i % 2
        results_vec.append(i % 2)

    calculate_rust_v1 = CalculateRustV1(intraday_results)
    calculate_rust_v2 = CalculateRustV2(intraday_results)

    calculate_rust_v1_result = calculate_rust_v1.calculate(results_dict)
    calculate_rust_v2_result = calculate_rust_v2.calculate(results_vec)

    assert calculate_rust_v1_result['wallet'] == calculate_rust_v2_result['wallet']

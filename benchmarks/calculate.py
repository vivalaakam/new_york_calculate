import pickle
import timeit

from new_york_calculate import format_candles, CalculateRustV1, CalculateRustV2

intraday_results = []
with open('tests/candles.pickle', 'rb') as handle:
    intraday_results += pickle.load(handle)

intraday_results = format_candles(intraday_results)

local_keys = list(
    map(lambda x: x[0], intraday_results[0: int(len(intraday_results) * 0.67)]))

results_dict = {}
results_list = []

for i, lk in enumerate(local_keys):
    results_dict[lk] = i % 2
    results_list.append(i % 2)

calculate_rust_v1 = CalculateRustV1(intraday_results)
calculate_rust_v2 = CalculateRustV2(intraday_results)

print(timeit.timeit("""
from new_york_calculate import calculate
calculate(intraday_results, results_dict)
""", globals=globals(), number=1000))

print(timeit.timeit("""
from new_york_calculate import CalculateRustV1
calculate_rust_v1 = CalculateRustV1(intraday_results)
calculate_rust_v1.calculate(results_dict)
""", globals=globals(), number=1000))

print(timeit.timeit("""
calculate_rust_v1.calculate(results_dict)
""", globals=globals(), number=1000))

print(timeit.timeit("""
from new_york_calculate import CalculateRustV2
calculate_rust_v2 = CalculateRustV2(intraday_results)
calculate_rust_v2.calculate(results_list)
""", globals=globals(), number=1000))

print(timeit.timeit("""
calculate_rust_v2.calculate(results_list)
""", globals=globals(), number=1000))

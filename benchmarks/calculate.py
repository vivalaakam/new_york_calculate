import pickle
import timeit

from new_york_calculate import format_candles, Calculate

intraday_results = []
with open('tests/candles.pickle', 'rb') as handle:
    intraday_results += pickle.load(handle)

intraday_results = format_candles(intraday_results)

local_keys = list(
    map(lambda x: x[0], intraday_results[int(len(intraday_results) * 0.33): int(len(intraday_results) * 0.67)]))

results = {}

for i, lk in enumerate(local_keys):
    results[lk] = i % 2

calculate_rust = Calculate(intraday_results)


print(timeit.timeit("""
from new_york_calculate import calculate
calculate(intraday_results, results)
""", globals=globals(), number=1000))

print(timeit.timeit("""
from new_york_calculate import Calculate
calculate_rust = Calculate(intraday_results)
calculate_rust.calculate(results)
""", globals=globals(), number=1000))

print(timeit.timeit("""
calculate_rust.calculate(results)
""", globals=globals(), number=1000))

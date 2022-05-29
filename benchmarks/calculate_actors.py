import pickle
import random
import timeit

from new_york_calculate import format_candles, Calculate

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

calculate_rust = Calculate(intraday_results)


print(timeit.timeit("""
from new_york_calculate import calculate
for actor in results:
    calculate(intraday_results, actor)
""", globals=globals(), number=1000))

print(timeit.timeit("""
from new_york_calculate import calculate_actors
calculate_actors(intraday_results, results)
""", globals=globals(), number=1000))

print(timeit.timeit("""
from new_york_calculate import Calculate
calculate_rust = Calculate(intraday_results)
for actor in results:
    calculate_rust.calculate(actor)
""", globals=globals(), number=1000))

print(timeit.timeit("""
for actor in results:
    calculate_rust.calculate(actor)
""", globals=globals(), number=1000))

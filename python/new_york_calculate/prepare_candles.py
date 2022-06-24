import math

import numpy as np


def prepare_candles(intraday, days, start, interval, look_back, target_ratio=1):
    target_ratio = 1 + (target_ratio / 100)

    candle_step = int(1440 / interval)

    cache_val = {}
    results_val = {}

    min_key = math.inf

    for i in range(len(intraday) - candle_step * 2):
        candle_key = intraday[candle_step + i][0]

        if start + 86400 <= candle_key < start + (days + 1) * 86400:
            prev = np.array(intraday[0 + i:candle_step + i])

            max_values = np.amax(prev, axis=0)
            min_values = np.amin(prev, axis=0)

            delta_1 = [min_values[3], max_values[2]]
            delta_2 = [min_values[5], max_values[5]]
            delta_3 = [min_values[7], max_values[7]]
            delta_4 = [min_values[8], max_values[8]]
            delta_5 = [min_values[9], max_values[9]]
            delta_6 = [min_values[10], max_values[10]]

            delta = np.array([
                delta_1[1] / delta_1[0] - 1,
                delta_2[1] / delta_2[0] - 1,
                delta_3[1] / delta_3[0] - 1,
                delta_4[1] / delta_4[0] - 1,
                delta_5[1] / delta_5[0] - 1,
                delta_6[1] / delta_6[0] - 1
            ])

            val = []

            for candle in prev:
                val.append(np.concatenate(
                    ([
                         np.interp(candle[1], delta_1, [0, 1]),
                         np.interp(candle[2], delta_1, [0, 1]),
                         np.interp(candle[3], delta_1, [0, 1]),
                         np.interp(candle[4], delta_1, [0, 1]),
                         np.interp(candle[5], delta_2, [0, 1]),
                         np.interp(candle[7], delta_3, [0, 1]),
                         np.interp(candle[8], delta_4, [0, 1]),
                         np.interp(candle[9], delta_5, [0, 1]),
                         np.interp(candle[10], delta_6, [0, 1])
                     ],
                     delta.copy())
                ))

            curr_ind = candle_step + i

            target = intraday[candle_step + i][1] * target_ratio

            success = None

            while success is None and curr_ind < len(intraday) and intraday[curr_ind][0] <= candle_key + 24 * 60 * 60:
                if intraday[curr_ind][2] > target:
                    success = intraday[curr_ind][0] - candle_key

                curr_ind += 1

            results_val[candle_key] = success if success is not None else 24 * 60 * 60

            cache_val[candle_key] = val[-look_back:]

            if candle_key < min_key:
                min_key = candle_key

    intraday = list(filter(lambda x: x[0] >= min_key, intraday))

    return cache_val, intraday, results_val

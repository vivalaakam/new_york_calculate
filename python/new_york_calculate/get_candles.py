import os
import pickle

import numpy as np
import requests

from .format_candles import format_candles
from .intervals import get_interval_key


def get_candles(days, start, interval, look_back):
    intraday_results = []

    for i in range(int(days) + 2):
        start = start + i * 86400

        fname = '{}.pickle'.format(start)

        if not os.path.exists(fname):
            r = requests.get(
                'https://api.binance.com/api/v3/klines?symbol=XRPUSDT&interval={}&startTime={}&endTime={}&limit=1000'.format(
                    get_interval_key(int(interval)), start * 1000, (start + 1439 * 60) * 1000))
            res = r.json()

            with open(fname, 'wb') as handle:
                pickle.dump(res, handle, protocol=pickle.HIGHEST_PROTOCOL)

        with open(fname, 'rb') as handle:
            intraday_results += pickle.load(handle)

    intraday = format_candles(intraday_results)

    candle_step = int(1440 / interval)

    cache_val = {}

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

            cache_val[candle_key] = val[-look_back:]

    min_key = min(cache_val.keys())

    intraday = list(filter(lambda x: x[0] >= min_key, intraday))

    return cache_val, intraday

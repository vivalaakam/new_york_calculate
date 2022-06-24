import os
import pickle

import requests

from .format_candles import format_candles
from .intervals import get_interval_key
from .prepare_candles import prepare_candles


def get_candles(days, start, interval, look_back, target_ratio=1):
    intraday_results = []

    for i in range(days + 2):
        start_stamp = start + i * 86400

        fname = '{}.pickle'.format(start_stamp)

        if not os.path.exists(fname):
            r = requests.get(
                'https://api.binance.com/api/v3/klines?symbol=XRPUSDT&interval={}&startTime={}&endTime={}&limit=1000'.format(
                    get_interval_key(int(interval)), start_stamp * 1000, (start_stamp + 1439 * 60) * 1000))
            res = r.json()

            with open(fname, 'wb') as handle:
                pickle.dump(res, handle, protocol=pickle.HIGHEST_PROTOCOL)

        with open(fname, 'rb') as handle:
            intraday_results += pickle.load(handle)

    intraday = format_candles(intraday_results)

    return prepare_candles(intraday, days, start, interval, look_back, target_ratio)

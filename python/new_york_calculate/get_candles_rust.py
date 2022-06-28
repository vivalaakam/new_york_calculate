from .new_york_calculate import get_candles


def get_candles_rust(ticker, period, start_time, look_back):
    return get_candles(ticker, period, start_time, look_back)

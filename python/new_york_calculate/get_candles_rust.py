import asyncio

from .new_york_calculate import get_candles


def get_candles_rust(ticker, period, start_time, look_back):
    return asyncio.get_event_loop().run_until_complete(get_candles_rust_async(ticker, period, start_time, look_back))


async def get_candles_rust_async(ticker, period, start_time, look_back):
    results = await get_candles(ticker, period, start_time, look_back)

    return results

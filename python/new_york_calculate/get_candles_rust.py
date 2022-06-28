import asyncio

from .new_york_calculate import get_candles


def get_candles_rust(ticker, period, start_time, look_back):
    loop = asyncio.get_event_loop()
    coroutine = get_candles_rust_async(ticker, period, start_time, look_back)
    result = loop.run_until_complete(coroutine)

    return result


async def get_candles_rust_async(ticker, period, start_time, look_back):
    results = await get_candles(ticker, period, start_time, look_back)

    return results

from new_york_calculate import get_candles_rust, CalculateRust, get_candles


def test_calculate():
    candles = get_candles_rust("XRPUSDT", 5, 1655769600, 12)

    calculate = CalculateRust(candles)

    results = list(map(lambda x: 1 if x.max_profit_12 > 1 else 0, candles))

    result = calculate.calculate(results)

    assert result['wallet'] == 8.26311035000005

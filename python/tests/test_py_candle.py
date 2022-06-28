from new_york_calculate import PyCandle


def test_py_candle():
    origin = [1655683200, 0.32620000, 0.32840000, 0.32520000, 0.32710000, 2130542.00000000, 1655683499,
              "696043.04930000", 1192, "1171785.00000000", "382943.15510000", "0"]

    candle = PyCandle((*origin[0:6],))

    assert candle.start_time == 1655683200

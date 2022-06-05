def format_candles(intraday_results):
    intraday = []
    for row in intraday_results:
        intraday.append(
            [int(row[0] / 1000), float(row[1]), float(row[2]), float(row[3]), float(row[4]), float(row[5]),
             float(row[6]),
             float(row[7]), float(row[8]), float(row[9]), float(row[10]), ])

    return intraday

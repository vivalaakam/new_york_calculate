from .to_nearest import ceil_to_nearest, floor_to_nearest


def calculate(candles, results, initial_balance=3000, stake=10, gain=1.0, wait=0, profit=0.5, interval=15):
    balance = initial_balance
    opened_orders = []
    executed_orders = []
    wallet = 0

    inner_gain = gain / 100 + 1

    step_lot = 1
    step_price = 0.0001

    min_balance = initial_balance

    for candle in candles:
        min_balance = min(min_balance, balance)
        if candle[0] in results and results[candle[0]] == 1 and balance > stake:
            curr_stake = floor_to_nearest(stake / candle[1], step_lot)

            order_sum = curr_stake * candle[1]

            balance -= order_sum

            # commission
            balance -= order_sum * 0.001

            opened_orders.append({
                'startTime': candle[0],
                'endTime': 0,
                'buyPrice': candle[1],
                'sellPrice': ceil_to_nearest(candle[1] * inner_gain, step_price),
                'qty': curr_stake,
                'commission': order_sum * 0.001,
            })

            # print("py order create: {} {} {} {}".format(candle[0], curr_stake, order_sum, balance))

        for order in reversed(opened_orders):
            if order['sellPrice'] < candle[2]:
                order_sum = order['sellPrice'] * order['qty']

                balance += order_sum
                balance -= order_sum * 0.001
                order['commission'] += order_sum * 0.001

                order['endTime'] = candle[0]

                profit_size = ((order['sellPrice'] - order['buyPrice']) * order['qty'] - order['commission']) * profit

                balance -= profit_size
                wallet += profit_size

                executed_orders.append(order)
                opened_orders.remove(order)

                # print("py order close: {} {} {}".format(candle[0], balance, wallet))

    base_count = 0
    base_sum = 0
    avg_wait = 0

    for order in opened_orders:
        base_count += order['qty']
        base_sum += order['qty'] * order['sellPrice']

    for order in executed_orders:
        avg_wait += (order['endTime'] - order['startTime']) + (interval * 60 - 1)

    total = len(executed_orders)

    return {
        'initial_balance': initial_balance,
        'min_balance': min_balance,
        'stake': stake,
        'profit': profit,
        'gain': gain,
        'wait': wait,
        'balance': balance,
        'wallet': wallet,
        'base_real': base_count * candles[-1][4],
        'base_expected': base_sum,
        'drawdown': (base_count * candles[-1][4]) / base_sum if len(opened_orders) > 0 else 1,
        'opened_orders': len(opened_orders),
        'executed_orders': len(executed_orders),
        'avg_wait': avg_wait / total if total > 0 else 0
    }

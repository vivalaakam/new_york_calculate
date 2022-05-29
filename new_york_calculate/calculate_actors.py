import numpy as np

from .to_nearest import ceil_to_nearest, floor_to_nearest


def calculate_actors(candles, actors, initial_balance=3000, stake=10, gain=1.0, wait=0, profit=0.5, interval=15):
    balance = [initial_balance for i in range(len(actors))]
    opened_orders = []
    executed_orders = []
    wallet = [0 for i in range(len(actors))]

    inner_gain = gain / 100 + 1

    step_lot = 1
    step_price = 0.0001

    min_balance = [initial_balance for i in range(len(actors))]

    for candle in candles:
        min_balance = np.minimum(min_balance, balance)

        has_orders = False

        for ind, actor in enumerate(actors):
            if candle[0] in actor and actor[candle[0]] == 1 and balance[ind] > stake:
                curr_stake = floor_to_nearest(stake / candle[1], step_lot)

                order_sum = curr_stake * candle[1]

                balance[ind] -= order_sum

                # commission
                balance[ind] -= order_sum * 0.001

                opened_orders.append({
                    'actor': ind,
                    'startTime': candle[0],
                    'endTime': 0,
                    'buyPrice': candle[1],
                    'sellPrice': ceil_to_nearest(candle[1] * inner_gain, step_price),
                    'qty': curr_stake,
                    'commission': order_sum * 0.001,
                })

                has_orders = True

        if has_orders:
            opened_orders = list(sorted(opened_orders, key=lambda d: d['sellPrice'], reverse=True))

        while len(opened_orders) > 0 and opened_orders[-1]['sellPrice'] < candle[2]:
            order = opened_orders.pop()
            order_sum = order['sellPrice'] * order['qty']

            balance[order['actor']] += order_sum
            balance[order['actor']] -= order_sum * 0.001
            order['commission'] += order_sum * 0.001

            order['endTime'] = candle[0]

            profit_size = ((order['sellPrice'] - order['buyPrice']) * order['qty'] - order['commission']) * profit

            balance[order['actor']] -= profit_size
            wallet[order['actor']] += profit_size

            executed_orders.append(order)

    base_count = [0 for i in range(len(actors))]
    base_sum = [0 for i in range(len(actors))]
    avg_wait = [0 for i in range(len(actors))]

    o_order = [0 for i in range(len(actors))]
    e_order = [0 for i in range(len(actors))]

    for order in opened_orders:
        base_count[order['actor']] += order['qty']
        base_sum[order['actor']] += order['qty'] * order['sellPrice']
        o_order[order['actor']] += 1

    for order in executed_orders:
        avg_wait[order['actor']] += (order['endTime'] - order['startTime']) + (interval * 60 - 1)
        e_order[order['actor']] += 1

    return [{
        'initial_balance': initial_balance,
        'min_balance': min_balance[actor],
        'stake': stake,
        'profit': profit,
        'gain': gain,
        'wait': wait,
        'balance': balance[actor],
        'wallet': wallet[actor],
        'base_real': base_count[actor] * candles[-1][4],
        'base_expected': base_sum[actor],
        'drawdown': (base_count[actor] * candles[-1][4]) / base_sum[actor] if o_order[actor] > 0 else 1,
        'opened_orders': o_order[actor],
        'executed_orders': e_order[actor],
        'avg_wait': avg_wait[actor] / e_order[actor] if e_order[actor] > 0 else 0
    } for actor in range(len(actors))]

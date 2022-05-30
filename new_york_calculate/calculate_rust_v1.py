from .new_york_calculate import PyCalculateV1


class CalculateRustV1:
    def __init__(self, intraday_results, initial_balance=3000, stake=10, gain=1.0, profit=0.5):
        self.instance = PyCalculateV1(candles=intraday_results, initial_balance=initial_balance, stake=stake, gain=gain,
                                      profit=profit)
        self.initial_balance = initial_balance
        self.stake = stake
        self.gain = gain
        self.profit = profit

    def calculate(self, results):
        result = self.instance.calculate(results)

        return {
            'initial_balance': self.initial_balance,
            'min_balance': result[4],
            'stake': self.stake,
            'profit': self.profit,
            'gain': self.gain,
            'balance': result[1],
            'wallet': result[0],
            'base_real': result[2],
            'base_expected': result[3],
            'drawdown': result[5],
            'opened_orders': result[6],
            'executed_orders': result[7],
            'avg_wait': result[8]
        }

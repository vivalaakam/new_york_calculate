from .new_york_calculate import PyCalculate, PyCandle


class CalculateRust:
    def __init__(self, intraday_results, initial_balance=3000, stake=10, gain=1.0, profit=0.5):
        candles = list(map(lambda candle: candle if isinstance(candle, PyCandle) else PyCandle(
            (*candle[0:6],)), intraday_results))

        self.instance = PyCalculate(candles=candles, initial_balance=initial_balance, stake=stake, gain=gain,
                                    profit=profit)
        self.initial_balance = initial_balance
        self.stake = stake
        self.gain = gain
        self.profit = profit

    def calculate(self, results):
        result = self.instance.calculate(list(results))

        return {
            'initial_balance': self.initial_balance,
            'min_balance': result.min_balance,
            'stake': self.stake,
            'profit': self.profit,
            'gain': self.gain,
            'balance': result.balance,
            'wallet': result.wallet,
            'base_real': result.base_real,
            'base_expected': result.base_expected,
            'drawdown': result.drawdown,
            'opened_orders': result.opened_orders,
            'executed_orders': result.executed_orders,
            'avg_wait': result.avg_wait,
            'score': result.score,
            'successful_ratio': result.successful_ratio,
        }

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use new_york_calculate_core::{
    Activate, Calculate, CalculateAgent, CalculateCommand, CalculateResultRef, CandleTrait, Order,
    Symbol, buy_market, sell_market,
};
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Mutex;
use std::time::Instant;

const SYMBOLS: [&str; 16] = [
    "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8", "S9", "S10", "S11", "S12", "S13", "S14",
    "S15",
];

#[derive(Clone, Debug)]
struct BenchCandle {
    start_time: u64,
    sym_idx: usize,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

impl CandleTrait for BenchCandle {
    fn get_start_time(&self) -> u64 {
        self.start_time
    }
    fn get_symbol(&self) -> &str {
        SYMBOLS[self.sym_idx]
    }
    fn get_open(&self) -> f32 {
        self.open
    }
    fn get_high(&self) -> f32 {
        self.high
    }
    fn get_low(&self) -> f32 {
        self.low
    }
    fn get_close(&self) -> f32 {
        self.close
    }
}

#[derive(Debug)]
struct BenchActivate {
    step: Mutex<u32>,
}

impl Activate<BenchCandle> for &BenchActivate {
    fn activate(
        &self,
        candles: &[BenchCandle],
        _prices: &HashMap<&str, f32>,
        _stats: CalculateResultRef<'_>,
        _active: &HashMap<Symbol, Vec<Order>>,
    ) -> Vec<CalculateCommand> {
        let mut step = self.step.lock().unwrap();
        *step += 1;
        let Some(_candle) = candles.last() else {
            return vec![];
        };
        match *step % 8 {
            0 => vec![buy_market!(candles[0].get_symbol(), 1.0)],
            4 => vec![sell_market!(candles[0].get_symbol(), 1.0)],
            _ => vec![CalculateCommand::None],
        }
    }
}

fn make_candles(steps: usize, symbols: usize) -> HashMap<u64, Vec<BenchCandle>> {
    let mut candles = HashMap::with_capacity(steps);
    for step in 0..steps {
        let ts = step as u64;
        let mut row = Vec::with_capacity(symbols);
        for sym in 0..symbols {
            let base = 100.0 + (step as f32 * 0.01) + sym as f32;
            row.push(BenchCandle {
                start_time: ts,
                sym_idx: sym,
                open: base,
                high: base * 1.05,
                low: base * 0.95,
                close: base,
            });
        }
        candles.insert(ts, row);
    }
    candles
}

fn run_calculate(steps: usize, symbols: usize, agents: usize) {
    let candles = make_candles(steps, symbols);
    let activate = BenchActivate {
        step: Mutex::new(0),
    };
    let mut agent_list = Vec::with_capacity(agents);
    for _ in 0..agents {
        agent_list.push(CalculateAgent::new(10_000.0, 0.0001, Box::new(&activate)));
    }
    let mut calc = Calculate::new(&candles, agent_list);
    while calc.next().is_some() {}
    calc.on_end();
    black_box(calc.get_pointer());
}

fn bench_calculate(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculate_full");
    for &(steps, symbols, agents) in &[
        (1_000, 1, 1),
        (1_000, 10, 1),
        (1_000, 1, 10),
        (10_000, 1, 1),
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("steps={steps} symbols={symbols} agents={agents}")),
            &(steps, symbols, agents),
            |b, &(steps, symbols, agents)| {
                b.iter(|| run_calculate(steps, symbols, agents));
            },
        );
    }
    group.finish();

    let mut hot = c.benchmark_group("calculate_hot_loop");
    for &(steps, symbols, agents) in &[(1_000, 1, 1), (1_000, 10, 1), (10_000, 1, 1)] {
        hot.bench_with_input(
            BenchmarkId::from_parameter(format!("steps={steps} symbols={symbols} agents={agents}")),
            &(steps, symbols, agents),
            |b, &(steps, symbols, agents)| {
                b.iter_custom(|iters| {
                    let mut total = std::time::Duration::ZERO;
                    for _ in 0..iters {
                        let candles = make_candles(steps, symbols);
                        let activate = BenchActivate {
                            step: Mutex::new(0),
                        };
                        let mut agent_list = Vec::with_capacity(agents);
                        for _ in 0..agents {
                            agent_list
                                .push(CalculateAgent::new(10_000.0, 0.0001, Box::new(&activate)));
                        }
                        let mut calc = Calculate::new(&candles, agent_list);
                        let start = Instant::now();
                        while calc.next().is_some() {}
                        total += start.elapsed();
                        black_box(calc.get_pointer());
                    }
                    total
                });
            },
        );
    }
    hot.finish();
}

criterion_group!(benches, bench_calculate);
criterion_main!(benches);

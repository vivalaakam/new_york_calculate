use crate::Symbol;

#[derive(Clone)]
pub enum CalculateCommand {
    Unknown,
    None,
    BuyProfit {
        symbol: Symbol,
        stake: f32,
        profit: f32,
    },
    BuyMarket {
        symbol: Symbol,
        stake: f32,
    },
    SellMarket {
        symbol: Symbol,
        stake: f32,
    },
    BuyLimit {
        symbol: Symbol,
        stake: f32,
        price: f32,
    },
    SellLimit {
        symbol: Symbol,
        stake: f32,
        price: f32,
    },
}

impl CalculateCommand {
    pub fn get_symbol(&self) -> Symbol {
        match self {
            CalculateCommand::BuyProfit { symbol, .. } => symbol.clone(),
            CalculateCommand::BuyMarket { symbol, .. } => symbol.clone(),
            CalculateCommand::SellMarket { symbol, .. } => symbol.clone(),
            CalculateCommand::BuyLimit { symbol, .. } => symbol.clone(),
            CalculateCommand::SellLimit { symbol, .. } => symbol.clone(),
            _ => Symbol::default(),
        }
    }
}

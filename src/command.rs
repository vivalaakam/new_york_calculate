use crate::types::{OrderId, Symbol, TimeStamp};

#[derive(Clone, Debug)]
pub enum CalculateCommand {
    Unknown,
    None,
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
        expiration: Option<TimeStamp>,
    },
    SellLimit {
        symbol: Symbol,
        stake: f32,
        price: f32,
        expiration: Option<TimeStamp>,
    },
    CancelLimit {
        symbol: Symbol,
        id: OrderId,
    },
}

impl CalculateCommand {
    pub fn get_symbol(&self) -> Symbol {
        match self {
            CalculateCommand::BuyMarket { symbol, .. } => symbol.clone(),
            CalculateCommand::SellMarket { symbol, .. } => symbol.clone(),
            CalculateCommand::BuyLimit { symbol, .. } => symbol.clone(),
            CalculateCommand::SellLimit { symbol, .. } => symbol.clone(),
            _ => Symbol::default(),
        }
    }
}

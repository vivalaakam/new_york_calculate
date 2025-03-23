use crate::Symbol;
use uuid::Uuid;

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
        expiration: Option<u64>,
    },
    SellLimit {
        symbol: Symbol,
        stake: f32,
        price: f32,
        expiration: Option<u64>,
    },
    CancelLimit {
        symbol: Symbol,
        id: Uuid,
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

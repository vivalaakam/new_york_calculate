use crate::types::{OrderId, Symbol, TimeStamp, UserId};

#[derive(Clone, Debug)]
pub enum CalculateCommand {
    Unknown,
    None,
    BuyMarket {
        symbol: Symbol,
        stake: f32,
        user_id: Option<UserId>,
    },
    SellMarket {
        symbol: Symbol,
        stake: f32,
        user_id: Option<UserId>,
    },
    BuyLimit {
        symbol: Symbol,
        stake: f32,
        price: f32,
        expiration: Option<TimeStamp>,
        user_id: Option<UserId>,
    },
    SellLimit {
        symbol: Symbol,
        stake: f32,
        price: f32,
        expiration: Option<TimeStamp>,
        user_id: Option<UserId>,
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
            CalculateCommand::CancelLimit { symbol, .. } => symbol.clone(),
            _ => Symbol::default(),
        }
    }
}

#[macro_export]
macro_rules! buy_market {
    ($symbol:expr, $stake:expr) => {
        CalculateCommand::BuyMarket {
            symbol: $symbol.clone(),
            stake: $stake,
            user_id: None,
        }
    };

    ($symbol:expr, $stake:expr, user_id = $user_id:expr) => {
        CalculateCommand::BuyMarket {
            symbol: $symbol.clone(),
            stake: $stake,
            user_id: Some($user_id.to_owned()),
        }
    };
}

#[macro_export]
macro_rules! sell_market {
    ($symbol:expr, $stake:expr) => {
        CalculateCommand::SellMarket {
            symbol: $symbol.clone(),
            stake: $stake,
            user_id: None,
        }
    };

    ($symbol:expr, $stake:expr, user_id = $user_id:expr) => {
        CalculateCommand::SellMarket {
            symbol: $symbol.clone(),
            stake: $stake,
            user_id: Some($user_id.to_owned()),
        }
    };
}

#[macro_export]
macro_rules! buy_limit {
    ($symbol:expr, $stake:expr, $price:expr) => {
        CalculateCommand::BuyLimit {
            symbol: $symbol,
            stake: $stake,
            price: $price,
            expiration: None,
            user_id: None,
        }
    };
    // expiration
    ($symbol:expr, $stake:expr, $price:expr, expiration = $expiration:expr) => {
        CalculateCommand::BuyLimit {
            symbol: $symbol,
            stake: $stake,
            price: $price,
            expiration: Some($expiration),
            user_id: None,
        }
    };
    // user_id
    ($symbol:expr, $stake:expr, $price:expr, user_id = $user_id:expr) => {
        CalculateCommand::BuyLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: None,
            user_id: Some($user_id.to_owned()),
        }
    };
    // expiration + user_id (в любом порядке)
    ($symbol:expr, $stake:expr, $price:expr, expiration = $expiration:expr, user_id = $user_id:expr) => {
        CalculateCommand::BuyLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: Some($expiration),
            user_id: Some($user_id.to_owned()),
        }
    };
    ($symbol:expr, $stake:expr, $price:expr, user_id = $user_id:expr, expiration = $expiration:expr) => {
        CalculateCommand::BuyLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: Some($expiration),
            user_id: Some($user_id.to_owned()),
        }
    };
}

#[macro_export]
macro_rules! sell_limit {
    ($symbol:expr, $stake:expr, $price:expr) => {
        CalculateCommand::SellLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: None,
            user_id: None,
        }
    };
    // expiration
    ($symbol:expr, $stake:expr, $price:expr, expiration = $expiration:expr) => {
        CalculateCommand::SellLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: Some($expiration),
            user_id: None,
        }
    };
    // user_id
    ($symbol:expr, $stake:expr, $price:expr, user_id = $user_id:expr) => {
        CalculateCommand::SellLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: None,
            user_id: Some($user_id.to_owned()),
        }
    };
    // expiration + user_id (в любом порядке)
    ($symbol:expr, $stake:expr, $price:expr, expiration = $expiration:expr, user_id = $user_id:expr) => {
        CalculateCommand::SellLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: Some($expiration),
            user_id: Some($user_id.to_owned()),
        }
    };
    ($symbol:expr, $stake:expr, $price:expr, user_id = $user_id:expr, expiration = $expiration:expr) => {
        CalculateCommand::SellLimit {
            symbol: $symbol.clone(),
            stake: $stake,
            price: $price,
            expiration: Some($expiration),
            user_id: Some($user_id.to_owned()),
        }
    };
}

#[macro_export]
macro_rules! cancel_limit {
    ($symbol:expr, $id:expr) => {
        CalculateCommand::CancelLimit {
            symbol: $symbol.clone(),
            id: $id,
        }
    };
}

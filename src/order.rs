use crate::types::{OrderId, Symbol, TimeStamp};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum OrderStatus {
    Open,
    Close,
    Cancel,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: Symbol,
    pub created_at: TimeStamp,
    pub finished_at: TimeStamp,
    pub price: f32,
    pub qty: f32,
    pub commission: f32,
    pub id: OrderId,
    pub status: OrderStatus,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub expiration: Option<TimeStamp>,
}

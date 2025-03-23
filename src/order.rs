use crate::symbol::Symbol;
use uuid::Uuid;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: Symbol,
    pub created_at: u64,
    pub finished_at: u64,
    pub price: f32,
    pub qty: f32,
    pub commission: f32,
    pub id: Uuid,
    pub status: OrderStatus,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub expiration: Option<u64>,
}

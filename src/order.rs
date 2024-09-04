use crate::symbol::Symbol;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Open,
    Close,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub uid: Uuid,
    pub symbol: Symbol,
    pub ts: u64,
    pub price: f32,
    pub qty: f32,
    pub commission: f32,
    pub agent: usize,
    pub id: Option<Uuid>,
    pub status: OrderStatus,
    pub side: OrderSide,
    pub order_type: OrderType,
}

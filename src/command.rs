#[derive(Clone)]
pub enum CalculateCommand {
    Unknown,
    None,
    BuyProfit { stake: f32, profit: f32 },
    BuyMarket { stake: f32 },
    SellMarket { stake: f32 },
    BuyLimit { stake: f32, price: f32 },
    SellLimit { stake: f32, price: f32 },
}

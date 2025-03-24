#[macro_export]
macro_rules! handle_sell_executed_order {
    ($self:expr, $order:expr, $candle:expr) => {{
        let mut order = $order.clone();

        $self.balance += order.price * order.qty;
        $self.balance -= order.commission;

        order.status = OrderStatus::Close;
        order.finished_at = $candle.get_start_time();

        debug!(
            balance = $self.balance,
            order = ?order,
            "sell order execution completed"
        );

        order
    }};
}

#[macro_export]
macro_rules! handle_buy_executed_order {
    ($self:expr, $order:expr, $candle:expr) => {{
        let mut order = $order.clone();

        $self.portfolio_stock
        .entry($candle.get_symbol())
        .and_modify(|v| *v += order.qty)
        .or_insert(order.qty);

        $self.balance -= order.commission;

        debug!(balance = $self.balance,  order = ?order, "buy order execution completed");

        order.status = OrderStatus::Close;
        order.finished_at = $candle.get_start_time();

        order
    }};
}

#[macro_export]
macro_rules! handle_cancel_order {
    ($self:expr, $order:expr, $candle:expr) => {{
        let mut order = $order.clone();

        match order.side {
            OrderSide::Buy => {
                $self.balance += order.price * order.qty;
            }
            OrderSide::Sell => {
                $self
                    .portfolio_stock
                    .entry($candle.get_symbol())
                    .and_modify(|v| *v += order.qty);
            }
        }
        order.status = OrderStatus::Cancel;
        order.finished_at = $candle.get_start_time();

        order
    }};
}

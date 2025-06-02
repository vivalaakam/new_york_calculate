#[macro_export]
macro_rules! handle_sell_executed_order {
    ($self:expr, $order:expr, $candle:expr) => {{
        let mut order = $order.clone();

        $self.balance += order.price * order.qty;
        $self.balance -= order.commission;

        $self.portfolio_frozen
                    .entry($candle.get_symbol())
                    .and_modify(|v| *v -= order.qty);

        order.status = OrderStatus::Close;
        order.finished_at = $candle.get_start_time();

        $self.activate.on_order($candle.get_start_time(), &order);

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

        $self.portfolio_available
        .entry($candle.get_symbol())
        .and_modify(|v| *v += order.qty)
        .or_insert(order.qty);

        $self.balance -= order.commission;

        debug!(balance = $self.balance,  order = ?order, "buy order execution completed");

        order.status = OrderStatus::Close;
        order.finished_at = $candle.get_start_time();

        $self.activate.on_order($candle.get_start_time(), &order);

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
                    .portfolio_available
                    .entry($candle.get_symbol())
                    .and_modify(|v| *v += order.qty);

                $self
                    .portfolio_frozen
                    .entry($candle.get_symbol())
                    .and_modify(|v| *v -= order.qty);
            }
        }
        order.status = OrderStatus::Cancel;
        order.finished_at = $candle.get_start_time();

        $self.activate.on_order($candle.get_start_time(), &order);

        order
    }};
}

#[macro_export]
macro_rules! truerange {
    ($high:expr, $low:expr, $close:expr) => {
        ($high - $low)
            .max(($high - $close).abs())
            .max(($low - $close).abs())
    };
}

#[macro_export]
macro_rules! hl2 {
    ($high:expr, $low:expr) => {
        ($high + $low) / 2.0
    };
}

#[macro_export]
macro_rules! hlc3 {
    ($high:expr, $low:expr, $close:expr) => {
        ($high + $low + $close) / 3.0
    };
}

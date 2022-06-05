pub fn floor_to_nearest(num: f64, base: f64) -> f64 {
    (num / base).floor() * base
}

pub fn ceil_to_nearest(num: f64, base: f64) -> f64 {
    (num / base).ceil() * base
}

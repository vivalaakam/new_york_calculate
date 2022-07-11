pub fn round_to_nearest(num: f64, base: f64) -> f64 {
    (num / base).round() * base
}

pub fn floor_to_nearest(num: f64, base: f64) -> f64 {
    (num / base).floor() * base
}

pub fn ceil_to_nearest(num: f64, base: f64) -> f64 {
    (num / base).ceil() * base
}

fn lerp(x: f64, y: f64, a: f64) -> f64 {
    x * (1f64 - a) + y * a
}

fn clamp(a: f64, min: f64, max: f64) -> f64 {
    a.min(max).max(min)
}

fn invlerp(x: f64, y: f64, a: f64) -> f64 {
    clamp((a - x) / (y - x), 0f64, 1f64)
}

pub fn range(x1: f64, y1: f64, x2: f64, y2: f64, a: f64) -> f64 {
    lerp(x2, y2, invlerp(x1, y1, a))
}

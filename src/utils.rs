pub fn floor_to_nearest(num: f32, base: f32) -> f32 {
    (num / base).floor() * base
}

pub fn ceil_to_nearest(num: f32, base: f32) -> f32 {
    (num / base).ceil() * base
}

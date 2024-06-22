/// truncating float thanks chatgpt
pub fn truncate(b: f32, precision: usize) -> f32 {
    let factor = 10f32.powi(precision as i32);
    (b * factor).ceil() / factor
}

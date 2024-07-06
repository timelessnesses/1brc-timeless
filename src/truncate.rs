/// truncating float thanks chatgpt
#[inline]
pub fn truncate(b: f64, precision: usize) -> f64 {
    let factor = 10f64.powi(precision as i32);
    (b * factor).ceil() / factor
}

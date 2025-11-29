use std::time::Duration;

/// the duration of a single interval based on the given rate.
pub const fn tick_dur(rate: f32) -> Duration {
    if !(rate.is_finite() && rate > 0.0) {
        return Duration::ZERO;
    }

    let secs_f = 1.0 / rate;
    let secs = secs_f as u64;
    let nanos = ((secs_f - secs as f32) * 1_000_000_000.0) as u32;

    Duration::new(secs, nanos)
}

/// debugging
pub fn bytes_to_mb_str(len: usize) -> String {
    let mb = len as f64 / 1_048_576.0; // 1024 * 1024
    format!("{:.2} MB", mb)
}

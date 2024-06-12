use crate::DEFAULT_INTERVAL;

pub fn get_interval() -> u64 {
    std::env::var("INTERVAL")
        .unwrap_or_else(|_| DEFAULT_INTERVAL.to_string())
        .parse()
        .expect("INTERVAL must be a number")
}

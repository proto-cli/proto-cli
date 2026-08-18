use std::sync::atomic::{AtomicBool, Ordering};

static QUIET: AtomicBool = AtomicBool::new(false);
static JSON: AtomicBool = AtomicBool::new(false);

pub fn set_quiet(v: bool) {
    QUIET.store(v, Ordering::Relaxed);
}
pub fn set_json(v: bool) {
    JSON.store(v, Ordering::Relaxed);
}
pub fn is_quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}
#[allow(dead_code)]
pub fn is_json() -> bool {
    JSON.load(Ordering::Relaxed)
}

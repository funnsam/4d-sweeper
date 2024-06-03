use core::sync::atomic::*;

static RAND_STATE: AtomicU64 = AtomicU64::new(1);

pub fn seed(s: u64) {
    RAND_STATE.store(s, Ordering::Relaxed)
}

pub fn rand() -> u64 {
    unsafe {
        let mut a = RAND_STATE.load(Ordering::Acquire);
        a ^= a << 13;
        a ^= a >> 7;
        a ^= a << 17;
        RAND_STATE.store(a, Ordering::Release);
        a
    }
}

static mut RAND_STATE: u64 = 0;

pub fn srand(s: u64) {
    unsafe {
        RAND_STATE = s
    }
}

pub fn rand() -> u64 {
    unsafe {
        let mut a = RAND_STATE;
        a ^= a << 13;
        a ^= a >> 7;
        a ^= a << 17;
        RAND_STATE = a;
        a
    }
}

static mut RAND_STATE: usize = 0;

pub fn srand(s: usize) {
    unsafe {
        RAND_STATE = s
    }
}

pub fn rand() -> usize {
    unsafe {
        let mut a = RAND_STATE;
        a ^= a << 13;
        a ^= a >> 7;
        a ^= a << 17;
        RAND_STATE = a;
        a
    }
}

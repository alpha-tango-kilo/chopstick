pub const EXTENSION_PREFIX: &str = "p";

pub const fn zero_pad_width(num: u64) -> usize {
    if num < 10 {
        1
    } else if num < 100 {
        2
    } else if num < 1000 {
        3
    } else {
        let mut count = 4;
        let mut current = num / 10u64.pow(4);
        while current > 0 {
            count += 1;
            current /= 10;
        }
        count
    }
}

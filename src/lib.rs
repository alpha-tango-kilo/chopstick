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

pub const fn round_up_div(a: u64, b: u64) -> u64 {
    a / b + (a % b != 0) as u64
}

#[cfg(test)]
mod unit_tests {
    use crate::round_up_div;

    #[test]
    fn round_up_division() {
        assert_eq!(round_up_div(1, 2), 1);
        assert_eq!(round_up_div(7, 2), 4);
        assert_eq!(round_up_div(10, 3), 4);
        assert_eq!(round_up_div(76, 2), 38);
        assert_eq!(round_up_div(16, 7), 3);
        assert_eq!(round_up_div(7, 3), 3);
        assert_eq!(round_up_div(10, 20), 1);
    }
}

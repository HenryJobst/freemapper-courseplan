//! Score codes carry the point value: the second digit of a two-digit code, the last digit of a
//! longer one. The tens digit tells controls of equal value apart.

use crate::check::codes::{rotated, six_nine};

/// The point value a code carries, or `None` for codes that are no numbers, have fewer than two
/// digits or end in 0.
pub fn value_of_code(code: &str) -> Option<u8> {
    if code.len() < 2 || !code.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let digit = code.chars().last()?.to_digit(10)? as u8;
    (digit != 0).then_some(digit)
}

/// The two-digit code for a tens digit (3 to 9) and a value (1 to 9).
pub fn code_for(tens: u8, value: u8) -> String {
    format!("{tens}{value}")
}

/// Whether two different codes read alike when turned by 180 degrees or differ only by 6 and 9.
/// Reuses the rule helpers of `check`.
pub fn reads_alike_turned(a: &str, b: &str) -> bool {
    a != b && (rotated(a).as_deref() == Some(b) || six_nine(a, b))
}

/// How many codes tens 3 to 9 and values 1 to `value_max` give.
pub fn code_space(value_max: u8) -> usize {
    7 * usize::from(value_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_of_code_follows_the_second_digit() {
        assert_eq!(value_of_code("31"), Some(1));
        assert_eq!(value_of_code("33"), Some(3));
        assert_eq!(value_of_code("49"), Some(9));
        assert_eq!(value_of_code("55"), Some(5));
        assert_eq!(value_of_code("105"), Some(5));
        assert_eq!(value_of_code("30"), None);
        assert_eq!(value_of_code("A1"), None);
        assert_eq!(value_of_code("5"), None);
        assert_eq!(value_of_code(""), None);
    }

    #[test]
    fn code_for_builds_two_digit_codes() {
        assert_eq!(code_for(3, 1), "31");
        assert_eq!(code_for(4, 9), "49");
        assert_eq!(code_for(9, 5), "95");
    }

    #[test]
    fn turned_pairs_are_found() {
        assert!(reads_alike_turned("69", "96"));
        assert!(reads_alike_turned("68", "89"));
        assert!(!reads_alike_turned("31", "13"));
        assert!(!reads_alike_turned("31", "31"));
    }

    #[test]
    fn code_space_is_35_and_63() {
        assert_eq!(code_space(5), 35);
        assert_eq!(code_space(9), 63);
    }
}

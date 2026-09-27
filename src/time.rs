use num_rational::Rational64;
use serde::{Deserialize, Serialize};

pub type RationalTime = Rational64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Timecode {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub frames: u32,
}

impl Timecode {
    pub fn from_frames(total_frames: i64, fps: Rational64) -> Self {
        let fps_f64 = *fps.numer() as f64 / *fps.denom() as f64;
        let fps_round = fps_f64.round().max(1.0) as i64;
        let non_neg = total_frames.max(0);

        let frames = (non_neg % fps_round) as u32;
        let total_secs = non_neg / fps_round;
        let seconds = (total_secs % 60) as u32;
        let total_mins = total_secs / 60;
        let minutes = (total_mins % 60) as u32;
        let hours = (total_mins / 60) as u32;

        Self {
            hours,
            minutes,
            seconds,
            frames,
        }
    }

    pub fn to_string_formatted(&self) -> String {
        format!(
            "{:02}:{:02}:{:02};{:02}",
            self.hours, self.minutes, self.seconds, self.frames
        )
    }
}

/// Computes the number of base-10 digits in a non-negative 64-bit integer,
/// matching Olive's `GetDigitCount`.
pub fn get_digit_count(n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut cur = n;
    while cur > 0 {
        count += 1;
        cur /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_count_parity() {
        assert_eq!(get_digit_count(1), 1);
        assert_eq!(get_digit_count(69), 2);
        assert_eq!(get_digit_count(420), 3);
        assert_eq!(get_digit_count(1337), 4);
        assert_eq!(get_digit_count(80085), 5);
        assert_eq!(get_digit_count(555555), 6);
        assert_eq!(get_digit_count(8675309), 7);
        assert_eq!(get_digit_count(78956423), 8);
        assert_eq!(get_digit_count(148497523), 9);
        assert_eq!(get_digit_count(4845821233), 10);
        assert_eq!(get_digit_count(18002738255), 11);
        assert_eq!(get_digit_count(180027382556), 12);
        assert_eq!(get_digit_count(1800273825568), 13);
        assert_eq!(get_digit_count(18002738255685), 14);
        assert_eq!(get_digit_count(180027382556857), 15);
        assert_eq!(get_digit_count(1800273825564857), 16);
    }
}

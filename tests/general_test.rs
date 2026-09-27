use num_rational::Rational64;
use olive_rs::time::{get_digit_count, Timecode};

#[test]
fn test_digit_count() {
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

#[test]
fn test_timecode() {
    let tc = Timecode::from_frames(3600 * 30 + 15, Rational64::new(30, 1));
    assert_eq!(tc.hours, 1);
    assert_eq!(tc.minutes, 0);
    assert_eq!(tc.seconds, 0);
    assert_eq!(tc.frames, 15);
    assert_eq!(tc.to_string_formatted(), "01:00:00;15");
}

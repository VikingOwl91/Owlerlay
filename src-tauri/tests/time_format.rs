use owlerlay_lib::overlay::model::TimeFormat;

fn f(secs: u64, fmt: TimeFormat) -> String {
    fmt.format(secs * 1_000)
}

#[test]
fn auto_strips_leading_zero_groups() {
    assert_eq!(f(9, TimeFormat::Auto), "09");
    assert_eq!(f(65, TimeFormat::Auto), "01:05");
    assert_eq!(f(3_903, TimeFormat::Auto), "01:05:03");
    assert_eq!(f(183_950, TimeFormat::Auto), "02:03:05:50");
    assert_eq!(f(0, TimeFormat::Auto), "00");
}

#[test]
fn fixed_modes_let_the_leading_unit_overflow() {
    assert_eq!(f(3_903, TimeFormat::S), "3903");
    assert_eq!(f(7_200, TimeFormat::Ms), "120:00");
    assert_eq!(f(180_000, TimeFormat::Hms), "50:00:00");
    assert_eq!(f(183_950, TimeFormat::Dhms), "02:03:05:50");
}

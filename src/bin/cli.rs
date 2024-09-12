use std::io::{stdin, stdout};
use transform::transform;

use chrono::{Datelike, TimeDelta, TimeZone};
use rrule::Tz;

fn main() {
    let keep_after = chrono::Utc::now().date_naive() - TimeDelta::days(30);
    let keep_after = Tz::UTC
        .with_ymd_and_hms(
            keep_after.year(),
            keep_after.month(),
            keep_after.day(),
            0,
            0,
            0,
        )
        .unwrap();

    transform(stdin().lock(), stdout(), keep_after, true);
}

use std::{
    borrow::BorrowMut,
    io::{BufRead, Write},
};

use chrono::{DateTime, NaiveDateTime};
use rrule::{RRuleSet, Tz};

pub fn transform<R: BufRead, W: Write>(
    mut input: R,
    mut output: W,
    keep_after: DateTime<Tz>,
    should_remove_exdates: bool,
) {
    let mut line_buf = String::with_capacity(1024);
    let mut event_buf = String::with_capacity(2024);

    loop {
        line_buf.clear();
        match input.read_line(line_buf.borrow_mut()) {
            Ok(len) => {
                if len == 0 {
                    // eprintln!("file end");
                    break;
                }
                if line_buf.starts_with("BEGIN:VEVENT") {
                    event_buf.push_str(line_buf.as_str());
                    loop {
                        line_buf.clear();
                        match input.read_line(line_buf.borrow_mut()) {
                            Ok(len) => {
                                if len == 0 {
                                    eprintln!("file end inside event");
                                    break;
                                }
                                event_buf.push_str(line_buf.as_str());
                                if line_buf.starts_with("END:VEVENT") {
                                    if should_keep_event(event_buf.as_str(), keep_after) {
                                        let event_buf = if should_remove_exdates {
                                            remove_exdates(event_buf.as_str())
                                        } else {
                                            event_buf.clone()
                                        };

                                        output
                                            .write(replace_dtstart(event_buf.as_str()).as_bytes())
                                            .unwrap();
                                    }
                                    event_buf.clear();
                                    break;
                                }
                            }
                            Err(_) => {
                                eprintln!("read error inside event");
                                break;
                            }
                        }
                    }
                } else {
                    output.write(line_buf.as_bytes()).unwrap();
                }
            }
            Err(_) => break,
        }
    }
}

fn should_keep_event(event: &str, keep_after: DateTime<Tz>) -> bool {
    let keep_after_str = keep_after.format("%Y%m%d").to_string();

    if !event.contains("RRULE:") {
        let lines = event
            .split("\r\n")
            .filter(|line| line.starts_with("DTSTART"))
            .collect::<Vec<&str>>();
        let dtstart = lines.first().expect("DTSTART missing in event");
        let (_, date) = dtstart.split_once(":").expect("DTSTART contains no colon");
        date >= keep_after_str.as_str()
    } else {
        let rr_event = event
            .split("\r\n")
            .filter(|line| {
                line.starts_with("DTSTART")
                    || line.starts_with("RRULE")
                    || line.starts_with("EXDATE")
            })
            .collect::<Vec<&str>>()
            .join("\n");
        match rr_event.parse::<RRuleSet>() {
            Ok(rrule_set) => {
                let rrule = rrule_set.after(keep_after);
                let result = rrule.all(1);
                if result.dates.is_empty() {
                    false
                } else {
                    true
                }
            }
            Err(e) => {
                eprintln!("Could not parse event: {e:?} {rr_event:?}");
                false
            }
        }
    }
}

fn replace_dtstart(event_buf: &str) -> String {
    event_buf
        .split("\n")
        .map(|line| {
            if line.starts_with("DTSTART;TZID")
                || line.starts_with("DTEND;TZID")
                || line.starts_with("EXDATE;TZID")
            {
                let (key, val) = line.trim_end().split_once(";").unwrap();
                let (tz_string, date_string) = val.split_once(":").unwrap();
                let (_, tzid) = tz_string.split_once("=").unwrap();
                let tz = string_into_tz(tzid);
                let datetime = NaiveDateTime::parse_from_str(date_string, "%Y%m%dT%H%M%S")
                    .unwrap()
                    .and_local_timezone(tz)
                    .unwrap();
                format!("{}:{}", key, datetime.to_utc().format("%Y%m%dT%H%M%SZ"))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn remove_exdates(event_buf: &str) -> String {
    event_buf
        .split("\n")
        .filter(|line| !(*line).starts_with("EXDATE"))
        .collect::<Vec<&str>>()
        .join("\n")
}

fn string_into_tz(tz_string: &str) -> Tz {
    match tz_string {
        "Europe/Berlin" => Tz::Europe__Berlin,
        "Europe/Prague" => Tz::Europe__Prague,
        "Europe/Paris" => Tz::Europe__Paris,
        "Europe/Amsterdam" => Tz::Europe__Amsterdam,
        "Europe/Brussels" => Tz::Europe__Brussels,
        _ => todo!("Unknown timezone {}", tz_string),
    }
}

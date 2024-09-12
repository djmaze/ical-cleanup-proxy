use std::str;
use std::time::Duration;

use chrono::{Datelike, TimeDelta, TimeZone};
use clap::Parser;
use moka::future::Cache;
use rrule::Tz;
use serde_derive::{Deserialize, Serialize};
use warp::{reject::Rejection, Filter};

use transform::transform;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of previous days to keep events for
    #[arg(short, long, default_value_t = 30)]
    previous_days_to_keep: u64,

    /// Time-to-live for the cache entries (in seconds)
    #[arg(short, long, default_value_t = 30 * 60)]
    ttl_seconds: u64,

    /// Maximum number of items to keep in the cache
    #[arg(short, long, default_value_t = 16)]
    capacity: u64,
}

#[derive(Serialize, Deserialize)]
struct UrlQuery {
    url: String,
}

type ResultCache = Cache<String, String>;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let cache: ResultCache = ResultCache::builder()
        .max_capacity(args.capacity)
        .time_to_live(Duration::from_secs(args.ttl_seconds))
        .build();

    let router = warp::any()
        .and(warp::query::<UrlQuery>())
        .and_then(move |q: UrlQuery| {
            let cache = cache.clone();
            let url = q.url.clone();
            async move {
                Ok::<String, Rejection>(
                    cache
                        .get_with(
                            url,
                            respond(q.url, args.previous_days_to_keep.try_into().unwrap()),
                        )
                        .await,
                )
            }
        });

    warp::serve(router).run(([0, 0, 0, 0], 3077)).await;
}

async fn respond(url: String, previous_days_to_keep: i64) -> String {
    eprintln!("{}", url);
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    let mut output = Vec::new();

    let keep_after = chrono::Utc::now().date_naive() - TimeDelta::days(previous_days_to_keep);
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

    transform(body.as_bytes(), &mut output, keep_after, true);
    str::from_utf8(&output).unwrap().to_string()
}

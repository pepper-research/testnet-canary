use sov_modules_api::digest::Digest;
use sov_modules_api::CryptoSpec;

use spicenet_shared::derivative::DerivativeID;

pub struct Seconds(i64);

impl Seconds {
    pub fn new(seconds: i64) -> Self {
        Self(seconds)
    }
}

pub fn check_valid_zdf_expiry(
    now_seconds: Seconds, // network time
    start_time_seconds: Seconds,
    expiry_time_seconds: Seconds,
) -> Result<(), String> {
    let Seconds(start_time_seconds) = start_time_seconds;
    let Seconds(expiry_time_seconds) = expiry_time_seconds;
    let Seconds(now_seconds) = now_seconds;
    let now = now_seconds.checked_mul(1000).unwrap(); // convert to milliseconds
    let start_time = start_time_seconds.checked_mul(1000).unwrap(); // convert to milliseconds
    let expiry_time = expiry_time_seconds.checked_mul(1000).unwrap(); // convert to milliseconds

    if start_time.rem_euclid(86_400_000_i64) / 3_600_000_i64 != 1
        && start_time.rem_euclid(86_400_000_i64) / 3_600_000_i64 != 23
    {
        return Err(format!("Start time {} must be exactly 1am or 11pm", start_time).into());
    }
    if expiry_time.rem_euclid(86_400_000_i64) / 3_600_000_i64 != 0 {
        return Err(format!("Expiry time {} must be 12am UTC", expiry_time).into());
    }
    if expiry_time - start_time != 25 * 60 * 60 * 1000
        && expiry_time - start_time != 23 * 60 * 60 * 1000
    {
        return Err(format!(
            "Future duration has to be 25 hours or 23 hours. Start time: {}. Expiry time: {}. Duration: {}",
            start_time,
            expiry_time,
            expiry_time - start_time,
        )
            .into());
    }
    if start_time - now > 24 * 60 * 60 * 1000 {
        return Err(format!(
            "Cannot start that far into the future. Start time: {}. Expiry time: {}. Duration: {}",
            start_time,
            expiry_time,
            expiry_time - start_time,
        )
        .into());
    }
    return Ok(());
}

// check for valid expiries so we can make rollovers permissionless
// this is for "normal" futures where expiry is on fridays 8am UTC and
// we want to allow 2 weeklies, 2 monthlies, 4 quarterlies
pub fn check_valid_future_expiry(
    start_time_seconds: Seconds,
    expiry_seconds: Seconds,
) -> Result<(), String> {
    // TODO do we also need to check the start time is a Friday 8am?
    // TODO do we also need to compare things to the network time?
    let Seconds(start_time_seconds) = start_time_seconds;
    let Seconds(expiry_seconds) = expiry_seconds;
    let start_time = start_time_seconds.checked_mul(1000).unwrap(); // convert to milliseconds
    let expiry = expiry_seconds.checked_mul(1000).unwrap(); // convert to milliseconds
                                                            // weekday = (floor(T / 86_400) + 4) mod 7
    let weekday = ((expiry / 86_400_000_i64) + 4).rem_euclid(7);

    if weekday != 5 {
        return Err(format!("Expiry {} is not a friday", expiry).into());
    }

    // hour = (T % 86_400) / 3_600
    let hour = expiry.rem_euclid(86_400_000_i64) / 3_600_000_i64;
    if hour != 8 {
        return Err(format!("Expiry {} is not 8am UTC", expiry).into());
    }

    if expiry - start_time < 0 {
        return Err(format!("Expiry {} is in the past", expiry).into());
    } else if expiry - start_time <= 15 * 24 * 60 * 60 * 1000 {
        // if we are within 2 weeks and a day any friday is fine
        return Ok(());
    } else if expiry - start_time <= 62 * 24 * 60 * 60 * 1000 {
        // if we are within 2 months and a bit needs to be the last friday of the month
        let expiry_plus_a_week = Seconds::new(expiry_seconds + 7 * 24 * 60 * 60);
        if timestamp_to_date(expiry_plus_a_week).mday <= 7 {
            // if (dt + chrono::Duration::days(7)).day() <= 7 {
            return Ok(());
        } else {
            return Err(format!("Date {} is not the last friday of the month", expiry).into());
        }
    } else if expiry - start_time <= 367 * 24 * 60 * 60 * 1000 {
        // if we are within 1 year and a day it needs to be the last friday of HMUZ
        let expiry_plus_a_week = Seconds::new(expiry_seconds + 7 * 24 * 60 * 60);
        if timestamp_to_date(expiry_plus_a_week).mday > 7 {
            // if you add a week and mday > 7, then must not be last friday
            return Err(format!("Date {} is not the last friday of the month", expiry).into());
        }
        let td = timestamp_to_date(Seconds::new(expiry_seconds));
        if [0, 2_u8, 5, 8].contains(&td.month) {
            return Ok(());
        } else {
            return Err(format!("Date {} is not a valid expiry month", expiry));
        }
    } else {
        return Err(format!("Expiry {} is too far out", expiry).into());
    }
}

/* 2000-03-01 (mod 400 year, immediately after feb29 */
const LEAPOCH: u64 = 946684800_u64 + 86400 * (31 + 29);

const DAYS_PER_400Y: u32 = 365 * 400 + 97;
const DAYS_PER_100Y: u32 = 365 * 100 + 24;
const DAYS_PER_4Y: u32 = 365 * 4 + 1;

#[derive(Debug)]
struct Date {
    year: u32,
    month: u8,
    mday: u8,
}

// lifted from C implemention linked here
// https://stackoverflow.com/questions/21593692/convert-unix-timestamp-to-date-without-system-libs
// and hosted here
// http://git.musl-libc.org/cgit/musl/tree/src/time/__secs_to_tm.c?h=v0.9.15
fn timestamp_to_date(t: Seconds) -> Date {
    let Seconds(t) = t;
    let t = t as u64;
    let (days, secs): (u32, u64);
    let (mut remdays, mut remyears): (u32, u32);
    let (qc_cycles, mut c_cycles, mut q_cycles): (u32, u32, u32);
    let (years, mut months): (u32, u32);

    let days_in_month = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];

    /* Reject time_t values whose year would overflow int */
    // if (t < INT_MIN * 31622400LL || t > INT_MAX * 31622400LL)
    // return -1;

    secs = t - LEAPOCH;
    days = (secs / 86400) as u32;

    qc_cycles = days / DAYS_PER_400Y;
    remdays = days % DAYS_PER_400Y;

    c_cycles = remdays / DAYS_PER_100Y;
    if c_cycles == 4 {
        c_cycles = c_cycles - 1;
    }
    remdays -= c_cycles * DAYS_PER_100Y;

    q_cycles = remdays / DAYS_PER_4Y;
    if q_cycles == 25 {
        q_cycles = q_cycles - 1;
    }
    remdays -= q_cycles * DAYS_PER_4Y;

    remyears = remdays / 365;
    if remyears == 4 {
        remyears = remyears - 1;
    }
    remdays -= remyears * 365;

    years = remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

    // for months in days_in_month[months as usize]..(remdays + 1) {
    months = 0;
    while days_in_month[months as usize] <= remdays {
        // for (months=0; days_in_month[months] <= remdays; months++)
        remdays -= days_in_month[months as usize];
        months += 1;
    }

    let mut res = Date {
        year: years + 100,
        month: (months + 2) as u8,
        mday: (remdays + 1) as u8,
    };

    if res.month >= 12 {
        res.month -= res.month;
        res.year += 1;
    }

    println!("{:?}", res);

    return res;
}

/// Derives derivative ID from `derivative_name`
pub fn get_derivative_id<S: sov_modules_api::Spec>(derivative_name: &str) -> DerivativeID {
    let mut hasher = <S::CryptoSpec as CryptoSpec>::Hasher::new();
    hasher.update(derivative_name.as_bytes());
    let hash: [u8; 32] = hasher.finalize().into();
    hash.into()
}

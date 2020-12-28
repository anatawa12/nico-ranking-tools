use std::time::Duration;
use std::num::NonZeroU64;

pub fn ymd_to_string(dur: Duration) -> String {
    if let Some(years) = NonZeroU64::new(dur.as_years()) {
        if let Some(months) = NonZeroU64::new(dur.subyear_months()) {
            format!("{}年{}ヶ月", years, months)
        } else {
            format!("{}年", years)
        }
    } else if let Some(months) = NonZeroU64::new(dur.subyear_months()) {
        if let Some(days) = NonZeroU64::new(dur.submonth_days()) {
            format!("{}ヶ月{}日", months, days)
        } else {
            format!("{}ヶ月", months)
        }
    } else if let Some(days) = NonZeroU64::new(dur.as_days()) {
        if let Some(hours) = NonZeroU64::new(dur.subday_hours()) {
            format!("{}日{}時間", days, hours)
        } else {
            format!("{}日", days)
        }
    } else if let Some(hours) = NonZeroU64::new(dur.as_hours()) {
        format!("{}時間{}分", hours, dur.subhour_minutes())
    } else if let Some(minutes) = NonZeroU64::new(dur.as_minutes()){
        format!("{}分{}秒", minutes, dur.submin_secs())
    } else {
        format!("{}秒", dur.as_secs())
    }
}

const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = SECS_PER_MINUTE * MINUTES_PER_HOUR;
const SECS_PER_DAY: u64 = SECS_PER_HOUR * HOURS_PER_DAY;
const SECS_PER_YEAR: u64 = SECS_PER_DAY * DAYS_PER_YEAR;

const MINUTES_PER_HOUR: u64 = 60;
const HOURS_PER_DAY: u64 = 24;
const DAYS_PER_YEAR: u64 = 364;

pub trait MinDayMonthYears {
    fn submin_secs(&self) -> u64;
    fn as_minutes(&self) -> u64;
    fn subhour_minutes(&self) -> u64;
    fn as_hours(&self) -> u64;
    fn subday_hours(&self) -> u64;
    fn as_days(&self) -> u64;
    fn subyear_days(&self) -> u64;
    fn submonth_days(&self) -> u64;
    fn as_months(&self) -> u64;
    fn subyear_months(&self) -> u64;
    fn as_years(&self) -> u64;
}

impl MinDayMonthYears for Duration {
    fn submin_secs(&self) -> u64 {
        self.as_secs() % SECS_PER_MINUTE
    }

    fn as_minutes(&self) -> u64 {
        self.as_secs() / SECS_PER_MINUTE
    }

    fn subhour_minutes(&self) -> u64 {
        self.as_minutes() % MINUTES_PER_HOUR
    }

    fn as_hours(&self) -> u64 {
        self.as_secs() / SECS_PER_HOUR
    }

    fn subday_hours(&self) -> u64 {
        self.as_hours() % HOURS_PER_DAY
    }

    fn as_days(&self) -> u64 {
        self.as_secs() / SECS_PER_DAY
    }

    fn subyear_days(&self) -> u64 {
        self.as_days() % DAYS_PER_YEAR
    }

    fn submonth_days(&self) -> u64 {
        let days = self.subyear_days();
        match days {
            0..=29 => days - 0,
            30..=59 => days - 30,
            60..=90 => days - 60,

            91..=120 => days - 91,
            121..=150 => days - 121,
            151..=181 => days - 151,

            182..=211 => days - 182,
            212..=241 => days - 212,
            242..=272 => days - 242,

            273..=302 => days - 273,
            303..=332 => days - 303,
            333..=363 => days - 333,
            _ => unreachable!(),
        }
    }

    fn as_months(&self) -> u64 {
        self.as_years() * 12 + self.subyear_months()
    }

    fn subyear_months(&self) -> u64 {
        match self.subyear_days() {
            0..=29 => 0,
            30..=59 => 1,
            60..=90 => 2,

            91..=120 => 3,
            121..=150 => 4,
            151..=181 => 5,

            182..=211 => 6,
            212..=241 => 7,
            242..=272 => 8,

            273..=302 => 9,
            303..=332 => 10,
            333..=363 => 11,
            _ => unreachable!(),
        }
    }

    fn as_years(&self) -> u64 {
        self.as_secs() / SECS_PER_YEAR
    }
}

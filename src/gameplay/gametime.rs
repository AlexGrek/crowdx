use std::{fmt, ops::Sub};

#[derive(Clone, Debug, Eq, Copy)]
pub struct TimeSpan {
    pub minutes: isize,
}

impl TimeSpan {
    pub fn new(minutes: isize) -> Self {
        Self { minutes }
    }

    pub fn new_zero() -> Self {
        Self { minutes: 0 }
    }

    pub fn new_hours(hours: isize) -> Self {
        Self {
            minutes: hours * 60,
        }
    }

    pub fn new_days(days: isize) -> Self {
        let hours = days * 24;
        Self {
            minutes: hours * 60,
        }
    }
}

impl PartialEq for TimeSpan {
    fn eq(&self, other: &Self) -> bool {
        self.minutes == other.minutes
    }
}

impl PartialOrd for TimeSpan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.minutes.cmp(&other.minutes))
    }
}

impl std::ops::Add for TimeSpan {
    type Output = TimeSpan;

    fn add(self, other: TimeSpan) -> TimeSpan {
        TimeSpan {
            minutes: self.minutes + other.minutes,
        }
    }
}

impl Sub for TimeSpan {
    type Output = TimeSpan;

    fn sub(self, other: TimeSpan) -> TimeSpan {
        TimeSpan {
            minutes: self.minutes - other.minutes,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Time {
    pub days: usize,
    pub hours: usize,
    pub minutes: usize,
    pub fraction: f32,
    pub speed: f32,
}

fn mins_to_h_and_mins(mins: usize) -> (usize, usize) {
    (mins / 60, mins % 60)
}

impl Time {
    pub fn new(start_minutes: usize) -> Self {
        let (hrs, mins) = mins_to_h_and_mins(start_minutes);
        Self {
            days: 0,
            hours: hrs,
            minutes: mins,
            fraction: 0.0,
            speed: 10.0,
        }
    }

    pub fn new_zero() -> Self {
        Self::new(0)
    }

    pub fn elapsed(&self, other: &Time) -> TimeSpan {
        let total_minutes_self = self.total_minutes() as isize;
        let total_minutes_other = other.total_minutes() as isize;
        let elapsed_minutes = total_minutes_other - total_minutes_self;
        TimeSpan::new(elapsed_minutes)
    }

    pub fn snapshot(&self) -> Time {
        Time {
            days: self.days,
            hours: self.hours,
            minutes: self.minutes,
            fraction: 0.0, // Ignoring fraction
            speed: self.speed,
        }
    }

    // Calculate total minutes
    pub fn total_minutes(&self) -> usize {
        let total_days_minutes = self.days * 24 * 60;
        let total_hours_minutes = self.hours * 60;
        total_days_minutes + total_hours_minutes + self.minutes
    }

    // Calculate total hours
    pub fn total_hours(&self) -> usize {
        let total_days_hours = self.days * 24;
        total_days_hours + self.hours
    }

    // Calculate total days
    pub fn total_days(&self) -> usize {
        self.days
    }

    // Calculate total time as a formatted string
    pub fn total_time_string(&self) -> String {
        format!(
            "{} days, {} hours, {} minutes",
            self.days, self.hours, self.minutes
        )
    }

    pub fn from_time_span(time_span: TimeSpan) -> Self {
        let days = time_span.minutes / (24 * 60);
        let remaining_minutes = time_span.minutes % (24 * 60);
        let hours = remaining_minutes / 60;
        let minutes = remaining_minutes % 60;

        Time {
            days: days as usize,
            hours: hours as usize,
            minutes: minutes as usize,
            fraction: 0.0,
            speed: 1.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.fraction += dt * self.speed;
        if self.fraction >= 1.0 {
            self.fraction -= 1.0;
            self.minutes += 1;
            if self.minutes >= 60 {
                self.minutes -= 60;
                self.hours += 1;
                if self.hours >= 24 {
                    self.hours -= 24;
                    self.days += 1;
                }
            }
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the fields as desired
        write!(f, "{:02}:{:02}", self.hours, self.minutes)
    }
}

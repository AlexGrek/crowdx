use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Time {
    pub days: usize,
    pub hours: usize,
    pub minutes: usize,
    pub fraction: f32,
    pub speed: f32
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
            speed: 10.0
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
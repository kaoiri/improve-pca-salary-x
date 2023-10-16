use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Month {
    pub year: u16,
    pub month: u16,
}

impl Month {
    pub fn new(year: u16, month: u16) -> Self {
        Self { year, month }
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >04}/{: >02}", self.year, self.month)
    }
}

impl FromStr for Month {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elements = s.split("/");

        Ok(Self {
            year: elements.next().ok_or(anyhow!("Invalid format"))?.parse()?,
            month: elements.next().ok_or(anyhow!("Invalid format"))?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum DayKind {
    Sun,
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Unknown,
}

impl Display for DayKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            DayKind::Sun => "日",
            DayKind::Mon => "月",
            DayKind::Tue => "火",
            DayKind::Wed => "水",
            DayKind::Thu => "木",
            DayKind::Fri => "金",
            DayKind::Sat => "土",
            DayKind::Unknown => "",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DayKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let day = match s {
            "日" => DayKind::Sun,
            "月" => DayKind::Mon,
            "火" => DayKind::Tue,
            "水" => DayKind::Wed,
            "木" => DayKind::Thu,
            "金" => DayKind::Fri,
            "土" => DayKind::Sat,
            _ => DayKind::Unknown,
        };

        Ok(day)
    }
}

#[derive(Debug, Clone)]
pub enum DateKind {
    On,
    Off,
    Unknown,
}

impl Display for DateKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            DateKind::On => "平日",
            DateKind::Off => "休日",
            DateKind::Unknown => "不明",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct RawDate {
    pub month: u8,
    pub date: u8,
}

impl RawDate {
    pub fn new(month: u8, date: u8) -> Self {
        Self { month, date }
    }
}

impl PartialEq for RawDate {
    fn eq(&self, other: &Self) -> bool {
        self.month == other.month && self.date == other.date
    }
}

impl Display for RawDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >02}/{: >02}", self.month, self.date)
    }
}

#[derive(Debug, Clone)]
pub struct Date {
    pub raw_date: RawDate,
    pub date_type: DateKind,
}

impl Date {
    pub fn new(month: u8, date: u8) -> Self {
        Self {
            raw_date: RawDate::new(month, date),
            date_type: DateKind::Unknown,
        }
    }

    pub fn annotate(mut self, off_list: &[Date]) -> Self {
        self.date_type = match off_list.iter().find(|o| o.raw_date == self.raw_date) {
            Some(_) => DateKind::Off,
            None => DateKind::On,
        };
        self
    }
}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.raw_date == other.raw_date
    }
}

impl FromStr for Date {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elements = s.split("/");
        let month = elements.next().ok_or(anyhow!("Invalid format"))?;
        let date = elements.next().ok_or(anyhow!("Invalid format"))?;

        Ok(Self {
            raw_date: RawDate::new(month.parse()?, date.parse()?),
            date_type: DateKind::Unknown,
        })
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw_date.to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Time {
    pub hours: u16,
    pub minutes: u16,
}

impl Time {
    pub fn new(hours: u16, minutes: u16) -> Self {
        Self { hours, minutes }.carry()
    }

    pub fn round_up(&self) -> Self {
        let minutes = (self.minutes as f32 / 15.).ceil() as u16 * 15;
        Self::new(self.hours, minutes)
    }

    pub fn round_down(&self) -> Self {
        let minutes = (self.minutes as f32 / 15.).floor() as u16 * 15;
        Self::new(self.hours, minutes)
    }

    pub fn merge(mut self, other: &Self) -> Self {
        self.hours += other.hours;
        self.minutes += other.minutes;
        self.carry()
    }

    pub fn sub(self, other: &Self) -> Self {
        let self_as_minutes = self.as_minutes();
        let other_as_minutes = other.as_minutes();

        if self_as_minutes < other_as_minutes {
            Self::new(0, 0)
        } else {
            Self::new(0, self_as_minutes - other_as_minutes)
        }
    }

    pub fn as_minutes(&self) -> u16 {
        60 * self.hours + self.minutes
    }

    fn carry(mut self) -> Self {
        if self.minutes >= 60 {
            self.minutes = self.minutes - 60;
            self.hours = self.hours + 1;
        }

        if self.minutes >= 60 {
            self = self.carry();
        }

        self
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}", self.hours as f32 + self.minutes as f32 / 60.0)
    }
}

impl FromStr for Time {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elements = s.split(":");

        Ok(Self::new(
            elements.next().ok_or(anyhow!("Invalid format"))?.parse()?,
            elements.next().ok_or(anyhow!("Invalid format"))?.parse()?,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Clock {
    pub hours: u16,
    pub minutes: u16,
}

impl Clock {
    pub fn new(hours: u16, minutes: u16) -> Self {
        Self { hours, minutes }.carry()
    }

    pub fn round_up(&self) -> Self {
        let minutes = (self.minutes as f32 / 15.).ceil() as u16 * 15;
        Self::new(self.hours, minutes)
    }

    pub fn round_down(&self) -> Self {
        let minutes = (self.minutes as f32 / 15.).floor() as u16 * 15;
        Self::new(self.hours, minutes)
    }

    pub fn diff(&self, other: &Self) -> Time {
        let mut self_as_minutes = self.as_minutes();
        let other_as_minutes = other.as_minutes();

        if self_as_minutes < other_as_minutes {
            self_as_minutes += 60 * 24;
        }

        Time::new(0, self_as_minutes - other_as_minutes)
    }

    pub fn later_than(&self, other: &Self) -> bool {
        self.as_minutes() > other.as_minutes()
    }

    pub fn or_later_than(&self, other: &Self) -> bool {
        self.as_minutes() >= other.as_minutes()
    }

    pub fn as_minutes(&self) -> u16 {
        60 * self.hours + self.minutes
    }

    fn carry(mut self) -> Self {
        if self.minutes >= 60 {
            self.minutes = self.minutes - 60;
            self.hours = self.hours + 1;
        }

        if self.hours == 24 {
            self.hours = 0;
        }

        if self.minutes >= 60 {
            self = self.carry();
        }

        self
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: >02}:{: >02}", self.hours, self.minutes)
    }
}

impl FromStr for Clock {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elements = s.split(":");

        Ok(Self::new(
            elements.next().ok_or(anyhow!("Invalid format"))?.parse()?,
            elements.next().ok_or(anyhow!("Invalid format"))?.parse()?,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Range {
    start: Clock,
    end: Clock,
}

impl Range {
    pub fn new(start: Clock, end: Clock) -> Self {
        Self { start, end }
    }

    pub fn includes(&self, other: &Self) -> bool {
        other.start.or_later_than(&self.start) && self.end.or_later_than(&other.end)
    }

    pub fn abs(&self) -> Time {
        self.end.diff(&self.start)
    }
}

#[cfg(test)]
mod tests {
    use crate::clock::{Clock, Time};

    #[test]
    fn parse() {
        let clock: Clock = "9:4".parse().unwrap();
        assert_eq!(clock, Clock::new(9, 4));

        let time: Time = "9:4".parse().unwrap();
        assert_eq!(time, Time::new(9, 4));
    }

    #[test]
    fn carry() {
        let mut before: Clock = Clock::new(9, 64);
        let mut after: Clock = Clock::new(10, 4);
        assert_eq!(before, after);

        before = Clock::new(9, 128);
        after = Clock::new(11, 8);
        assert_eq!(before, after);

        before = Clock::new(24, 32);
        after = Clock::new(0, 32);
        assert_eq!(before, after);

        before = Clock::new(23, 128);
        after = Clock::new(1, 8);
        assert_eq!(before, after);

        before = "23:128".parse().unwrap();
        after = Clock::new(1, 8);
        assert_eq!(before, after);

        let mut before: Time = Time::new(9, 64);
        let mut after: Time = Time::new(10, 4);
        assert_eq!(before, after);

        before = Time::new(9, 128);
        after = Time::new(11, 8);
        assert_eq!(before, after);

        before = Time::new(24, 32);
        after = Time::new(24, 32);
        assert_eq!(before, after);

        before = Time::new(23, 128);
        after = Time::new(25, 8);
        assert_eq!(before, after);

        before = "23:128".parse().unwrap();
        after = Time::new(25, 8);
        assert_eq!(before, after);
    }

    #[test]
    fn round() {
        let mut before = Clock::new(9, 32);
        let mut ceil = Clock::new(9, 45);
        let mut floor = Clock::new(9, 30);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 0);
        ceil = Clock::new(9, 0);
        floor = Clock::new(9, 0);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 15);
        ceil = Clock::new(9, 15);
        floor = Clock::new(9, 15);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 30);
        ceil = Clock::new(9, 30);
        floor = Clock::new(9, 30);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 45);
        ceil = Clock::new(9, 45);
        floor = Clock::new(9, 45);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 8);
        ceil = Clock::new(9, 15);
        floor = Clock::new(9, 0);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 23);
        ceil = Clock::new(9, 30);
        floor = Clock::new(9, 15);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 34);
        ceil = Clock::new(9, 45);
        floor = Clock::new(9, 30);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(9, 53);
        ceil = Clock::new(10, 00);
        floor = Clock::new(9, 45);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);

        before = Clock::new(23, 58);
        ceil = Clock::new(0, 0);
        floor = Clock::new(23, 45);
        assert_eq!(before.round_up().minutes, ceil.minutes);
        assert_eq!(before.round_down().minutes, floor.minutes);
    }
}

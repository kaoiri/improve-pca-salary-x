use crate::cell::Cell;
use crate::clock::{Clock, Date, DateKind, DayKind, Month, Range, Time};
use crate::member::Member;
use std::collections::HashSet;
use std::io::BufRead;

#[derive(Debug)]
pub struct Record {
    pub month: Cell<Month>,
    pub member: Cell<Member>,
    pub date: Cell<Date>,
    pub day: Cell<DayKind>,
    pub came_at: Cell<Clock>,
    pub left_at: Cell<Clock>,
    pub break_time: Cell<Time>,
    pub work_time: Cell<Time>,
    pub remarks: Cell<String>,
    pub days: Cell<u8>,
}

impl Record {
    pub fn from_strs(
        roster: &HashSet<Member>,
        off_list: &[Date],
        month: &str,
        member_id: &str,
        date: &str,
        day: &str,
        came_at: &str,
        left_at: &str,
        break_time: &str,
        work_time: &str,
        remarks: &str,
        days: &str,
    ) -> anyhow::Result<Self> {
        let member_id: u16 = member_id.parse()?;
        let member = roster
            .iter()
            .find(|m| m.id == member_id)
            .ok_or(anyhow!("No member has been found"))?
            .to_owned();

        Ok(Self {
            month: month.parse()?,
            member: Cell::new(member),
            date: date
                .parse::<Cell<Date>>()
                .unwrap()
                .map(|d| d.annotate(off_list)),
            day: day.parse()?,
            came_at: came_at.parse()?,
            left_at: left_at.parse()?,
            break_time: break_time.parse::<Cell<Time>>()?.or(Time::new(0, 0)),
            work_time: work_time.parse()?,
            remarks: remarks.parse()?,
            days: days.parse()?,
        })
    }

    pub fn rounded_work_time(&self) -> anyhow::Result<Time> {
        let came_at = self.came_at.peek()?.clone();
        let mut start_at = self.member.peek()?.clone().start_at();
        start_at = match came_at.later_than(&start_at) {
            true => came_at.round_up(),
            false => start_at,
        };
        let left_at = self.left_at.peek()?.clone().round_down();
        let mut work_time = Time::new(0, 0);

        let start_lunch_at = Clock::new(12, 10);
        let mut work_time_am = match left_at.later_than(&start_lunch_at) {
            true => start_lunch_at.diff(&start_at),
            false => left_at.diff(&start_at),
        };

        if left_at == Clock::new(12, 0) && self.left_at.peek()?.or_later_than(&start_lunch_at) {
            work_time_am = start_lunch_at.diff(&start_at);
        }

        if !start_at.later_than(&start_lunch_at) {
            work_time = work_time.merge(&work_time_am);
        }

        let end_lunch_at = Clock::new(13, 0);
        let start_at_pm = match start_at.later_than(&end_lunch_at) {
            true => start_at,
            false => end_lunch_at,
        };

        let work_time_pm = match left_at.later_than(&start_at_pm) {
            true => left_at.diff(&start_at_pm),
            false => Time::new(0, 0),
        };
        work_time = work_time.merge(&work_time_pm);

        work_time = work_time.sub(&self.break_time()?);
        Ok(work_time.round_down())
    }

    pub fn over_work_time(&self) -> anyhow::Result<Time> {
        let nominal = match self.date.clone().data()?.date_type {
            DateKind::On => Ok(Time::new(8, 0)),
            DateKind::Off => Ok(Time::new(0, 0)),
            DateKind::Unknown => Err(anyhow!("DateKind is not annotated")),
        };

        nominal.and_then(|n| Ok(self.rounded_work_time()?.sub(&n)))
    }

    pub fn export_rounded_to_csv(&self) -> anyhow::Result<String> {
        let mut buf: Vec<String> = vec![];
        buf.push(self.month.to_string());
        buf.push(self.member.to_string());
        buf.push(self.date.to_string());
        buf.push(self.date.peek()?.date_type.to_string());
        buf.push(self.day.to_string());
        buf.push(self.member.peek()?.start_at().to_string());
        buf.push(self.came_at.to_string());
        buf.push(self.left_at.to_string());
        buf.push(self.break_time.to_string());
        buf.push(self.work_time.to_string());
        buf.push(self.work_time.to_string().replace(".", ":"));
        buf.push(
            self.rounded_work_time()
                .unwrap_or(Time::new(0, 0))
                .to_string(),
        );
        buf.push(self.over_work_time().unwrap_or(Time::new(0, 0)).to_string());
        buf.push(self.remarks.to_string());
        buf.push(self.days.to_string());
        Ok(buf.join(","))
    }

    pub fn export_rounded_to_daily_csv(&self, is_start: bool) -> anyhow::Result<String> {
        let mut buf: Vec<String> = vec![];
        buf.push(if is_start {
            "*".to_string()
        } else {
            "".to_string()
        });
        buf.push(self.date.to_string());
        buf.push(self.member.to_string());
        buf.push(self.member.peek()?.from.clone());
        buf.push("出勤".to_string());
        buf.push(match self.came_at {
            Cell::Data(_) => "1,".to_string(),
            Cell::NoData => ",1".to_string(),
        });
        buf.push(self.member.peek()?.start_at().to_string());
        buf.push(self.member.peek()?.member_type.print_force_breaks());
        buf.push(self.left_at.to_string());
        // work_time or rounded_work_time
        /*
        buf.push(
            self.rounded_work_time()
                .unwrap_or(Time::new(0, 0))
                .to_string(),
        );
        */
        buf.push(self.work_time.to_string().replace(".", ":"));
        buf.push(self.remarks.to_string());
        Ok(buf.join(","))
    }

    pub fn break_time(&self) -> anyhow::Result<Time> {
        let mut result = self.break_time.peek()?.clone();
        let forces = self.member.peek()?.member_type.force_breaks();
        let range = Range::new(self.came_at.peek()?.clone(), self.left_at.peek()?.clone());
        for f in forces.iter() {
            if !range.includes(f) {
                result = result.sub(&f.abs());
            }
        }

        Ok(result)
    }
}

pub fn collect_from_csv<R: BufRead>(
    reader: R,
    roster: &HashSet<Member>,
    off_list: &[Date],
) -> Vec<Record> {
    reader
        .lines()
        .flat_map(|line| {
            line.ok().and_then(|l| {
                let trimmed = l.replace("\"", "");
                let mut columns = trimmed.split(",");
                Record::from_strs(
                    roster,
                    off_list,
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.nth(1).unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                )
                .ok()
            })
        })
        .collect()
}

pub fn get_csv_headings() -> &'static str {
    "年月,社員番号,氏名,日付,日付区分,曜日,規定出勤時刻,出勤時刻,退勤時刻,休憩時間,労働時間,労働時間（HH:mm）,補正労働時間,法定外労働時間,備考,出勤日数"
}

pub fn get_daily_csv_headings() -> &'static str {
    "レコードの開始行,生産日,管理番号,作業者,派遣元,出勤,出勤[出勤],出勤[欠勤],開始_time1,休憩15:00[有り],休憩15:00[無し],休憩17:00[有り],休憩17:00[無し],退勤,勤務時間,備考"
}

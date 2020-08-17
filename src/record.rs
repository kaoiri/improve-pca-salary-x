use std::io::BufRead;
use std::collections::HashSet;
use crate::member::Member;
use crate::clock::{Month, DayKind, Date, DateKind, Time, Clock};
use crate::cell::Cell;

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
    pub days: Cell<u8>
}

impl Record {
    pub fn from_strs(
        roster: &HashSet<Member>,
        off_list: &[Date],
        month: &str,
        member_id: &str, date: &str, day: &str,
        came_at: &str, left_at: &str,
        break_time: &str, work_time: &str,
        remarks: &str, days: &str
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
            date: date.parse::<Cell<Date>>().unwrap().map(|d| d.annotate(off_list)),
            day: day.parse()?,
            came_at: came_at.parse()?,
            left_at: left_at.parse()?,
            break_time: break_time.parse::<Cell<Time>>()?.or(Time::new(0, 0)),
            work_time: work_time.parse()?,
            remarks: remarks.parse()?,
            days: days.parse()?
        })
    }

    pub fn rounded_work_time(&self) -> anyhow::Result<Time> {
        let came_at = self.came_at.clone().data()?;
        let mut start_at = self.member.clone().data()?.start_at();
        start_at =
            match came_at.later_than(&start_at) {
                true  => came_at.round_up(),
                false => start_at
            };

        let mut work_time = self.left_at.clone().data()?.round_down().diff(start_at);

        // 昼休憩
        if self.left_at.peek()?.later_than(&Clock::new(13, 0)) {
            work_time = work_time.sub(&Time::new(0, 50));
        }

        work_time = work_time.sub(&self.break_time.clone().data()?);
        Ok(work_time.round_down())
    }

    pub fn over_work_time(&self) -> anyhow::Result<Time> {
        let nominal =
            match self.date.clone().data()?.date_type {
                DateKind::On => Ok(Time::new(8, 0)),
                DateKind::Off => Ok(Time::new(0, 0)),
                DateKind::Unknown => Err(anyhow!("DateKind is not annotated"))
            };

        nominal.and_then(|n| Ok(self.rounded_work_time()?.sub(&n)))
    }

    pub fn export_rounded_to_csv(&self) -> anyhow::Result<String> {
        let mut buf: Vec<String> = vec![];
        buf.push(self.month.to_string());
        buf.push(self.member.to_string());
        buf.push(self.date.to_string());
        buf.push(self.date.clone().data()?.date_type.to_string());
        buf.push(self.day.to_string());
        buf.push(self.member.clone().data()?.start_at().to_string());
        buf.push(self.came_at.to_string());
        buf.push(self.left_at.to_string());
        buf.push(self.break_time.to_string());
        buf.push(self.work_time.to_string());
        buf.push(self.rounded_work_time().unwrap_or(Time::new(0, 0)).to_string());
        buf.push(self.over_work_time().unwrap_or(Time::new(0, 0)).to_string());
        buf.push(self.remarks.to_string());
        buf.push(self.days.to_string());
        Ok(buf.join(","))
    }
}

pub fn collect_from_csv<R: BufRead>(reader: R, roster: &HashSet<Member>, off_list: &[Date]) -> Vec<Record> {
    reader
    .lines()
    .flat_map(|line| {
        line
        .ok()
        .and_then(|l| {
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
                columns.next().unwrap_or("")
            ).ok()
        })
    })
    .collect()
}

pub fn get_csv_headings() -> &'static str {
    "年月,社員番号,氏名,日付,日付区分,曜日,規定出勤時刻,出勤時刻,退勤時刻,休憩時間,労働時間,補正労働時間,法定外労働時間,備考,出勤日数"
}
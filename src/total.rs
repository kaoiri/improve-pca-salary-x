use crate::cell::Cell;
use crate::clock::Time;
use crate::member::Member;
use crate::record::Record;
use std::collections::HashSet;
use std::io::BufRead;

#[derive(Debug)]
pub struct Total {
    pub member: Cell<Member>,
    pub nominal_work_days: Cell<u8>,
    pub nominal_work_time: Cell<Time>,
    pub work_days: Cell<u8>,
    pub total_work_time: Cell<Time>,
    pub others: Vec<String>,
    pub rounded_work_time: Cell<Time>,
    pub rounded_over_work_time: Cell<Time>,
}

impl Total {
    pub fn from_strs(
        roster: &HashSet<Member>,
        member_id: &str,
        nominal_work_days: &str,
        nominal_work_time: &str,
        work_days: &str,
        total_work_time: &str,
        others: Vec<&str>,
    ) -> anyhow::Result<Self> {
        let member_id: u16 = member_id.parse()?;
        let member = roster
            .iter()
            .find(|m| m.id == member_id)
            .ok_or(anyhow!("No member has been found"))?
            .to_owned();

        Ok(Self {
            member: Cell::new(member),
            nominal_work_days: nominal_work_days.parse()?,
            nominal_work_time: nominal_work_time.parse()?,
            work_days: work_days.parse()?,
            total_work_time: total_work_time.parse()?,
            others: others.iter().map(|o| o.to_string()).collect(),
            rounded_work_time: Cell::new(Time::new(0, 0)),
            rounded_over_work_time: Cell::new(Time::new(0, 0)),
        })
    }

    pub fn empty() -> Self {
        Self {
            member: Cell::NoData,
            nominal_work_days: Cell::NoData,
            nominal_work_time: Cell::NoData,
            work_days: Cell::NoData,
            total_work_time: Cell::NoData,
            others: vec![],
            rounded_work_time: Cell::NoData,
            rounded_over_work_time: Cell::NoData,
        }
    }

    pub fn total(mut self, records: Vec<&Record>) -> anyhow::Result<Self> {
        for r in records.iter() {
            self.rounded_work_time = self
                .rounded_work_time
                .map(|s| s.merge(&r.rounded_work_time().unwrap_or(Time::new(0, 0))));
            self.rounded_over_work_time = self
                .rounded_over_work_time
                .map(|s| s.merge(&r.over_work_time().unwrap_or(Time::new(0, 0))));
        }

        Ok(self)
    }

    pub fn export_to_csv(&self) -> String {
        let mut buf: Vec<String> = vec![];
        buf.push(self.member.to_string());
        buf.push(self.nominal_work_days.to_string());
        buf.push(self.nominal_work_time.to_string());
        buf.push(self.work_days.to_string());
        buf.push(self.total_work_time.to_string());
        buf.push(self.rounded_work_time.to_string());
        buf.push(self.rounded_over_work_time.to_string());
        buf.append(
            &mut self
                .others
                .iter()
                .map(|o| o.replace(":", "."))
                .collect::<Vec<String>>(),
        );
        buf.join(",")
    }
}

pub fn collect_from_csv<R: BufRead>(reader: R, roster: &HashSet<Member>) -> Vec<Total> {
    reader
        .lines()
        .flat_map(|line| {
            line.ok().and_then(|l| {
                let trimmed = l.replace("\"", "");
                let mut columns = trimmed.split(",");
                Total::from_strs(
                    roster,
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.next().unwrap_or(""),
                    columns.collect(),
                )
                .ok()
            })
        })
        .collect()
}

pub fn get_csv_headings() -> &'static str {
    "社員コード,氏名,要勤務日数,要勤務時間,出勤日数,出勤時間,補正出勤時間,法定外労働時間,事故欠勤日数,病気欠勤日数,代休特休日数,休日出勤日数,有休消化日数,有休残日数,残業平日普通,残業平日深夜,残業休日普通,残業休日深夜,残業法定普通,残業法定深夜,遅刻早退回数,遅刻早退時間,有休日数消化,有休時間消化,有休日数残,有休時間残,有休可能時間,残業平日普通45下,残業平日普通45超,残業平日普通60超,残業平日普通代休,残業平日深夜45下,残業平日深夜45超,残業平日深夜60超,残業平日深夜代休,残業休日普通45下,残業休日普通45超,残業休日普通60超,残業休日普通代休,残業休日深夜45下,残業休日深夜45超,残業休日深夜60超,残業休日深夜代休,勤怠自由時間1,勤怠自由時間2,勤怠自由時間3,勤怠自由時間4,勤怠自由時間5,勤怠自由時間6,勤怠自由時間7,勤怠自由時間8,勤怠自由時間9,勤怠自由時間10,勤怠自由数値1,勤怠自由数値2,勤怠自由数値3,勤怠自由数値4,勤怠自由数値5,勤怠自由数値6,勤怠自由数値7,勤怠自由数値8,勤怠自由数値9,勤怠自由数値10,回数1,回数2,回数3,回数4,回数5,回数6,回数7,回数8,回数9,回数10,回数11,回数12,回数13,回数14,回数15,回数16,回数17,回数18,回数19,回数20,回数21,回数22,回数23,回数24,回数25,回数26,回数27,回数28,回数29,回数30"
}

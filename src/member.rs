use crate::clock::{Clock, Range};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::io::BufRead;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum MemberKind {
    FullTime,
    Associate,
    PartTimeA,
    PartTimeB,
    PartTimeC,
    PartTimeD,
    Unknown,
}

impl MemberKind {
    pub fn force_breaks(&self) -> Vec<Range> {
        let break0 = Range::new(Clock::new(10, 30), Clock::new(10, 40));
        let break1 = Range::new(Clock::new(15, 00), Clock::new(15, 15));
        let break2 = Range::new(Clock::new(17, 15), Clock::new(17, 30));
        let break3 = Range::new(Clock::new(19, 30), Clock::new(19, 45));

        match self {
            MemberKind::FullTime => vec![break0, break1, break2],
            MemberKind::Associate => vec![break0],
            MemberKind::PartTimeA => vec![break0],
            MemberKind::PartTimeB => vec![break0],
            MemberKind::PartTimeC => vec![break0],
            MemberKind::PartTimeD => vec![break0, break1, break2],
            _ => vec![],
        }
    }

    pub fn print_force_breaks(&self) -> String {
        // 休憩15:00[有り],休憩15:00[無し],休憩17:00[有り],休憩17:00[無し]

        match self {
            MemberKind::FullTime => "1,,1,".to_string(),
            MemberKind::Associate => ",1,,1".to_string(),
            MemberKind::PartTimeA => ",1,,1".to_string(),
            MemberKind::PartTimeB => ",1,,1".to_string(),
            MemberKind::PartTimeC => ",1,,1".to_string(),
            MemberKind::PartTimeD => "1,,1,".to_string(),
            _ => ",,,".to_string(),
        }
    }
}

impl FromStr for MemberKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let member_type = match s {
            "LUC社員" => MemberKind::FullTime,
            "LUC準社員" => MemberKind::Associate,
            "役員" => MemberKind::FullTime,
            "A" => MemberKind::PartTimeA,
            "B" => MemberKind::PartTimeB,
            "C" => MemberKind::PartTimeC,
            "D" => MemberKind::PartTimeD,
            _ => MemberKind::Unknown,
        };
        Ok(member_type)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Member {
    pub id: u16,
    pub name: String,
    pub member_type: MemberKind,
    pub from: String,
}

impl Member {
    pub fn new<T: Into<String>>(id: u16, name: T, member_type: MemberKind, from: T) -> Self {
        Self {
            id,
            name: name.into(),
            member_type,
            from: from.into(),
        }
    }

    pub fn from_strs(id: &str, name: &str, member_type: &str, from: &str) -> anyhow::Result<Self> {
        Ok(Self::new(
            id.parse()?,
            name,
            member_type.parse::<MemberKind>()?,
            from,
        ))
    }

    pub fn start_at(&self) -> Clock {
        match self.member_type {
            MemberKind::FullTime => Clock::new(8, 30),
            MemberKind::Associate => Clock::new(8, 30),
            _ => Clock::new(9, 0),
        }
    }
}

impl Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.id, self.name)
    }
}

pub fn collect_from_csv<R: BufRead>(reader: R) -> HashSet<Member> {
    reader
        .lines()
        .filter_map(|line| {
            line.ok().and_then(|l| {
                let trimmed = l.replace("\"", "");
                let columns: Vec<&str> = trimmed.split(",").collect();
                if columns.len() < 3 {
                    return None;
                }
                match Member::from_strs(
                    columns[0],
                    columns[1],
                    columns[2],
                    if columns.len() > 3 { columns[3] } else { "" },
                ) {
                    Ok(m) => Some(m),
                    _ => None,
                }
            })
        })
        .collect()
}

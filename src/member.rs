use std::fmt::{self, Display};
use std::io::BufRead;
use std::collections::HashSet;
use crate::clock::Clock;

const FULLTIME: [u16; 9] = [1, 2, 1111, 1112, 1113, 1114, 1115, 3110, 3119];

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum MemberKind {
    FullTime,
    PartTime
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Member {
    pub id: u16,
    pub name: String,
    member_type: MemberKind
}

impl Member {
    pub fn new<T: Into<String>> (id: u16, name: T) -> Self {
        let member_type = match FULLTIME.iter().find(|&&f| f == id) {
            Some(_) => MemberKind::FullTime,
            None => MemberKind::PartTime
        };
        Self { id, name: name.into(), member_type }
    }

    pub fn from_strs(id: &str, name: &str) -> Result<Self, std::num::ParseIntError> {
        Ok(Self::new(id.parse()?, name))
    }

    pub fn start_at(&self) -> Clock {
        match self.member_type {
            MemberKind::FullTime => Clock::new(9, 0),
            MemberKind::PartTime => Clock::new(8, 30)
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
        line
        .ok()
        .and_then(|l| {
            let trimmed = l.replace("\"", "");
            let columns: Vec<&str> = trimmed.split(",").collect();
            if columns.len() < 3 { return None; }
            match Member::from_strs(columns[1], columns[2]) {
                Ok(m) => Some(m),
                _ => None
            }
        })
    })
    .collect()
}
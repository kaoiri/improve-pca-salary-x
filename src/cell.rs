use std::fmt::{self, Display};
use std::str::FromStr;
use std::default::Default;

#[derive(Debug, Clone)]
pub enum Cell<T: Clone> {
    Data(T),
    NoData
}

impl<T: Clone> Cell<T> {
    pub fn new(data: T) -> Self {
        Self::Data(data)
    }

    pub fn map<F>(self, op: F) -> Self where F: FnOnce(T) -> T {
        match self {
            Self::Data(d) => Self::Data(op(d)),
            Self::NoData => Self::NoData
        }
    } 

    pub fn or(self, res: T) -> Self {
        match self {
            Self::Data(d) => Self::Data(d),
            Self::NoData =>  Self::Data(res)
        }
    }

    pub fn data(self) -> anyhow::Result<T> {
        match self {
            Self::Data(d) => Ok(d),
            Self::NoData => Err(anyhow!("Missing data"))
        }
    } 
}

impl<T: PartialEq + Clone> PartialEq for Cell<T> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Data(data) => match other {
                Self::Data(d) => data == d,
                Self::NoData => false
            },
            Self::NoData => false
        }
    }
}

impl<T: Display + Clone> Display for Cell<T> { 
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Data(d) => write!(f, "{}", d),
            Self::NoData => write!(f, "")
        }
    }
}

impl<T: FromStr + Clone> FromStr for Cell<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<T>() {
            Ok(d) => Ok(Self::Data(d)),
            _ => Ok(Self::NoData)
        }
    }
}

impl<T: Clone> Default for Cell<T> {
    fn default() -> Self { Self::NoData }
}
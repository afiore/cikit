use std::str::FromStr;
use structopt::StructOpt;
#[derive(Debug, PartialEq, StructOpt)]
pub enum SortingOrder {
    Asc,
    Desc,
}

impl FromStr for SortingOrder {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &*s.to_uppercase() {
            "ASC" => Ok(SortingOrder::Asc),
            "DESC" => Ok(SortingOrder::Desc),
            _ => Err(anyhow::Error::msg(format!(
                "Cannot parse `SortingOrder`, invalid token {}",
                s
            ))),
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum ReportSorting {
    Time(SortingOrder),
}
impl FromStr for ReportSorting {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let chunks: Vec<&str> = s.split(" ").take(2).collect();
        if chunks.len() == 1 && chunks[0].to_lowercase() == "time" {
            Ok(ReportSorting::Time(SortingOrder::Desc))
        } else if chunks.len() == 2 && chunks[0].to_lowercase() == "time" {
            let order = SortingOrder::from_str(chunks[1])?;
            Ok(ReportSorting::Time(order))
        } else {
            Err(anyhow::Error::msg(format!(
                "Cannot parse `SortingOrder`, invalid token {}",
                s
            )))
        }
    }
}

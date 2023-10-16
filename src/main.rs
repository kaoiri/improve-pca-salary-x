#[macro_use]
extern crate anyhow;

mod cell;
mod clock;
mod decode;
mod member;
mod record;
mod total;

use crate::clock::Date;
use crate::decode::Decode;
use crate::total::Total;
use encoding_rs::SHIFT_JIS;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};

fn main() -> anyhow::Result<()> {
    println!("起動しています...");
    let dir_work = env::current_dir()?;
    let exe = env::current_exe()?;
    let dir = exe.parent().unwrap_or(dir_work.as_path());
    println!("起動ディレクトリ：{:?}", dir);

    let path_roster = dir.join("名簿.csv");
    let path_records = dir.join("出勤簿.csv");
    let path_totals = dir.join("PCA給与X.csv");
    let path_offs = dir.join("休日.csv");
    let path_rounded_records = dir.join("出勤簿_補正版.csv");
    let path_rounded_daily = dir.join("派遣日報.csv");
    let path_rounded_totals = dir.join("PCA給与X_補正版.csv");

    println!("名簿を読み込んでいます...");
    let reader_roster = File::open(&path_roster)?.decode()?;
    let roster = member::collect_from_csv(reader_roster);
    println!("完了");

    println!("休日リストを読み込んでいます...");
    let reader_offs = File::open(&path_offs)?.decode()?;
    let offs: Vec<Date> = reader_offs
        .lines()
        .flat_map(|line| {
            line.unwrap_or("".to_string())
                .split(",")
                .map(|s| s.parse::<Date>().unwrap_or(Date::new(0, 0)))
                .collect::<Vec<Date>>()
        })
        .collect();
    println!("完了");

    println!("出勤簿を読み込んでいます...");
    let reader_records = File::open(&path_records)?.decode()?;
    let records = record::collect_from_csv(reader_records, &roster, &offs);
    println!("完了");

    println!("PCA給与Xを読み込んでいます...");
    let reader_totals = File::open(&path_totals)?.decode()?;
    let totals = total::collect_from_csv(reader_totals, &roster);
    println!("完了");

    println!("集計しています...");
    let rounded_totals = totals.into_iter().map(|t| {
        let the_records = records.iter().filter(|r| r.member == t.member).collect();
        t.total(the_records).unwrap_or(Total::empty())
    });
    println!("完了");

    println!("書き出しています...");
    let mut target_records = io::BufWriter::new(File::create(&path_rounded_records)?);
    let mut target_daily = io::BufWriter::new(File::create(&path_rounded_daily)?);
    let mut target_totals = io::BufWriter::new(File::create(&path_rounded_totals)?);

    write_line_with_shift_jis(&mut target_records, record::get_csv_headings().to_string())?;
    for r in &records {
        write_line_with_shift_jis(&mut target_records, r.export_rounded_to_csv()?)?;
    }

    write_line_with_shift_jis(
        &mut target_daily,
        record::get_daily_csv_headings().to_string(),
    )?;
    let mut pre: Option<Date> = None;
    for r in &records {
        write_line_with_shift_jis(
            &mut target_daily,
            r.export_rounded_to_daily_csv(
                /*match pre {
                    Some(ref date) => date != r.date.peek().unwrap(),
                    None => true,
                }*/
                false,
            )?,
        )?;
        pre = Some(r.date.peek()?.clone());
    }

    write_line_with_shift_jis(&mut target_totals, total::get_csv_headings().to_string())?;
    for t in rounded_totals {
        write_line_with_shift_jis(&mut target_totals, t.export_to_csv())?;
    }
    println!("完了");

    Ok(())
}

fn write_line_with_shift_jis(
    writer: &mut BufWriter<File>,
    s: String,
) -> Result<(), std::io::Error> {
    let line = s + "\n";
    let (encoded, _encoding, _res) = SHIFT_JIS.encode(&line);
    writer.write(&encoded)?;
    Ok(())
}

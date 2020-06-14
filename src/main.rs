use std::fs::{self, File};
use std::env;
use std::io::{self, BufRead, Read, Write};
use std::collections::HashMap;
use encoding_rs::SHIFT_JIS;

fn main() -> anyhow::Result<()> {
    println!("起動しています...");
    let dir_work = env::current_dir()?;
    let exe = env::current_exe()?;
    let dir = exe.parent().unwrap_or(dir_work.as_path());
    println!("起動ディレクトリ：{:?}", dir);

    let path_read_roster = dir.join("出勤簿.csv");
    let path_read_data = dir.join("PCA給与X.csv");
    let path_output = dir.join("PCA給与X_社員名.csv");
    let path_roster = dir.join("出勤簿_UTF8.csv");
    let path_data = dir.join("PCA給与X_UTF8.csv");

    {
        println!("CSVファイルをUTF-8に変換します...");
        let mut roster_raw = File::open(&path_read_roster)?;
        let mut data_raw = File::open(&path_read_data)?;

        let mut writer_roster = io::BufWriter::new(File::create(&path_roster)?);
        let mut writer_data = io::BufWriter::new(File::create(&path_data)?);

        let mut buf = Vec::new();
        roster_raw.read_to_end(&mut buf)?;
        let (roster_decoded, _encoding, _errors) = SHIFT_JIS.decode(&buf);
        writer_roster.write(roster_decoded.as_bytes())?;

        buf = Vec::new();
        data_raw.read_to_end(&mut buf)?;
        let (data_decoded, _encoding, _errors) = SHIFT_JIS.decode(&buf);
        writer_data.write(data_decoded.as_bytes())?;
        println!("完了");
    }

    println!("ファイルを開いています...");
    let reader_roster = io::BufReader::new(File::open(&path_roster)?);
    let reader_data = io::BufReader::new(File::open(&path_data)?);
    let mut target = io::BufWriter::new(File::create(&path_output)?);
    println!("完了");

    
    println!("名簿を読み込んでいます...");
    let roster: HashMap<String, String> =
        reader_roster
        .lines()
        .filter_map(|line| {
            line.map(|l| {
                let columns: Vec<&str> = l.split(",").collect();
                (columns[1].to_string(), columns[2].to_string())
            }).ok()
        })
        .collect();
    println!("完了");

    println!("社員名を書き込んでいます...");
    let lines = reader_data.lines();
    for (i, line) in lines.enumerate() {
        if let Ok(l) = line {
            let mut columns: Vec<&str> = l.split(",").collect();

            if i == 0 {
                   columns.insert(1, "社員名")
            } else {
                match roster.get(&columns[0].to_string()) {
                    Some(name) => columns.insert(1, name),
                    None => columns.insert(1, "")
                };
            }

            let line = columns.join(",") + "\n";
            let (encoded, _encoding, _res) = SHIFT_JIS.encode(&line);
            target.write(&encoded)?;
        }
    }
    println!("完了");

    fs::remove_file(&path_roster)?;
    fs::remove_file(&path_data)?;

    Ok(())
}
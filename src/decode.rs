use encoding_rs::SHIFT_JIS;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;

pub trait Decode
where
    Self: Sized + Read,
{
    fn decode(self) -> anyhow::Result<Cursor<String>>;
}

impl Decode for File {
    fn decode(mut self) -> anyhow::Result<Cursor<String>> {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;

        let (decoded, _encoding, _errors) = SHIFT_JIS.decode(&buf);
        Ok(Cursor::new(decoded.to_string()))
    }
}


// A single password strength assessment, as produced by whatever scored the
// password in the first place. This tool doesn't score passwords itself; it
// only moves already-scored records between formats.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub label: String,
    pub score: u8,
    pub entropy_bits: f64,
    pub crack_time: String,
}

impl Record {
    pub fn new(label: String, score: u8, entropy_bits: f64, crack_time: String) -> Result<Record, String> {
        if score > 4 {
            return Err(format!("score {} out of range 0-4 for '{}'", score, label));
        }
        Ok(Record { label, score, entropy_bits, crack_time })
    }
}

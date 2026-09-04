// A minimal JSON reader/writer for exactly the shape newer tooling expects:
// an array of objects with the four Record fields. This is not a general
// purpose JSON library; it doesn't need to be.
use crate::record::Record;

pub fn write(records: &[Record]) -> String {
    let mut out = String::from("[\n");
    for (i, r) in records.iter().enumerate() {
        out.push_str("  {\"label\": \"");
        out.push_str(&escape(&r.label));
        out.push_str("\", \"score\": ");
        out.push_str(&r.score.to_string());
        out.push_str(", \"entropy_bits\": ");
        out.push_str(&format!("{:.2}", r.entropy_bits));
        out.push_str(", \"crack_time\": \"");
        out.push_str(&escape(&r.crack_time));
        out.push_str("\"}");
        if i + 1 < records.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("]\n");
    out
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push(c),
        }
    }
    out
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, bytes: input.as_bytes(), pos: 0 }
    }

    fn skip_ws(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.skip_ws();
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", byte as char, self.pos))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".to_string()),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'"') => { out.push('"'); self.pos += 1; }
                        Some(b'\\') => { out.push('\\'); self.pos += 1; }
                        Some(b'/') => { out.push('/'); self.pos += 1; }
                        Some(b'n') => { out.push('\n'); self.pos += 1; }
                        Some(b't') => { out.push('\t'); self.pos += 1; }
                        Some(b'r') => { out.push('\r'); self.pos += 1; }
                        Some(b'u') => {
                            self.pos += 1;
                            if self.pos + 4 > self.bytes.len() {
                                return Err("truncated unicode escape".to_string());
                            }
                            let hex = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
                                .map_err(|_| "invalid unicode escape".to_string())?;
                            let code = u32::from_str_radix(hex, 16)
                                .map_err(|_| "invalid unicode escape".to_string())?;
                            out.push(char::from_u32(code).unwrap_or('?'));
                            self.pos += 4;
                        }
                        other => return Err(format!("invalid escape '{:?}'", other)),
                    }
                }
                Some(_) => {
                    let ch = self.input[self.pos..].chars().next().unwrap();
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        self.skip_ws();
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E')) {
            self.pos += 1;
        }
        self.input[start..self.pos]
            .parse::<f64>()
            .map_err(|_| format!("invalid number at byte {}", start))
    }
}

pub fn parse(input: &str) -> Result<Vec<Record>, String> {
    let mut p = Parser::new(input);
    p.expect(b'[')?;
    let mut records = Vec::new();
    p.skip_ws();
    if p.peek() == Some(b']') {
        p.pos += 1;
        return Ok(records);
    }
    loop {
        p.expect(b'{')?;
        let mut label = None;
        let mut score = None;
        let mut entropy_bits = None;
        let mut crack_time = None;
        loop {
            let key = p.parse_string()?;
            p.expect(b':')?;
            match key.as_str() {
                "label" => label = Some(p.parse_string()?),
                "score" => score = Some(p.parse_number()? as u8),
                "entropy_bits" => entropy_bits = Some(p.parse_number()?),
                "crack_time" => crack_time = Some(p.parse_string()?),
                other => return Err(format!("unknown field '{}'", other)),
            }
            p.skip_ws();
            match p.peek() {
                Some(b',') => { p.pos += 1; }
                Some(b'}') => { p.pos += 1; break; }
                other => return Err(format!("unexpected byte {:?} in object", other)),
            }
        }
        let record = Record::new(
            label.ok_or_else(|| "missing 'label'".to_string())?,
            score.ok_or_else(|| "missing 'score'".to_string())?,
            entropy_bits.ok_or_else(|| "missing 'entropy_bits'".to_string())?,
            crack_time.ok_or_else(|| "missing 'crack_time'".to_string())?,
        )?;
        records.push(record);
        p.skip_ws();
        match p.peek() {
            Some(b',') => { p.pos += 1; }
            Some(b']') => { p.pos += 1; break; }
            other => return Err(format!("unexpected byte {:?} after object", other)),
        }
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_array() {
        let records = parse("[]").unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn parses_basic_object() {
        let input = r#"[{"label": "alice", "score": 2, "entropy_bits": 34.50, "crack_time": "3 hours"}]"#;
        let records = parse(input).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].label, "alice");
        assert_eq!(records[0].score, 2);
        assert_eq!(records[0].entropy_bits, 34.50);
        assert_eq!(records[0].crack_time, "3 hours");
    }

    #[test]
    fn round_trip() {
        let records = vec![
            Record::new("alice".to_string(), 2, 34.56, "3 hours".to_string()).unwrap(),
            Record::new("bob \"the\" builder".to_string(), 4, 78.23, "centuries".to_string()).unwrap(),
            Record::new("carol\\zero".to_string(), 0, 8.10, "line1\nline2".to_string()).unwrap(),
        ];
        let text = write(&records);
        let parsed = parse(&text).unwrap();
        assert_eq!(records, parsed);
    }

    #[test]
    fn escapes_special_characters_in_write() {
        let records = vec![
            Record::new("line\nbreak".to_string(), 1, 10.0, "tab\there".to_string()).unwrap(),
        ];
        let text = write(&records);
        assert!(text.contains("line\\nbreak"));
        assert!(text.contains("tab\\there"));
    }

    #[test]
    fn rejects_missing_field() {
        let input = r#"[{"label": "alice", "score": 2, "entropy_bits": 34.50}]"#;
        let err = parse(input).unwrap_err();
        assert!(err.contains("missing"));
    }

    #[test]
    fn rejects_unknown_field() {
        let input = r#"[{"label": "alice", "score": 2, "entropy_bits": 34.50, "crack_time": "3 hours", "extra": 1}]"#;
        let err = parse(input).unwrap_err();
        assert!(err.contains("unknown field"));
    }

    #[test]
    fn rejects_score_out_of_range() {
        let input = r#"[{"label": "alice", "score": 9, "entropy_bits": 34.50, "crack_time": "3 hours"}]"#;
        let err = parse(input).unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn rejects_malformed_input() {
        let err = parse(r#"[{"label": "alice""#).unwrap_err();
        assert!(!err.is_empty());
    }
}

// The "linescore" format: one record per line, four fields separated by '|'.
//
//   label|score|entropy_bits|crack_time
//
// This is the format an old internal auth service wrote to its logs. A
// literal '\' or '|' inside a field is backslash-escaped so the field split
// stays unambiguous.
use crate::record::Record;

pub fn write(records: &[Record]) -> String {
    let mut out = String::new();
    for r in records {
        out.push_str(&escape(&r.label));
        out.push('|');
        out.push_str(&r.score.to_string());
        out.push('|');
        out.push_str(&format!("{:.2}", r.entropy_bits));
        out.push('|');
        out.push_str(&escape(&r.crack_time));
        out.push('\n');
    }
    out
}

pub fn parse(input: &str) -> Result<Vec<Record>, String> {
    let mut records = Vec::new();
    for (i, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = split_unescaped(line);
        if fields.len() != 4 {
            return Err(format!("line {}: expected 4 fields, found {}", i + 1, fields.len()));
        }
        let label = unescape(&fields[0]);
        let score: u8 = fields[1]
            .parse()
            .map_err(|_| format!("line {}: invalid score '{}'", i + 1, fields[1]))?;
        let entropy_bits: f64 = fields[2]
            .parse()
            .map_err(|_| format!("line {}: invalid entropy '{}'", i + 1, fields[2]))?;
        let crack_time = unescape(&fields[3]);
        let record = Record::new(label, score, entropy_bits, crack_time)
            .map_err(|e| format!("line {}: {}", i + 1, e))?;
        records.push(record);
    }
    Ok(records)
}

fn escape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    for c in field.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '|' => out.push_str("\\|"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

// Splits on '|', but a '\|' produced by escape() is kept intact (still
// backslash-prefixed) so unescape() can turn it back into a literal pipe.
fn split_unescaped(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            current.push('\\');
            if let Some(next) = chars.next() {
                current.push(next);
            }
        } else if c == '|' {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_line() {
        let records = parse("alice|2|34.50|3 hours\n").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].label, "alice");
        assert_eq!(records[0].score, 2);
        assert_eq!(records[0].entropy_bits, 34.50);
        assert_eq!(records[0].crack_time, "3 hours");
    }

    #[test]
    fn skips_blank_lines() {
        let input = "alice|2|34.50|3 hours\n\nbob|4|78.20|centuries\n";
        let records = parse(input).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn round_trip() {
        let records = vec![
            Record::new("alice".to_string(), 2, 34.56, "3 hours".to_string()).unwrap(),
            Record::new("bob|weird\\name".to_string(), 4, 78.23, "centuries".to_string()).unwrap(),
            Record::new("carol".to_string(), 0, 8.10, "line1\nline2".to_string()).unwrap(),
        ];
        let text = write(&records);
        let parsed = parse(&text).unwrap();
        assert_eq!(records, parsed);
    }

    #[test]
    fn rejects_wrong_field_count() {
        let err = parse("alice|2|34.50\n").unwrap_err();
        assert!(err.contains("expected 4 fields"));
    }

    #[test]
    fn rejects_invalid_score() {
        let err = parse("alice|nine|34.50|3 hours\n").unwrap_err();
        assert!(err.contains("invalid score"));
    }

    #[test]
    fn rejects_invalid_entropy() {
        let err = parse("alice|2|not-a-number|3 hours\n").unwrap_err();
        assert!(err.contains("invalid entropy"));
    }

    #[test]
    fn rejects_score_out_of_range() {
        let err = parse("alice|5|34.50|3 hours\n").unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn escapes_pipe_and_backslash_in_write() {
        let records = vec![
            Record::new("a|b\\c".to_string(), 1, 10.0, "d|e".to_string()).unwrap(),
        ];
        let text = write(&records);
        assert_eq!(text, "a\\|b\\\\c|1|10.00|d\\|e\n");
    }
}

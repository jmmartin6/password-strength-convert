// A single password strength assessment, as produced by whatever scored the
// password in the first place. This tool doesn't score passwords itself; it
// only moves already-scored records between formats.
use std::collections::HashSet;

// Entropy above this is outside anything a real password scorer would ever
// emit (it's well past a 128-byte fully random secret); values beyond it
// are almost always a decimal shifted by a bad upstream export.
const MAX_ENTROPY_BITS: f64 = 1024.0;

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
        if !entropy_bits.is_finite() || entropy_bits < 0.0 || entropy_bits > MAX_ENTROPY_BITS {
            return Err(format!(
                "entropy_bits {} out of range 0-{} for '{}'",
                entropy_bits, MAX_ENTROPY_BITS, label
            ));
        }
        Ok(Record { label, score, entropy_bits, crack_time })
    }
}

// Duplicate labels are almost always the same account scored twice by a
// re-run of the upstream job; letting both through would silently corrupt
// whatever keys off label as a unique id downstream.
pub fn check_unique_labels(records: &[Record]) -> Result<(), String> {
    let mut seen = HashSet::with_capacity(records.len());
    for r in records {
        if !seen.insert(r.label.as_str()) {
            return Err(format!("duplicate label '{}'", r.label));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero_entropy() {
        assert!(Record::new("a".to_string(), 0, 0.0, "instant".to_string()).is_ok());
    }

    #[test]
    fn accepts_max_entropy() {
        assert!(Record::new("a".to_string(), 4, MAX_ENTROPY_BITS, "eons".to_string()).is_ok());
    }

    #[test]
    fn rejects_negative_entropy() {
        let err = Record::new("a".to_string(), 2, -1.0, "3 hours".to_string()).unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn rejects_entropy_above_max() {
        let err = Record::new("a".to_string(), 2, MAX_ENTROPY_BITS + 0.1, "3 hours".to_string()).unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn rejects_non_finite_entropy() {
        let err = Record::new("a".to_string(), 2, f64::NAN, "3 hours".to_string()).unwrap_err();
        assert!(err.contains("out of range"));
        let err = Record::new("a".to_string(), 2, f64::INFINITY, "3 hours".to_string()).unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn accepts_unique_labels() {
        let records = vec![
            Record::new("alice".to_string(), 2, 34.5, "3 hours".to_string()).unwrap(),
            Record::new("bob".to_string(), 4, 78.2, "centuries".to_string()).unwrap(),
        ];
        assert!(check_unique_labels(&records).is_ok());
    }

    #[test]
    fn rejects_duplicate_labels() {
        let records = vec![
            Record::new("alice".to_string(), 2, 34.5, "3 hours".to_string()).unwrap(),
            Record::new("alice".to_string(), 1, 10.0, "instant".to_string()).unwrap(),
        ];
        let err = check_unique_labels(&records).unwrap_err();
        assert!(err.contains("duplicate label"));
    }
}

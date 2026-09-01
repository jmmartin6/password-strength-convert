mod json;
mod linescore;
mod record;

use record::Record;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.len() > 5 {
        return Err(usage());
    }
    let from = args[1].as_str();
    let to = args[2].as_str();

    let input = if let Some(path) = args.get(3) {
        fs::read_to_string(path).map_err(|e| format!("reading '{}': {}", path, e))?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).map_err(|e| e.to_string())?;
        buf
    };

    let records: Vec<Record> = match from {
        "linescore" => linescore::parse(&input)?,
        "json" => json::parse(&input)?,
        other => return Err(format!("unknown input format '{}'\n{}", other, usage())),
    };

    let output = match to {
        "linescore" => linescore::write(&records),
        "json" => json::write(&records),
        other => return Err(format!("unknown output format '{}'\n{}", other, usage())),
    };

    if let Some(path) = args.get(4) {
        fs::write(path, output).map_err(|e| format!("writing '{}': {}", path, e))?;
    } else {
        io::stdout().write_all(output.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn usage() -> String {
    "usage: pwconv <from> <to> [input-file] [output-file]\nformats: linescore, json".to_string()
}

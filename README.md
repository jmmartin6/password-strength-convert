# pwconv

Converts password-strength assessment records between two on-disk formats.

## Why

An old internal service scored passwords at signup time and logged the
result as plain text, one record per line. Newer tooling that consumes
those scores (a dashboard, an audit script) wants JSON instead. Rather than
teach every downstream consumer to read the old format, or rewrite the old
service, this converts a batch of records from one shape to the other.

It doesn't score passwords itself, and it never sees the passwords
themselves - only labels (e.g. a username) and the numbers/strings some
scorer already produced: a 0-4 strength score, an entropy estimate in bits,
and a human-readable crack-time string.

## Formats

**linescore** - one record per line, four `|`-separated fields:

```
label|score|entropy_bits|crack_time
alice|2|34.50|3 hours
bob|4|78.20|centuries
carol|0|8.10|instant
```

A literal `\` or `|` inside a field is backslash-escaped.

**json** - an array of objects with the same four fields:

```json
[
  {"label": "alice", "score": 2, "entropy_bits": 34.50, "crack_time": "3 hours"},
  {"label": "bob", "score": 4, "entropy_bits": 78.20, "crack_time": "centuries"}
]
```

`score` must be an integer from 0 to 4, and `entropy_bits` must be a finite
number from 0 to 1024, in either format. A batch may not contain the same
`label` twice.

## Usage

```
pwconv <from> <to> [input-file] [output-file]
```

`from` and `to` are each `linescore` or `json`. If `input-file` is omitted,
input is read from stdin; if `output-file` is omitted, output is written to
stdout.

```
# convert a log file to JSON
pwconv linescore json signup-scores.log scores.json

# pipe linescore data in, get JSON on stdout
cat signup-scores.log | pwconv linescore json

# round-trip back to linescore
pwconv json linescore scores.json scores.log
```

## Building

Standard library only, no dependencies:

```
cargo build --release
```

## Status

Handles the two formats above and validates score range, entropy range, and
duplicate labels within a batch. No csv format, no compressed input, and no
control over JSON output layout yet.

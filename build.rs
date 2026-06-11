//! Generates the multicodec and multibase tables from the vendored canonical
//! registry CSVs in data/ (from https://github.com/multiformats/multicodec
//! and https://github.com/multiformats/multibase).

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const MULTICODEC_CSV: &str = "data/multicodec-table.csv";
const MULTIBASE_CSV: &str = "data/multibase-table.csv";

/// "dag-pb" -> "DAG_PB"; used for both the Rust and the Python constants.
fn constant_name(name: &str) -> String {
    name.replace('-', "_").to_uppercase()
}

fn write_generated(file_name: &str, contents: &str) {
    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join(file_name);
    fs::write(dest, contents).expect("write generated table");
}

struct MulticodecRow {
    name: String,
    tag: String,
    code: u64,
    status: String,
}

/// Columns: name, tag, code, status, description (may contain commas).
fn parse_multicodec(csv: &str) -> Vec<MulticodecRow> {
    csv.lines()
        .skip(1) // header
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.splitn(5, ',').map(str::trim).collect();
            let [name, tag, code, status] = fields[..4] else {
                panic!("malformed row in {MULTICODEC_CSV}: {line:?}");
            };
            let code = code
                .strip_prefix("0x")
                .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                .unwrap_or_else(|| panic!("invalid code in {MULTICODEC_CSV}: {line:?}"));
            MulticodecRow {
                name: name.to_owned(),
                tag: tag.to_owned(),
                code,
                status: status.to_owned(),
            }
        })
        .collect()
}

fn generate_multicodec() {
    let csv = fs::read_to_string(MULTICODEC_CSV).expect("read multicodec table");
    let rows = parse_multicodec(&csv);
    assert!(
        !rows.is_empty(),
        "{MULTICODEC_CSV} parsed to an empty table"
    );

    let mut out = String::new();
    out.push_str("pub static ENTRIES: &[Entry] = &[\n");
    for row in &rows {
        let MulticodecRow {
            name,
            tag,
            code,
            status,
        } = row;
        let constant = constant_name(name);
        writeln!(
            out,
            "    Entry {{ name: {name:?}, tag: {tag:?}, code: {code:#x}, status: {status:?}, constant: {constant:?} }},"
        )
        .unwrap();
    }
    out.push_str("];\n\n");

    out.push_str("/// Every registry code as a constant, e.g. `consts::DAG_PB`.\n");
    out.push_str("#[allow(dead_code)]\npub mod consts {\n");
    for row in &rows {
        writeln!(
            out,
            "    pub const {}: u64 = {:#x};",
            constant_name(&row.name),
            row.code
        )
        .unwrap();
    }
    out.push_str("}\n\n");

    let mut name_to_index = phf_codegen::Map::new();
    let mut code_to_index = phf_codegen::Map::new();
    for (index, row) in rows.iter().enumerate() {
        name_to_index.entry(row.name.as_str(), index.to_string());
        code_to_index.entry(row.code, index.to_string());
    }
    writeln!(
        out,
        "static NAME_TO_INDEX: phf::Map<&'static str, usize> = {};\n",
        name_to_index.build()
    )
    .unwrap();
    writeln!(
        out,
        "static CODE_TO_INDEX: phf::Map<u64, usize> = {};",
        code_to_index.build()
    )
    .unwrap();

    write_generated("multicodec_gen.rs", &out);
}

struct MultibaseRow {
    name: String,
    character: char,
    status: String,
}

/// Columns: Unicode, character, encoding, description, status. The Unicode
/// codepoint, prefix character, and encoding name are 1:1:1; the codepoint
/// column is authoritative (the character column spells NUL as a label).
/// Rows whose encoding is "none" are reserved prefixes, not encodings.
fn parse_multibase(csv: &str) -> Vec<MultibaseRow> {
    csv.lines()
        .skip(1) // header
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let fields: Vec<&str> = line.splitn(4, ',').map(str::trim).collect();
            let [unicode, _character, encoding] = fields[..3] else {
                panic!("malformed row in {MULTIBASE_CSV}: {line:?}");
            };
            if encoding == "none" {
                return None;
            }
            let character = unicode
                .strip_prefix("U+")
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .and_then(char::from_u32)
                .unwrap_or_else(|| panic!("invalid codepoint in {MULTIBASE_CSV}: {line:?}"));
            let status = fields[3]
                .rsplit(',')
                .next()
                .expect("rsplit yields at least one part")
                .trim();
            Some(MultibaseRow {
                name: encoding.to_owned(),
                character,
                status: status.to_owned(),
            })
        })
        .collect()
}

fn generate_multibase() {
    let csv = fs::read_to_string(MULTIBASE_CSV).expect("read multibase table");
    let rows = parse_multibase(&csv);
    assert!(!rows.is_empty(), "{MULTIBASE_CSV} parsed to an empty table");

    let mut out = String::new();
    out.push_str("pub static ENTRIES: &[Entry] = &[\n");
    for row in &rows {
        let MultibaseRow {
            name,
            character,
            status,
        } = row;
        let constant = constant_name(name);
        writeln!(
            out,
            "    Entry {{ name: {name:?}, character: {character:?}, status: {status:?}, constant: {constant:?} }},"
        )
        .unwrap();
    }
    out.push_str("];\n\n");

    out.push_str("/// Every encoding's canonical name as a constant, e.g. `consts::BASE58BTC`.\n");
    out.push_str("#[allow(dead_code)]\npub mod consts {\n");
    for row in &rows {
        writeln!(
            out,
            "    pub const {}: &str = {:?};",
            constant_name(&row.name),
            row.name
        )
        .unwrap();
    }
    out.push_str("}\n\n");

    let mut name_to_index = phf_codegen::Map::new();
    let mut char_to_index = phf_codegen::Map::new();
    for (index, row) in rows.iter().enumerate() {
        name_to_index.entry(row.name.as_str(), index.to_string());
        char_to_index.entry(row.character as u32, index.to_string());
    }
    writeln!(
        out,
        "static NAME_TO_INDEX: phf::Map<&'static str, usize> = {};\n",
        name_to_index.build()
    )
    .unwrap();
    writeln!(
        out,
        "static CHAR_TO_INDEX: phf::Map<u32, usize> = {};",
        char_to_index.build()
    )
    .unwrap();

    write_generated("multibase_gen.rs", &out);
}

fn main() {
    println!("cargo:rerun-if-changed={MULTICODEC_CSV}");
    println!("cargo:rerun-if-changed={MULTIBASE_CSV}");
    generate_multicodec();
    generate_multibase();
}

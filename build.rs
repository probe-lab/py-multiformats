//! Generates the multicodec table from the vendored canonical registry CSV
//! (data/multicodec-table.csv, from https://github.com/multiformats/multicodec).

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const TABLE_CSV: &str = "data/multicodec-table.csv";

struct Row {
    name: String,
    tag: String,
    code: u64,
    status: String,
}

fn parse_table(csv: &str) -> Vec<Row> {
    csv.lines()
        .skip(1) // header
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            // The trailing description column may contain commas.
            let fields: Vec<&str> = line.splitn(5, ',').map(str::trim).collect();
            let [name, tag, code, status] = fields[..4] else {
                panic!("malformed row in {TABLE_CSV}: {line:?}");
            };
            let code = code
                .strip_prefix("0x")
                .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                .unwrap_or_else(|| panic!("invalid code in {TABLE_CSV}: {line:?}"));
            Row {
                name: name.to_owned(),
                tag: tag.to_owned(),
                code,
                status: status.to_owned(),
            }
        })
        .collect()
}

/// "dag-pb" -> "DAG_PB"; used for both the Rust and the Python constants.
fn constant_name(name: &str) -> String {
    name.replace('-', "_").to_uppercase()
}

fn main() {
    println!("cargo:rerun-if-changed={TABLE_CSV}");

    let csv = fs::read_to_string(TABLE_CSV).expect("read multicodec table");
    let rows = parse_table(&csv);
    assert!(!rows.is_empty(), "{TABLE_CSV} parsed to an empty table");

    let mut out = String::new();
    out.push_str("pub static ENTRIES: &[Entry] = &[\n");
    for row in &rows {
        let Row {
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

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("multicodec_gen.rs");
    fs::write(dest, out).expect("write generated multicodec table");
}

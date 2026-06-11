//! The multicodec registry, generated at build time from the vendored
//! canonical table (data/multicodec-table.csv). See build.rs.

use pyo3::prelude::*;

use crate::MultiformatsError;

/// One row of the multicodec registry.
pub struct Entry {
    pub name: &'static str,
    pub tag: &'static str,
    pub code: u64,
    pub status: &'static str,
    /// The UPPER_SNAKE_CASE constant name, e.g. "DAG_PB".
    pub constant: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/multicodec_gen.rs"));

pub fn entry_for_name(name: &str) -> Option<&'static Entry> {
    NAME_TO_INDEX.get(name).map(|&index| &ENTRIES[index])
}

pub fn entry_for_code(code: u64) -> Option<&'static Entry> {
    CODE_TO_INDEX.get(&code).map(|&index| &ENTRIES[index])
}

pub fn code_for_name(name: &str) -> Option<u64> {
    entry_for_name(name).map(|entry| entry.code)
}

pub fn name_for_code(code: u64) -> Option<&'static str> {
    entry_for_code(code).map(|entry| entry.name)
}

/// Registry entry for `name`, or a Python error.
pub fn named_entry(name: &str) -> PyResult<&'static Entry> {
    entry_for_name(name)
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown multicodec name: {name:?}")))
}

/// Registry entry for `code`, or a Python error.
pub fn coded_entry(code: u64) -> PyResult<&'static Entry> {
    entry_for_code(code)
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown multicodec code: {code:#x}")))
}

/// The code registered for a multicodec name.
#[pyfunction]
fn code(name: &str) -> PyResult<u64> {
    Ok(named_entry(name)?.code)
}

/// The name registered for a multicodec code.
#[pyfunction]
fn name(code: u64) -> PyResult<&'static str> {
    Ok(coded_entry(code)?.name)
}

/// The registry tag (e.g. "ipld", "multihash") of a name or code.
#[pyfunction]
fn tag(codec: &Bound<'_, PyAny>) -> PyResult<&'static str> {
    if let Ok(code) = codec.extract::<u64>() {
        return Ok(coded_entry(code)?.tag);
    }
    if let Ok(name) = codec.extract::<&str>() {
        return Ok(named_entry(name)?.tag);
    }
    Err(MultiformatsError::new_err(
        "codec must be an integer code or a multicodec name",
    ))
}

/// All registry entries as (name, tag, code, status) tuples, in table order.
#[pyfunction]
fn entries() -> Vec<(&'static str, &'static str, u64, &'static str)> {
    ENTRIES
        .iter()
        .map(|e| (e.name, e.tag, e.code, e.status))
        .collect()
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(code, m)?)?;
    m.add_function(wrap_pyfunction!(name, m)?)?;
    m.add_function(wrap_pyfunction!(tag, m)?)?;
    m.add_function(wrap_pyfunction!(entries, m)?)?;

    // Every registry entry as an UPPER_SNAKE_CASE integer constant,
    // e.g. DAG_PB = 0x70.
    for entry in ENTRIES {
        m.add(entry.constant, entry.code)?;
    }
    Ok(())
}

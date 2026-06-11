//! The multibase registry, generated at build time from the vendored
//! canonical table (data/multibase-table.csv). See build.rs.

use multibase::Base;
use pyo3::prelude::*;

use crate::MultiformatsError;

/// One row of the multibase registry (reserved prefixes excluded).
pub struct Entry {
    pub name: &'static str,
    pub character: char,
    pub status: &'static str,
    /// The UPPERCASE constant name, e.g. "BASE58BTC".
    pub constant: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/multibase_gen.rs"));

fn entry_for_name(name: &str) -> Option<&'static Entry> {
    NAME_TO_INDEX.get(name).map(|&index| &ENTRIES[index])
}

fn entry_for_char(character: char) -> Option<&'static Entry> {
    CHAR_TO_INDEX
        .get(&(character as u32))
        .map(|&index| &ENTRIES[index])
}

/// Translate a registry entry into the rust-multibase native constant.
/// None when the crate does not implement the encoding (e.g. proquint).
fn native_base(entry: &Entry) -> Option<Base> {
    Base::from_code(entry.character).ok()
}

pub fn base_from_name(name: &str) -> PyResult<Base> {
    let entry = entry_for_name(name).ok_or_else(|| {
        MultiformatsError::new_err(format!("unknown multibase encoding: {name:?}"))
    })?;
    native_base(entry).ok_or_else(|| {
        MultiformatsError::new_err(format!("multibase encoding {name:?} is not supported"))
    })
}

/// Encode bytes with the given multibase encoding, returning the prefixed string.
#[pyfunction]
fn encode(base: &str, data: &[u8]) -> PyResult<String> {
    Ok(multibase::encode(base_from_name(base)?, data))
}

/// Decode a multibase-prefixed string, returning `(base_name, data)`.
#[pyfunction]
fn decode(string: &str) -> PyResult<(&'static str, Vec<u8>)> {
    let (base, data) = multibase::decode(string)
        .map_err(|e| MultiformatsError::new_err(format!("invalid multibase string: {e}")))?;
    let entry = entry_for_char(base.code()).ok_or_else(|| {
        MultiformatsError::new_err(format!(
            "multibase prefix {:?} is reserved, not an encoding",
            base.code()
        ))
    })?;
    Ok((entry.name, data))
}

/// All registry encodings as (name, prefix_character, status) tuples, in
/// table order — including ones the underlying implementation cannot
/// encode (compare with bases()).
#[pyfunction]
fn entries() -> Vec<(&'static str, char, &'static str)> {
    ENTRIES
        .iter()
        .map(|entry| (entry.name, entry.character, entry.status))
        .collect()
}

/// Names of all supported multibase encodings, sorted alphabetically.
#[pyfunction]
fn bases() -> Vec<&'static str> {
    let mut names: Vec<_> = ENTRIES
        .iter()
        .filter(|entry| native_base(entry).is_some())
        .map(|entry| entry.name)
        .collect();
    names.sort_unstable();
    names
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    m.add_function(wrap_pyfunction!(bases, m)?)?;
    m.add_function(wrap_pyfunction!(entries, m)?)?;

    // Every registry encoding as an UPPERCASE string constant holding its
    // canonical name (e.g. BASE58BTC = "base58btc"), accepted anywhere a
    // base name is. The multibase registry identifies encodings by prefix
    // character, not integer code, so the constants are name strings.
    for entry in ENTRIES {
        m.add(entry.constant, entry.name)?;
    }
    Ok(())
}

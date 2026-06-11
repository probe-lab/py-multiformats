use std::collections::HashMap;
use std::sync::OnceLock;

use multibase::Base;
use pyo3::prelude::*;

use crate::MultiformatsError;

const EMOJI_PREFIX: char = '\u{1F680}'; // 🚀, the base256emoji multibase code

/// Canonical multibase spec names mapped to the Rust `Base` enum.
const BASES: &[(&str, Base)] = &[
    ("identity", Base::Identity),
    ("base2", Base::Base2),
    ("base8", Base::Base8),
    ("base10", Base::Base10),
    ("base16", Base::Base16Lower),
    ("base16upper", Base::Base16Upper),
    ("base32", Base::Base32Lower),
    ("base32upper", Base::Base32Upper),
    ("base32pad", Base::Base32PadLower),
    ("base32padupper", Base::Base32PadUpper),
    ("base32hex", Base::Base32HexLower),
    ("base32hexupper", Base::Base32HexUpper),
    ("base32hexpad", Base::Base32HexPadLower),
    ("base32hexpadupper", Base::Base32HexPadUpper),
    ("base32z", Base::Base32Z),
    ("base36", Base::Base36Lower),
    ("base36upper", Base::Base36Upper),
    ("base58flickr", Base::Base58Flickr),
    ("base58btc", Base::Base58Btc),
    ("base64", Base::Base64),
    ("base64pad", Base::Base64Pad),
    ("base64url", Base::Base64Url),
    ("base64urlpad", Base::Base64UrlPad),
    ("base256emoji", Base::Base256Emoji),
];

pub fn base_from_name(name: &str) -> PyResult<Base> {
    BASES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, b)| *b)
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown multibase encoding: {name:?}")))
}

pub fn base_name(base: Base) -> &'static str {
    BASES
        .iter()
        .find(|(_, b)| *b == base)
        .map(|(n, _)| *n)
        .expect("every Base variant has a name entry")
}

/// The base256emoji alphabet mapped back to byte values, derived from the
/// (correct) encoder. The multibase crate's own base256emoji *decoder* is
/// broken: its lookup table is generated with `char_indices()`, which yields
/// byte offsets instead of ordinals for multi-byte characters.
fn emoji_reverse_table() -> &'static HashMap<char, u8> {
    static TABLE: OnceLock<HashMap<char, u8>> = OnceLock::new();
    TABLE.get_or_init(|| {
        (0u8..=255)
            .map(|byte| {
                let encoded = Base::Base256Emoji.encode([byte]);
                let symbol = encoded.chars().next().expect("encoder emits one symbol");
                (symbol, byte)
            })
            .collect()
    })
}

fn decode_emoji_payload(payload: &str) -> PyResult<Vec<u8>> {
    let table = emoji_reverse_table();
    payload
        .chars()
        .map(|c| {
            table.get(&c).copied().ok_or_else(|| {
                MultiformatsError::new_err(format!(
                    "invalid multibase string: {c:?} is not in the base256emoji alphabet"
                ))
            })
        })
        .collect()
}

/// Decode any multibase-prefixed string, routing base256emoji around the
/// broken upstream decoder.
pub fn decode_any(string: &str) -> PyResult<(Base, Vec<u8>)> {
    if let Some(payload) = string.strip_prefix(EMOJI_PREFIX) {
        return Ok((Base::Base256Emoji, decode_emoji_payload(payload)?));
    }
    multibase::decode(string)
        .map_err(|e| MultiformatsError::new_err(format!("invalid multibase string: {e}")))
}

/// Encode bytes with the given multibase encoding, returning the prefixed string.
#[pyfunction]
fn encode(base: &str, data: &[u8]) -> PyResult<String> {
    Ok(multibase::encode(base_from_name(base)?, data))
}

/// Decode a multibase-prefixed string, returning `(base_name, data)`.
#[pyfunction]
fn decode(string: &str) -> PyResult<(&'static str, Vec<u8>)> {
    let (base, data) = decode_any(string)?;
    Ok((base_name(base), data))
}

/// Names of all supported multibase encodings.
#[pyfunction]
fn bases() -> Vec<&'static str> {
    BASES.iter().map(|(n, _)| *n).collect()
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    m.add_function(wrap_pyfunction!(bases, m)?)?;
    Ok(())
}

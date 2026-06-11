use multibase::Base;
use phf::phf_map;
use pyo3::prelude::*;

use crate::MultiformatsError;

/// Canonical multibase spec names for the Rust `Base` enum, defined here
/// because the crate itself only knows the one-character codes. One listing
/// expands into both lookup directions at compile time: a perfect hash map
/// for name -> Base and an exhaustive match (a jump table) for Base -> name.
/// The match also breaks the build if the crate ever adds a variant.
macro_rules! define_bases {
    ($($name:literal => $variant:ident,)*) => {
        static BASES: phf::Map<&'static str, Base> = phf_map! {
            $($name => Base::$variant,)*
        };

        pub fn base_name(base: Base) -> &'static str {
            match base {
                $(Base::$variant => $name,)*
            }
        }
    };
}

define_bases! {
    "identity" => Identity,
    "base2" => Base2,
    "base8" => Base8,
    "base10" => Base10,
    "base16" => Base16Lower,
    "base16upper" => Base16Upper,
    "base32" => Base32Lower,
    "base32upper" => Base32Upper,
    "base32pad" => Base32PadLower,
    "base32padupper" => Base32PadUpper,
    "base32hex" => Base32HexLower,
    "base32hexupper" => Base32HexUpper,
    "base32hexpad" => Base32HexPadLower,
    "base32hexpadupper" => Base32HexPadUpper,
    "base32z" => Base32Z,
    "base36" => Base36Lower,
    "base36upper" => Base36Upper,
    "base58flickr" => Base58Flickr,
    "base58btc" => Base58Btc,
    "base64" => Base64,
    "base64pad" => Base64Pad,
    "base64url" => Base64Url,
    "base64urlpad" => Base64UrlPad,
    "base256emoji" => Base256Emoji,
}

pub fn base_from_name(name: &str) -> PyResult<Base> {
    BASES
        .get(name)
        .copied()
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown multibase encoding: {name:?}")))
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
    Ok((base_name(base), data))
}

/// Names of all supported multibase encodings, sorted alphabetically.
#[pyfunction]
fn bases() -> Vec<&'static str> {
    let mut names: Vec<_> = BASES.keys().copied().collect();
    names.sort_unstable();
    names
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    m.add_function(wrap_pyfunction!(bases, m)?)?;
    Ok(())
}

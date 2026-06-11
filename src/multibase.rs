use multibase::Base;
use phf::phf_map;
use pyo3::prelude::*;

use crate::MultiformatsError;

/// Canonical multibase spec names mapped to the Rust `Base` enum. The crate
/// itself only knows the one-character codes, not the spec names, so the
/// mapping is defined here (compile-time perfect hash map, O(1) lookup).
static BASES: phf::Map<&'static str, Base> = phf_map! {
    "identity" => Base::Identity,
    "base2" => Base::Base2,
    "base8" => Base::Base8,
    "base10" => Base::Base10,
    "base16" => Base::Base16Lower,
    "base16upper" => Base::Base16Upper,
    "base32" => Base::Base32Lower,
    "base32upper" => Base::Base32Upper,
    "base32pad" => Base::Base32PadLower,
    "base32padupper" => Base::Base32PadUpper,
    "base32hex" => Base::Base32HexLower,
    "base32hexupper" => Base::Base32HexUpper,
    "base32hexpad" => Base::Base32HexPadLower,
    "base32hexpadupper" => Base::Base32HexPadUpper,
    "base32z" => Base::Base32Z,
    "base36" => Base::Base36Lower,
    "base36upper" => Base::Base36Upper,
    "base58flickr" => Base::Base58Flickr,
    "base58btc" => Base::Base58Btc,
    "base64" => Base::Base64,
    "base64pad" => Base::Base64Pad,
    "base64url" => Base::Base64Url,
    "base64urlpad" => Base::Base64UrlPad,
    "base256emoji" => Base::Base256Emoji,
};

pub fn base_from_name(name: &str) -> PyResult<Base> {
    BASES
        .get(name)
        .copied()
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown multibase encoding: {name:?}")))
}

/// The exhaustive match breaks the build if the crate ever adds a variant.
pub fn base_name(base: Base) -> &'static str {
    match base {
        Base::Identity => "identity",
        Base::Base2 => "base2",
        Base::Base8 => "base8",
        Base::Base10 => "base10",
        Base::Base16Lower => "base16",
        Base::Base16Upper => "base16upper",
        Base::Base32Lower => "base32",
        Base::Base32Upper => "base32upper",
        Base::Base32PadLower => "base32pad",
        Base::Base32PadUpper => "base32padupper",
        Base::Base32HexLower => "base32hex",
        Base::Base32HexUpper => "base32hexupper",
        Base::Base32HexPadLower => "base32hexpad",
        Base::Base32HexPadUpper => "base32hexpadupper",
        Base::Base32Z => "base32z",
        Base::Base36Lower => "base36",
        Base::Base36Upper => "base36upper",
        Base::Base58Flickr => "base58flickr",
        Base::Base58Btc => "base58btc",
        Base::Base64 => "base64",
        Base::Base64Pad => "base64pad",
        Base::Base64Url => "base64url",
        Base::Base64UrlPad => "base64urlpad",
        Base::Base256Emoji => "base256emoji",
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bases_map_and_base_name_agree() {
        for (name, base) in BASES.entries() {
            assert_eq!(base_name(*base), *name);
        }
    }
}

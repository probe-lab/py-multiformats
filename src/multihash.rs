use multihash_codetable::{Code, MultihashDigest};
use multihash_derive::Hasher;
use phf::phf_map;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::MultiformatsError;

type Multihash64 = ::multihash::Multihash<64>;

// Identity and SHA-1 are not part of the codetable's `Code` enum (identity
// has no hasher, sha1 was dropped as insecure), and rust-multihash exposes
// no constants for their multicodec codes, so these two are defined here.
// Every other code comes from the `Code` variants via the generated
// `From<Code> for u64` / `TryFrom<u64> for Code` impls.
const IDENTITY_CODE: u64 = 0x00;
const IDENTITY_NAME: &str = "identity";
const SHA1_CODE: u64 = 0x11;
const SHA1_NAME: &str = "sha1";

/// Multicodec names for the codetable algorithms. One listing expands into
/// the name -> `Code` perfect hash map, the exhaustive `Code` -> name match
/// (which breaks the build if the codetable gains a variant), and the
/// per-algorithm Python digest functions.
macro_rules! define_table_algorithms {
    ($($pyname:ident => $name:literal : $variant:ident,)*) => {
        static TABLE_ALGORITHMS: phf::Map<&'static str, Code> = phf_map! {
            $($name => Code::$variant,)*
        };

        fn table_name(code: Code) -> &'static str {
            match code {
                $(Code::$variant => $name,)*
            }
        }

        fn register_table_digest_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
            $(
                #[pyfunction]
                fn $pyname(data: &[u8]) -> PyMultihash {
                    PyMultihash { inner: Code::$variant.digest(data) }
                }
                m.add_function(wrap_pyfunction!($pyname, m)?)?;
            )*
            Ok(())
        }
    };
}

define_table_algorithms! {
    sha2_256 => "sha2-256" : Sha2_256,
    sha2_512 => "sha2-512" : Sha2_512,
    sha3_224 => "sha3-224" : Sha3_224,
    sha3_256 => "sha3-256" : Sha3_256,
    sha3_384 => "sha3-384" : Sha3_384,
    sha3_512 => "sha3-512" : Sha3_512,
    keccak_224 => "keccak-224" : Keccak224,
    keccak_256 => "keccak-256" : Keccak256,
    keccak_384 => "keccak-384" : Keccak384,
    keccak_512 => "keccak-512" : Keccak512,
    blake3 => "blake3" : Blake3_256,
    blake2b_256 => "blake2b-256" : Blake2b256,
    blake2b_512 => "blake2b-512" : Blake2b512,
    blake2s_128 => "blake2s-128" : Blake2s128,
    blake2s_256 => "blake2s-256" : Blake2s256,
    ripemd_160 => "ripemd-160" : Ripemd160,
    ripemd_256 => "ripemd-256" : Ripemd256,
    ripemd_320 => "ripemd-320" : Ripemd320,
}

pub fn code_name(code: u64) -> Option<&'static str> {
    match code {
        IDENTITY_CODE => Some(IDENTITY_NAME),
        SHA1_CODE => Some(SHA1_NAME),
        _ => Code::try_from(code).ok().map(table_name),
    }
}

fn wrap(code: u64, digest: &[u8]) -> PyResult<Multihash64> {
    Multihash64::wrap(code, digest)
        .map_err(|e| MultiformatsError::new_err(format!("invalid multihash: {e}")))
}

fn sha1_digest(data: &[u8]) -> PyResult<Multihash64> {
    let mut hasher = multihash_codetable::Sha1::default();
    hasher.update(data);
    wrap(SHA1_CODE, hasher.finalize())
}

fn digest_by_code(code: u64, data: &[u8]) -> PyResult<Multihash64> {
    match code {
        IDENTITY_CODE => wrap(IDENTITY_CODE, data),
        SHA1_CODE => sha1_digest(data),
        _ => Code::try_from(code).map(|c| c.digest(data)).map_err(|_| {
            MultiformatsError::new_err(format!("unknown hash algorithm code: {code:#x}"))
        }),
    }
}

fn digest_by_name(name: &str, data: &[u8]) -> PyResult<Multihash64> {
    match name {
        IDENTITY_NAME => wrap(IDENTITY_CODE, data),
        SHA1_NAME => sha1_digest(data),
        _ => TABLE_ALGORITHMS
            .get(name)
            .map(|c| c.digest(data))
            .ok_or_else(|| MultiformatsError::new_err(format!("unknown hash algorithm: {name:?}"))),
    }
}

/// A multihash: a self-describing hash (algorithm code, digest size, digest).
#[pyclass(
    name = "Multihash",
    module = "multiformats",
    frozen,
    eq,
    hash,
    skip_from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyMultihash {
    pub(crate) inner: Multihash64,
}

// PyO3 methods take `&self`; the `to_*` names are the Python-facing API.
#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyMultihash {
    /// Wrap an already-computed digest in a multihash with the given code.
    #[staticmethod]
    fn wrap(code: u64, digest: &[u8]) -> PyResult<Self> {
        Ok(Self {
            inner: wrap(code, digest)?,
        })
    }

    /// Parse a binary multihash.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let inner = Multihash64::from_bytes(data)
            .map_err(|e| MultiformatsError::new_err(format!("invalid multihash: {e}")))?;
        Ok(Self { inner })
    }

    /// The multicodec code of the hash algorithm.
    #[getter]
    fn code(&self) -> u64 {
        self.inner.code()
    }

    /// The multicodec name of the hash algorithm, or None if unknown.
    #[getter]
    fn name(&self) -> Option<&'static str> {
        code_name(self.inner.code())
    }

    /// The digest size in bytes.
    #[getter]
    fn size(&self) -> u8 {
        self.inner.size()
    }

    /// The raw digest.
    #[getter]
    fn digest(&self) -> &[u8] {
        self.inner.digest()
    }

    /// The binary multihash encoding (varint code, varint size, digest).
    fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes()
    }

    fn __repr__(&self) -> String {
        let digest = self
            .inner
            .digest()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        match self.name() {
            Some(name) => format!("Multihash({name}, digest={digest})"),
            None => format!("Multihash(code={:#x}, digest={digest})", self.inner.code()),
        }
    }
}

/// Hash data with the algorithm given by multicodec name or numeric code.
#[derive(FromPyObject)]
enum Algorithm<'py> {
    Name(Bound<'py, pyo3::types::PyString>),
    Code(u64),
}

#[pyfunction]
fn digest(algorithm: Algorithm<'_>, data: &[u8]) -> PyResult<PyMultihash> {
    let inner = match algorithm {
        Algorithm::Name(name) => digest_by_name(name.to_str()?, data)?,
        Algorithm::Code(code) => digest_by_code(code, data)?,
    };
    Ok(PyMultihash { inner })
}

/// Mapping of supported algorithm names to their multicodec codes,
/// sorted by code.
#[pyfunction]
fn codes(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let mut entries: Vec<(&'static str, u64)> = TABLE_ALGORITHMS
        .entries()
        .map(|(name, code)| (*name, u64::from(*code)))
        .collect();
    entries.push((IDENTITY_NAME, IDENTITY_CODE));
    entries.push((SHA1_NAME, SHA1_CODE));
    entries.sort_unstable_by_key(|(_, code)| *code);

    let dict = PyDict::new(py);
    for (name, code) in entries {
        dict.set_item(name, code)?;
    }
    Ok(dict)
}

#[pyfunction]
fn identity(data: &[u8]) -> PyResult<PyMultihash> {
    Ok(PyMultihash {
        inner: wrap(IDENTITY_CODE, data)?,
    })
}

#[pyfunction]
fn sha1(data: &[u8]) -> PyResult<PyMultihash> {
    Ok(PyMultihash {
        inner: sha1_digest(data)?,
    })
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMultihash>()?;
    m.add_function(wrap_pyfunction!(digest, m)?)?;
    m.add_function(wrap_pyfunction!(codes, m)?)?;
    m.add_function(wrap_pyfunction!(identity, m)?)?;
    m.add_function(wrap_pyfunction!(sha1, m)?)?;
    register_table_digest_fns(m)
}

use multihash_codetable::{Code, MultihashDigest};
use multihash_derive::Hasher;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::multicodec;
use crate::MultiformatsError;

type Multihash64 = ::multihash::Multihash<64>;

/// Per-algorithm Python digest functions for the codetable hashers. Only
/// the function name <-> `Code` variant pairing is listed here, because it
/// cannot be derived mechanically (e.g. `blake3` is `Code::Blake3_256`);
/// codes and names come from the `Code` enum and the multicodec registry.
macro_rules! define_digest_fns {
    ($($pyname:ident => $variant:ident,)*) => {
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

define_digest_fns! {
    sha2_256 => Sha2_256,
    sha2_512 => Sha2_512,
    sha3_224 => Sha3_224,
    sha3_256 => Sha3_256,
    sha3_384 => Sha3_384,
    sha3_512 => Sha3_512,
    keccak_224 => Keccak224,
    keccak_256 => Keccak256,
    keccak_384 => Keccak384,
    keccak_512 => Keccak512,
    blake3 => Blake3_256,
    blake2b_256 => Blake2b256,
    blake2b_512 => Blake2b512,
    blake2s_128 => Blake2s128,
    blake2s_256 => Blake2s256,
    ripemd_160 => Ripemd160,
    ripemd_256 => Ripemd256,
    ripemd_320 => Ripemd320,
}

/// The registry name of a hash code; None for codes that are not registered
/// as multihashes (the registry also holds codecs from other namespaces).
pub fn code_name(code: u64) -> Option<&'static str> {
    multicodec::entry_for_code(code)
        .filter(|entry| entry.tag == "multihash")
        .map(|entry| entry.name)
}

fn wrap(code: u64, digest: &[u8]) -> PyResult<Multihash64> {
    Multihash64::wrap(code, digest)
        .map_err(|e| MultiformatsError::new_err(format!("invalid multihash: {e}")))
}

fn digest_by_code(code: u64, data: &[u8]) -> PyResult<Multihash64> {
    if let Ok(table_code) = Code::try_from(code) {
        return Ok(table_code.digest(data));
    }
    // Identity and sha1 are not part of the codetable's `Code` enum (identity
    // has no hasher, sha1 was dropped as insecure); dispatch them by their
    // registry names.
    match code_name(code) {
        Some("identity") => wrap(code, data),
        Some("sha1") => {
            let mut hasher = multihash_codetable::Sha1::default();
            hasher.update(data);
            wrap(code, hasher.finalize())
        }
        _ => Err(MultiformatsError::new_err(format!(
            "unknown hash algorithm code: {code:#x}"
        ))),
    }
}

/// Whether `code` can be digested: in the codetable, or one of the two
/// manually dispatched algorithms.
fn is_supported_code(code: u64) -> bool {
    Code::try_from(code).is_ok() || matches!(code_name(code), Some("identity" | "sha1"))
}

fn digest_by_name(name: &str, data: &[u8]) -> PyResult<Multihash64> {
    let code = multicodec::code_for_name(name)
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown hash algorithm: {name:?}")))?;
    digest_by_code(code, data)
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
    let mut supported: Vec<&multicodec::Entry> = multicodec::ENTRIES
        .iter()
        .filter(|entry| entry.tag == "multihash" && is_supported_code(entry.code))
        .collect();
    supported.sort_unstable_by_key(|entry| entry.code);

    let dict = PyDict::new(py);
    for entry in supported {
        dict.set_item(entry.name, entry.code)?;
    }
    Ok(dict)
}

#[pyfunction]
fn identity(data: &[u8]) -> PyResult<PyMultihash> {
    Ok(PyMultihash {
        inner: digest_by_name("identity", data)?,
    })
}

#[pyfunction]
fn sha1(data: &[u8]) -> PyResult<PyMultihash> {
    Ok(PyMultihash {
        inner: digest_by_name("sha1", data)?,
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

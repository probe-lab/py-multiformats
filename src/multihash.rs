use multihash_codetable::{Code, MultihashDigest};
use multihash_derive::Hasher;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::MultiformatsError;

type Multihash64 = ::multihash::Multihash<64>;

const IDENTITY_CODE: u64 = 0x00;
const SHA1_CODE: u64 = 0x11;

/// Multicodec names for every supported hash algorithm.
const ALGORITHMS: &[(&str, u64)] = &[
    ("identity", IDENTITY_CODE),
    ("sha1", SHA1_CODE),
    ("sha2-256", 0x12),
    ("sha2-512", 0x13),
    ("sha3-224", 0x17),
    ("sha3-256", 0x16),
    ("sha3-384", 0x15),
    ("sha3-512", 0x14),
    ("keccak-224", 0x1a),
    ("keccak-256", 0x1b),
    ("keccak-384", 0x1c),
    ("keccak-512", 0x1d),
    ("blake3", 0x1e),
    ("blake2b-256", 0xb220),
    ("blake2b-512", 0xb240),
    ("blake2s-128", 0xb250),
    ("blake2s-256", 0xb260),
    ("ripemd-160", 0x1053),
    ("ripemd-256", 0x1054),
    ("ripemd-320", 0x1055),
];

pub fn code_name(code: u64) -> Option<&'static str> {
    ALGORITHMS.iter().find(|(_, c)| *c == code).map(|(n, _)| *n)
}

fn code_by_name(name: &str) -> PyResult<u64> {
    ALGORITHMS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
        .ok_or_else(|| MultiformatsError::new_err(format!("unknown hash algorithm: {name:?}")))
}

fn wrap(code: u64, digest: &[u8]) -> PyResult<Multihash64> {
    Multihash64::wrap(code, digest)
        .map_err(|e| MultiformatsError::new_err(format!("invalid multihash: {e}")))
}

fn digest_by_code(code: u64, data: &[u8]) -> PyResult<Multihash64> {
    match code {
        // The identity and sha1 hashers are not part of the codetable's
        // `Code` enum, so they are dispatched manually.
        IDENTITY_CODE => wrap(IDENTITY_CODE, data),
        SHA1_CODE => {
            let mut hasher = multihash_codetable::Sha1::default();
            hasher.update(data);
            wrap(SHA1_CODE, hasher.finalize())
        }
        _ => {
            let code = Code::try_from(code).map_err(|_| {
                MultiformatsError::new_err(format!("unknown hash algorithm code: {code:#x}"))
            })?;
            Ok(code.digest(data))
        }
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
    let code = match algorithm {
        Algorithm::Name(name) => code_by_name(name.to_str()?)?,
        Algorithm::Code(code) => code,
    };
    Ok(PyMultihash {
        inner: digest_by_code(code, data)?,
    })
}

/// Mapping of supported algorithm names to their multicodec codes.
#[pyfunction]
fn codes(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let dict = PyDict::new(py);
    for (name, code) in ALGORITHMS {
        dict.set_item(name, code)?;
    }
    Ok(dict)
}

macro_rules! digest_fns {
    ($m:ident, $(($pyname:ident, $code:expr)),* $(,)?) => {
        $(
            #[pyfunction]
            fn $pyname(data: &[u8]) -> PyResult<PyMultihash> {
                Ok(PyMultihash { inner: digest_by_code($code, data)? })
            }
            $m.add_function(wrap_pyfunction!($pyname, $m)?)?;
        )*
    };
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMultihash>()?;
    m.add_function(wrap_pyfunction!(digest, m)?)?;
    m.add_function(wrap_pyfunction!(codes, m)?)?;
    digest_fns!(
        m,
        (identity, IDENTITY_CODE),
        (sha1, SHA1_CODE),
        (sha2_256, 0x12),
        (sha2_512, 0x13),
        (sha3_224, 0x17),
        (sha3_256, 0x16),
        (sha3_384, 0x15),
        (sha3_512, 0x14),
        (keccak_224, 0x1a),
        (keccak_256, 0x1b),
        (keccak_384, 0x1c),
        (keccak_512, 0x1d),
        (blake3, 0x1e),
        (blake2b_256, 0xb220),
        (blake2b_512, 0xb240),
        (blake2s_128, 0xb250),
        (blake2s_256, 0xb260),
        (ripemd_160, 0x1053),
        (ripemd_256, 0x1054),
        (ripemd_320, 0x1055),
    );
    Ok(())
}

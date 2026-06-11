use ::cid::{Cid, Version};
use pyo3::prelude::*;

use crate::multibase::base_from_name;
use crate::multihash::PyMultihash;
use crate::MultiformatsError;

fn cid_err(e: impl std::fmt::Display) -> PyErr {
    MultiformatsError::new_err(format!("invalid CID: {e}"))
}

/// Accept a codec as either its integer code or its multicodec name.
fn resolve_codec(codec: &Bound<'_, PyAny>) -> PyResult<u64> {
    if let Ok(code) = codec.extract::<u64>() {
        return Ok(code);
    }
    if let Ok(name) = codec.extract::<&str>() {
        return crate::multicodec::code_for_name(name).ok_or_else(|| {
            MultiformatsError::new_err(format!("unknown multicodec name: {name:?}"))
        });
    }
    Err(MultiformatsError::new_err(
        "codec must be an integer code or a multicodec name",
    ))
}

/// A self-describing content identifier (version, codec, multihash).
#[pyclass(
    name = "CID",
    module = "multiformats",
    frozen,
    eq,
    hash,
    skip_from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyCid {
    inner: Cid,
}

// PyO3 methods take `&self`; the `to_*` names are the Python-facing API.
#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyCid {
    #[new]
    fn new(version: u64, codec: &Bound<'_, PyAny>, hash: &PyMultihash) -> PyResult<Self> {
        let version = Version::try_from(version).map_err(cid_err)?;
        let inner = Cid::new(version, resolve_codec(codec)?, hash.inner).map_err(cid_err)?;
        Ok(Self { inner })
    }

    /// Parse a CID from its string form (multibase-prefixed CIDv1 or base58btc CIDv0).
    #[staticmethod]
    fn decode(string: &str) -> PyResult<Self> {
        let inner = Cid::try_from(string).map_err(cid_err)?;
        Ok(Self { inner })
    }

    /// Parse a CID from its binary form.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let inner = Cid::try_from(data).map_err(cid_err)?;
        Ok(Self { inner })
    }

    /// The CID version (0 or 1).
    #[getter]
    fn version(&self) -> u64 {
        self.inner.version().into()
    }

    /// The multicodec code of the content type (e.g. 0x70 for dag-pb).
    #[getter]
    fn codec(&self) -> u64 {
        self.inner.codec()
    }

    /// The multicodec name of the content type (e.g. "dag-pb"), or None if
    /// the code is not in the registry.
    #[getter]
    fn codec_name(&self) -> Option<&'static str> {
        crate::multicodec::name_for_code(self.inner.codec())
    }

    /// The multihash of the content.
    #[getter]
    fn hash(&self) -> PyMultihash {
        PyMultihash {
            inner: *self.inner.hash(),
        }
    }

    /// The binary CID encoding.
    fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes()
    }

    /// Encode as a string. `base` is a multibase name (CIDv1 defaults to
    /// "base32"; CIDv0 has no multibase prefix and only allows "base58btc").
    #[pyo3(signature = (base = None))]
    fn encode(&self, base: Option<&str>) -> PyResult<String> {
        match base {
            None => Ok(self.inner.to_string()),
            Some(name) => self
                .inner
                .to_string_of_base(base_from_name(name)?)
                .map_err(cid_err),
        }
    }

    /// Convert to a CIDv1 (returns self if already v1).
    fn to_v1(&self) -> PyResult<Self> {
        let inner = self.inner.into_v1().map_err(cid_err)?;
        Ok(Self { inner })
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("CID({:?})", self.inner.to_string())
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCid>()?;
    Ok(())
}

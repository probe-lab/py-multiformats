use std::str::FromStr;

use ::multiaddr::Multiaddr;
use pyo3::prelude::*;
use pyo3::types::PyIterator;

use crate::MultiformatsError;

fn addr_err(e: impl std::fmt::Display) -> PyErr {
    MultiformatsError::new_err(format!("invalid multiaddr: {e}"))
}

fn parse(s: &str) -> PyResult<Multiaddr> {
    Multiaddr::from_str(s).map_err(addr_err)
}

/// Accept either a Multiaddr instance or a multiaddr string.
fn to_multiaddr(obj: &Bound<'_, PyAny>) -> PyResult<Multiaddr> {
    if let Ok(addr) = obj.cast::<PyMultiaddr>() {
        return Ok(addr.get().inner.clone());
    }
    if let Ok(s) = obj.extract::<&str>() {
        return parse(s);
    }
    Err(MultiformatsError::new_err(
        "expected a Multiaddr or a multiaddr string",
    ))
}

/// Split a protocol component's canonical string form ("/ip4/127.0.0.1")
/// into a (name, value) pair.
fn component_parts(component: &::multiaddr::Protocol<'_>) -> (String, Option<String>) {
    let s = component.to_string();
    let body = s.strip_prefix('/').unwrap_or(&s);
    match body.split_once('/') {
        Some((name, value)) => (name.to_owned(), Some(value.to_owned())),
        None => (body.to_owned(), None),
    }
}

/// A self-describing network address.
#[pyclass(
    name = "Multiaddr",
    module = "multiformats",
    frozen,
    eq,
    hash,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PyMultiaddr {
    inner: Multiaddr,
}

#[pymethods]
impl PyMultiaddr {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        Ok(Self {
            inner: parse(addr)?,
        })
    }

    /// Parse a multiaddr from its binary form.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let inner = Multiaddr::try_from(data.to_vec()).map_err(addr_err)?;
        Ok(Self { inner })
    }

    /// The binary multiaddr encoding.
    fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_vec()
    }

    /// The protocol names in the address, in order (e.g. ["ip4", "tcp"]).
    fn protocols(&self) -> Vec<&'static str> {
        self.inner.protocol_stack().collect()
    }

    /// The (protocol, value) pairs in the address, in order. Protocols
    /// without a value (e.g. "quic-v1") have a value of None.
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.inner.iter().map(|p| component_parts(&p)).collect()
    }

    /// Return a new Multiaddr with `other` appended.
    fn encapsulate(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut inner = self.inner.clone();
        for component in to_multiaddr(other)?.iter() {
            inner.push(component);
        }
        Ok(Self { inner })
    }

    /// Return a new Multiaddr with the last occurrence of `other` and
    /// everything after it removed. Returns the address unchanged if
    /// `other` is not contained in it.
    fn decapsulate(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let s = self.inner.to_string();
        let o = to_multiaddr(other)?.to_string();
        match s.rfind(&o) {
            None => Ok(self.clone()),
            Some(0) => Ok(Self {
                inner: Multiaddr::empty(),
            }),
            Some(index) => Ok(Self {
                inner: parse(&s[..index])?,
            }),
        }
    }

    fn __iter__<'py>(slf: PyRef<'py, Self>) -> PyResult<Bound<'py, PyIterator>> {
        let items = slf.items().into_pyobject(slf.py())?;
        PyIterator::from_object(&items)
    }

    fn __len__(&self) -> usize {
        self.inner.iter().count()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Multiaddr({:?})", self.inner.to_string())
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMultiaddr>()?;
    Ok(())
}

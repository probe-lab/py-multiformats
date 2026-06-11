use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

mod cid;
mod multiaddr;
mod multibase;
mod multicodec;
mod multihash;

create_exception!(
    multiformats,
    MultiformatsError,
    PyValueError,
    "Raised when multiformats data fails to parse, decode, or encode."
);

#[pymodule]
fn _multiformats(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("MultiformatsError", py.get_type::<MultiformatsError>())?;

    let multibase_mod = PyModule::new(py, "multibase")?;
    multibase::register(&multibase_mod)?;
    m.add_submodule(&multibase_mod)?;

    let multicodec_mod = PyModule::new(py, "multicodec")?;
    multicodec::register(&multicodec_mod)?;
    m.add_submodule(&multicodec_mod)?;

    let multihash_mod = PyModule::new(py, "multihash")?;
    multihash::register(&multihash_mod)?;
    m.add_submodule(&multihash_mod)?;

    let cid_mod = PyModule::new(py, "cid")?;
    cid::register(&cid_mod)?;
    m.add_submodule(&cid_mod)?;

    let multiaddr_mod = PyModule::new(py, "multiaddr")?;
    multiaddr::register(&multiaddr_mod)?;
    m.add_submodule(&multiaddr_mod)?;

    Ok(())
}

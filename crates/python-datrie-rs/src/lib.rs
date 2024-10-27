use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyInt, PyList, PyString, PyTuple},
};

use alpha_map::AlphaMap;

mod alpha_map;

/// A Python module implemented in Rust.
#[pymodule]
fn _datrie_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AlphaMap>()?;
    m.add_class::<BaseTrie>()?;
    m.add_class::<Trie>()?;
    Ok(())
}

#[pyclass(subclass)]
struct BaseTrie {
    alpha_map: Py<AlphaMap>,
}

#[pymethods]
impl BaseTrie {
    #[new]
    #[pyo3(signature=(alphabet=None,ranges=None,alpha_map=None))]
    fn new<'a>(
        alphabet: Option<Bound<'a, PyString>>,
        ranges: Option<Bound<'a, PyList>>,
        alpha_map: Option<Bound<'a, AlphaMap>>,
        py: Python<'_>,
    ) -> PyResult<Self> {
        if alphabet.is_none() && ranges.is_none() && alpha_map.is_none() {
            return Err(PyValueError::new_err(
                "Please provide alphabet, ranges or alpha_map argument.",
            ));
        }
        let alpha_map: Py<AlphaMap> = match alpha_map {
            Some(alpha_map) => alpha_map.unbind(),
            None => Py::new(
                py,
                PyClassInitializer::from(AlphaMap::new(alphabet, ranges)?),
            )?,
        };
        Ok(BaseTrie { alpha_map })
    }
}

#[pyclass(extends=BaseTrie,subclass)]
struct Trie {}

#[pymethods]
impl Trie {
    #[new]
    #[pyo3(signature=(alphabet=None,ranges=None,alpha_map=None))]
    fn new<'a>(
        alphabet: Option<Bound<'a, PyString>>,
        ranges: Option<Bound<'a, PyList>>,
        alpha_map: Option<Bound<'a, AlphaMap>>,
        py: Python<'_>,
    ) -> PyResult<(Self, BaseTrie)> {
        Ok((Trie {}, BaseTrie::new(alphabet, ranges, alpha_map, py)?))
    }

    fn is_dirty(&self) -> bool {
        true
    }
}

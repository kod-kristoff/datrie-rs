use pyo3::{
    prelude::*,
    types::{PyList, PyString, PyTuple},
};

#[pyclass]
pub struct AlphaMap {
    alpha_map: datrie::alpha_map::AlphaMap2,
}

#[pymethods]
impl AlphaMap {
    #[new]
    #[pyo3(signature=(alphabet=None,ranges=None))]
    pub fn new<'py>(
        alphabet: Option<Bound<'py, PyString>>,
        ranges: Option<Bound<'py, PyList>>,
    ) -> PyResult<Self> {
        let mut alpha_map = AlphaMap::default();
        if let Some(ranges) = ranges {
            for range in ranges.iter() {
                let range = range.downcast_into::<PyTuple>()?;
                let start = (range.get_item(0)?).downcast_into::<PyString>()?;
                let end = (range.get_item(1)?).downcast_into::<PyString>()?;
                alpha_map.add_range(start, end)?;
            }
        }
        if let Some(alphabet) = alphabet {
            alpha_map.add_alphabet(alphabet)?;
        }
        Ok(alpha_map)
    }

    pub fn add_range<'py>(
        &mut self,
        start: Bound<'py, PyString>,
        end: Bound<'py, PyString>,
    ) -> PyResult<()> {
        let builtins = PyModule::import_bound(start.py(), "builtins")?;
        let start: datrie::AlphaChar = builtins.getattr("ord")?.call1((start,))?.extract()?;
        let end: datrie::AlphaChar = builtins.getattr("ord")?.call1((end,))?.extract()?;
        self._add_range(start, end);
        Ok(())
    }

    pub fn add_alphabet<'py>(&mut self, alphabet: Bound<'py, PyString>) -> PyResult<()> {
        let builtins = PyModule::import_bound(alphabet.py(), "builtins")?;
        let mut chars = Vec::new();
        for ch in alphabet.iter()? {
            let ch = ch?;
            let ch: datrie::AlphaChar = builtins.getattr("ord")?.call1((ch,))?.extract()?;
            chars.push(ch);
        }
        chars.sort();

        if chars.is_empty() {
            return Ok(());
        }
        let mut ranges = Vec::new();
        let mut curr_start: u32 = chars[0];
        let mut curr_end: u32 = chars[0];
        for ch in &chars[1..] {
            if ch - curr_end > 1 {
                ranges.push((curr_start, curr_end));
                curr_start = *ch;
                curr_end = *ch;
            } else {
                curr_end = *ch;
            }
        }
        for (begin, end) in ranges {
            self._add_range(begin, end);
        }
        Ok(())
    }
}

impl Default for AlphaMap {
    fn default() -> Self {
        Self {
            alpha_map: datrie::alpha_map::AlphaMap2::default(),
        }
    }
}

impl AlphaMap {
    fn _add_range(&mut self, begin: datrie::AlphaChar, end: datrie::AlphaChar) {
        self.alpha_map.add_range(begin, end).unwrap();
    }
}

//! Helpers for Python SDK implementation.
use pyo3::prelude::*;

/// Similar to [`pyo3::ToPyObject`] but allows the conversion to fail.
pub trait TryToPyObject {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject>;
}

// Implementing on `&T` to allow dtolnay specialization[1] (e.g., for `Option<T>` below).
//
// [1]: https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
impl<T: ToPyObject> TryToPyObject for &T {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.to_object(py))
    }
}

impl<T> TryToPyObject for Py<T> {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.to_object(py))
    }
}

impl<T: TryToPyObject> TryToPyObject for Option<T> {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        match self {
            Some(it) => it.try_to_pyobject(py),
            None => Ok(().to_object(py)),
        }
    }
}

use micromail::python_api;
use pyo3::prelude::*;

// We simply re-export the Python API from the main crate
// The pymodule macro is defined in the main crate
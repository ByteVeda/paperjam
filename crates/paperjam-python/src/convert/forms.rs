use paperjam_core::forms::types::FormField;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert a Rust FormField to a Python dict.
pub fn form_field_to_py<'py>(py: Python<'py>, field: &FormField) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("name", &field.name)?;
    dict.set_item("field_type", field.field_type.as_str())?;
    dict.set_item("value", field.value.as_deref())?;
    dict.set_item("default_value", field.default_value.as_deref())?;
    dict.set_item("page", field.page)?;
    dict.set_item(
        "rect",
        field.rect.map(|r| (r[0], r[1], r[2], r[3])),
    )?;
    dict.set_item("read_only", field.read_only)?;
    dict.set_item("required", field.required)?;
    dict.set_item("max_length", field.max_length)?;

    let options_list = PyList::empty(py);
    for opt in &field.options {
        let opt_dict = PyDict::new(py);
        opt_dict.set_item("display", &opt.display)?;
        opt_dict.set_item("export_value", &opt.export_value)?;
        options_list.append(opt_dict)?;
    }
    dict.set_item("options", options_list)?;

    Ok(dict)
}

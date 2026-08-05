use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::document::PyDocument;
use crate::errors::to_py_err;

#[pyfunction]
#[pyo3(name = "fill_form", signature = (document, values, need_appearances=true, generate_appearances=false))]
pub fn py_fill_form<'py>(
    py: Python<'py>,
    document: &PyDocument,
    values: &Bound<'py, PyDict>,
    need_appearances: bool,
    generate_appearances: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let mut field_values = HashMap::new();
    for (key, val) in values.iter() {
        let k: String = key.extract()?;
        let v: String = val.extract()?;
        field_values.insert(k, v);
    }

    let options = paperjam_core::forms::types::FillFormOptions {
        need_appearances,
        generate_appearances,
    };

    let (filled_doc, result) = py
        .allow_threads(move || {
            paperjam_core::forms::fill_form_fields(&inner, &field_values, &options)
        })
        .map_err(to_py_err)?;

    let result_dict = PyDict::new(py);
    result_dict.set_item("fields_filled", result.fields_filled)?;
    result_dict.set_item("fields_not_found", result.fields_not_found)?;

    let not_found_list = PyList::empty(py);
    for name in &result.not_found_names {
        not_found_list.append(name)?;
    }
    result_dict.set_item("not_found_names", not_found_list)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(filled_doc),
        },
        result_dict,
    ))
}

#[pyfunction]
#[pyo3(name = "modify_form_field", signature = (document, field_name, **kwargs))]
pub fn py_modify_form_field<'py>(
    py: Python<'py>,
    document: &PyDocument,
    field_name: String,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    let mut options = paperjam_core::forms::types::ModifyFieldOptions::default();

    if let Some(kw) = kwargs {
        if let Some(val) = kw.get_item("value")? {
            options.value = Some(val.extract::<String>()?);
        }
        if let Some(val) = kw.get_item("default_value")? {
            options.default_value = Some(val.extract::<String>()?);
        }
        if let Some(val) = kw.get_item("read_only")? {
            options.read_only = Some(val.extract::<bool>()?);
        }
        if let Some(val) = kw.get_item("required")? {
            options.required = Some(val.extract::<bool>()?);
        }
        if let Some(val) = kw.get_item("max_length")? {
            options.max_length = Some(val.extract::<u32>()?);
        }
        if let Some(val) = kw.get_item("options")? {
            let opts_list: Vec<Bound<'py, PyDict>> = val.extract()?;
            let mut choice_opts = Vec::new();
            for opt_dict in &opts_list {
                let display: String = opt_dict
                    .get_item("display")?
                    .ok_or_else(|| {
                        pyo3::exceptions::PyValueError::new_err("Option missing 'display' key")
                    })?
                    .extract()?;
                let export_value: String = opt_dict
                    .get_item("export_value")?
                    .ok_or_else(|| {
                        pyo3::exceptions::PyValueError::new_err("Option missing 'export_value' key")
                    })?
                    .extract()?;
                choice_opts.push(paperjam_core::forms::types::ChoiceOption {
                    display,
                    export_value,
                });
            }
            options.options = Some(choice_opts);
        }
    }

    let field_name_clone = field_name.clone();
    let (modified_doc, result) = py
        .allow_threads(move || {
            paperjam_core::forms::modify_form_field(&inner, &field_name_clone, &options)
        })
        .map_err(to_py_err)?;

    let result_dict = PyDict::new(py);
    result_dict.set_item("field_name", &result.field_name)?;
    result_dict.set_item("modified", result.modified)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(modified_doc),
        },
        result_dict,
    ))
}

#[pyfunction]
#[pyo3(name = "add_form_field", signature = (
    document, name, field_type, page, rect,
    value=None, default_value=None, read_only=false, required=false,
    max_length=None, options=None, font_size=0.0, generate_appearance=true
))]
#[allow(clippy::too_many_arguments)]
pub fn py_add_form_field<'py>(
    py: Python<'py>,
    document: &PyDocument,
    name: String,
    field_type: String,
    page: u32,
    rect: (f64, f64, f64, f64),
    value: Option<String>,
    default_value: Option<String>,
    read_only: bool,
    required: bool,
    max_length: Option<u32>,
    options: Option<Vec<Bound<'py, PyDict>>>,
    font_size: f64,
    generate_appearance: bool,
) -> PyResult<(PyDocument, Bound<'py, PyDict>)> {
    let inner = std::sync::Arc::clone(&document.inner);

    // Parse field type string to enum
    let ft = match field_type.as_str() {
        "text" => paperjam_core::forms::types::FormFieldType::Text,
        "checkbox" => paperjam_core::forms::types::FormFieldType::Checkbox,
        "radio_button" => paperjam_core::forms::types::FormFieldType::RadioButton,
        "combo_box" => paperjam_core::forms::types::FormFieldType::ComboBox,
        "list_box" => paperjam_core::forms::types::FormFieldType::ListBox,
        "push_button" => paperjam_core::forms::types::FormFieldType::PushButton,
        "signature" => paperjam_core::forms::types::FormFieldType::Signature,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown field type: '{}'. Must be one of: text, checkbox, radio_button, \
                 combo_box, list_box, push_button, signature",
                other
            )));
        }
    };

    // Parse choice options
    let choice_opts = if let Some(opts) = options {
        let mut result = Vec::new();
        for opt_dict in &opts {
            let display: String = opt_dict
                .get_item("display")?
                .ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("Option missing 'display' key")
                })?
                .extract()?;
            let export_value: String = opt_dict
                .get_item("export_value")?
                .ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("Option missing 'export_value' key")
                })?
                .extract()?;
            result.push(paperjam_core::forms::types::ChoiceOption {
                display,
                export_value,
            });
        }
        result
    } else {
        Vec::new()
    };

    let create_options = paperjam_core::forms::types::CreateFieldOptions {
        name: name.clone(),
        field_type: ft,
        page,
        rect: [rect.0, rect.1, rect.2, rect.3],
        value,
        default_value,
        read_only,
        required,
        max_length,
        options: choice_opts,
        font_size,
        generate_appearance,
    };

    let (new_doc, result) = py
        .allow_threads(move || {
            paperjam_core::forms::create::create_form_field(&inner, &create_options)
        })
        .map_err(to_py_err)?;

    let result_dict = PyDict::new(py);
    result_dict.set_item("field_name", &result.field_name)?;
    result_dict.set_item("created", result.created)?;

    Ok((
        PyDocument {
            inner: std::sync::Arc::new(new_doc),
        },
        result_dict,
    ))
}

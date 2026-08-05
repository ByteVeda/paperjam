use paperjam_core::signature::types::{SignatureInfo, SignatureValidity};
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Convert a Rust SignatureInfo to a Python dict.
pub fn signature_info_to_py<'py>(
    py: Python<'py>,
    sig: &SignatureInfo,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("name", &sig.name)?;
    dict.set_item("signer", sig.signer.as_deref())?;
    dict.set_item("reason", sig.reason.as_deref())?;
    dict.set_item("location", sig.location.as_deref())?;
    dict.set_item("date", sig.date.as_deref())?;
    dict.set_item("contact_info", sig.contact_info.as_deref())?;
    dict.set_item(
        "byte_range",
        sig.byte_range.map(|br| (br[0], br[1], br[2], br[3])),
    )?;
    dict.set_item("covers_whole_document", sig.covers_whole_document)?;
    dict.set_item("has_timestamp", sig.has_timestamp)?;
    dict.set_item("timestamp_date", sig.timestamp_date.as_deref())?;
    dict.set_item("has_ocsp", sig.has_ocsp)?;
    dict.set_item("has_crls", sig.has_crls)?;

    if let Some(ref cert) = sig.certificate {
        let cert_dict = PyDict::new(py);
        cert_dict.set_item("subject", &cert.subject)?;
        cert_dict.set_item("issuer", &cert.issuer)?;
        cert_dict.set_item("serial_number", &cert.serial_number)?;
        cert_dict.set_item("not_before", &cert.not_before)?;
        cert_dict.set_item("not_after", &cert.not_after)?;
        cert_dict.set_item("is_self_signed", cert.is_self_signed)?;
        dict.set_item("certificate", cert_dict)?;
    } else {
        dict.set_item("certificate", py.None())?;
    }

    Ok(dict)
}

/// Convert a Rust SignatureValidity to a Python dict.
pub fn signature_validity_to_py<'py>(
    py: Python<'py>,
    result: &SignatureValidity,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("name", &result.name)?;
    dict.set_item("integrity_ok", result.integrity_ok)?;
    dict.set_item("certificate_valid", result.certificate_valid)?;
    dict.set_item("message", &result.message)?;
    dict.set_item("signer", result.signer.as_deref())?;
    dict.set_item("timestamp_valid", result.timestamp_valid)?;
    dict.set_item("revocation_ok", result.revocation_ok)?;
    dict.set_item("is_ltv", result.is_ltv)?;
    Ok(dict)
}

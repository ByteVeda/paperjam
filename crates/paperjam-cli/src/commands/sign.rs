use crate::cli::{Cli, SignArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &SignArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let cert_data =
        std::fs::read(&args.cert).map_err(|_| CliError::FileNotFound(args.cert.clone()))?;
    let key_data =
        std::fs::read(&args.key).map_err(|_| CliError::FileNotFound(args.key.clone()))?;

    // Parse PEM or use raw DER
    let cert_der = pem_or_der(&cert_data);
    let key_der = pem_or_der(&key_data);

    let options = paperjam_core::signature::SignOptions {
        reason: args.reason.clone(),
        location: args.location.clone(),
        ..paperjam_core::signature::SignOptions::default()
    };

    let signed_bytes =
        paperjam_core::signature::sign_document(&doc, &key_der, &[cert_der], &options)?;
    std::fs::write(&args.output, &signed_bytes)?;

    if !cli.quiet {
        println!("Signed -> {}", args.output.display());
    }

    Ok(())
}

/// If the data looks like PEM, extract the DER payload. Otherwise return as-is.
fn pem_or_der(data: &[u8]) -> Vec<u8> {
    if let Ok(text) = std::str::from_utf8(data) {
        if text.contains("-----BEGIN") {
            // Simple PEM extraction: find base64 between headers
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with("-----")).collect();
            let b64: String = lines.join("");
            if let Ok(decoded) = base64_decode(&b64) {
                return decoded;
            }
        }
    }
    data.to_vec()
}

fn base64_decode(input: &str) -> Result<Vec<u8>, CliError> {
    // Simple base64 decoder for PEM files
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf = Vec::new();
    let mut bits: u32 = 0;
    let mut n_bits: u32 = 0;

    for &b in input.as_bytes() {
        if b == b'=' || b == b'\n' || b == b'\r' || b == b' ' {
            continue;
        }
        let val = table
            .iter()
            .position(|&c| c == b)
            .ok_or_else(|| CliError::InvalidArgument("Invalid base64 in PEM".into()))?
            as u32;
        bits = (bits << 6) | val;
        n_bits += 6;
        if n_bits >= 8 {
            n_bits -= 8;
            buf.push((bits >> n_bits) as u8);
            bits &= (1 << n_bits) - 1;
        }
    }
    Ok(buf)
}

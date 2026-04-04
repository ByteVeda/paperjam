use paperjam_core::encryption::{EncryptionAlgorithm, EncryptionOptions, Permissions};

use crate::cli::{Cli, EncryptArgs};
use crate::document::open_document;
use crate::error::CliError;

pub fn run(args: &EncryptArgs, cli: &Cli) -> Result<(), CliError> {
    let doc = open_document(&args.file, cli.password.as_deref())?;

    let algorithm = match args.algorithm.to_lowercase().as_str() {
        "rc4" => EncryptionAlgorithm::Rc4,
        "aes128" | "aes-128" => EncryptionAlgorithm::Aes128,
        "aes256" | "aes-256" => EncryptionAlgorithm::Aes256,
        other => {
            return Err(CliError::InvalidArgument(format!(
                "Unknown algorithm: {}. Use rc4, aes128, or aes256.",
                other
            )));
        }
    };

    let owner_password = args
        .owner_password
        .clone()
        .unwrap_or_else(|| args.user_password.clone());

    let options = EncryptionOptions {
        user_password: args.user_password.clone(),
        owner_password,
        algorithm,
        permissions: Permissions {
            print: !args.no_print,
            copy: !args.no_copy,
            modify: !args.no_modify,
            ..Permissions::default()
        },
    };

    let bytes = paperjam_core::encryption::encrypt(&doc, &options)?;
    std::fs::write(&args.output, &bytes)?;

    if !cli.quiet {
        println!("Encrypted -> {}", args.output.display());
    }

    Ok(())
}

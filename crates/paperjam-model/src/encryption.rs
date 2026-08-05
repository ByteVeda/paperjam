/// Which cipher to use for encryption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncryptionAlgorithm {
    Rc4,
    #[default]
    Aes128,
    Aes256,
}

/// Options for encrypting a PDF document.
pub struct EncryptionOptions {
    pub user_password: String,
    pub owner_password: String,
    pub permissions: Permissions,
    pub algorithm: EncryptionAlgorithm,
}

/// Granular permission flags for an encrypted PDF.
pub struct Permissions {
    pub print: bool,
    pub modify: bool,
    pub copy: bool,
    pub annotate: bool,
    pub fill_forms: bool,
    pub accessibility: bool,
    pub assemble: bool,
    pub print_high_quality: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            print: true,
            modify: true,
            copy: true,
            annotate: true,
            fill_forms: true,
            accessibility: true,
            assemble: true,
            print_high_quality: true,
        }
    }
}

impl Permissions {
    /// Encode permissions as a 32-bit integer per PDF spec (Table 3.20).
    pub fn to_i32(&self) -> i32 {
        let mut p: u32 = 0xFFFFF000; // bits 13-32 set
        p |= 0b1100_0000; // bits 7-8 set

        if self.print {
            p |= 1 << 2;
        }
        if self.modify {
            p |= 1 << 3;
        }
        if self.copy {
            p |= 1 << 4;
        }
        if self.annotate {
            p |= 1 << 5;
        }
        if self.fill_forms {
            p |= 1 << 8;
        }
        if self.accessibility {
            p |= 1 << 9;
        }
        if self.assemble {
            p |= 1 << 10;
        }
        if self.print_high_quality {
            p |= 1 << 11;
        }

        p as i32
    }
}

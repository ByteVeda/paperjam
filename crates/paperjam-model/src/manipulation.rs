/// Options for merging PDF documents.
#[derive(Debug, Clone, Default)]
pub struct MergeOptions {
    pub deduplicate_resources: bool,
}

/// Page rotation angle.
#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
}

impl Rotation {
    pub fn as_degrees(&self) -> i32 {
        match self {
            Self::Degrees0 => 0,
            Self::Degrees90 => 90,
            Self::Degrees180 => 180,
            Self::Degrees270 => 270,
        }
    }
}

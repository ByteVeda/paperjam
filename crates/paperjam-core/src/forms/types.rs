/// Type of a PDF form field.
#[derive(Debug, Clone, PartialEq)]
pub enum FormFieldType {
    Text,
    Checkbox,
    RadioButton,
    ComboBox,
    ListBox,
    PushButton,
    Signature,
    Unknown(String),
}

impl FormFieldType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Checkbox => "checkbox",
            Self::RadioButton => "radio_button",
            Self::ComboBox => "combo_box",
            Self::ListBox => "list_box",
            Self::PushButton => "push_button",
            Self::Signature => "signature",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

/// An option in a choice field (combo box or list box).
#[derive(Debug, Clone)]
pub struct ChoiceOption {
    /// Display text (what the user sees).
    pub display: String,
    /// Export value (what gets stored). Same as display if not specified.
    pub export_value: String,
}

/// A form field extracted from the PDF's AcroForm.
#[derive(Debug, Clone)]
pub struct FormField {
    /// Fully qualified field name (e.g., "person.name.first").
    pub name: String,
    /// Type of the form field.
    pub field_type: FormFieldType,
    /// Current value of the field (None if unset).
    pub value: Option<String>,
    /// Default value (None if unset).
    pub default_value: Option<String>,
    /// Page number this field appears on (1-based, None if unknown).
    pub page: Option<u32>,
    /// Field rectangle on the page [x1, y1, x2, y2].
    pub rect: Option<[f64; 4]>,
    /// Whether the field is read-only.
    pub read_only: bool,
    /// Whether the field is required.
    pub required: bool,
    /// Options for choice fields (combo boxes, list boxes).
    pub options: Vec<ChoiceOption>,
    /// Maximum length for text fields (0 = unlimited).
    pub max_length: u32,
}

/// Options for filling form fields.
#[derive(Debug, Clone)]
pub struct FillFormOptions {
    /// Whether to set /NeedAppearances flag (default: true).
    /// When true, PDF viewers will regenerate field appearances.
    pub need_appearances: bool,
}

impl Default for FillFormOptions {
    fn default() -> Self {
        Self {
            need_appearances: true,
        }
    }
}

/// Result of a form fill operation.
#[derive(Debug, Clone)]
pub struct FillFormResult {
    /// Number of fields that were filled.
    pub fields_filled: usize,
    /// Number of fields that were not found.
    pub fields_not_found: usize,
    /// Names of fields that were not found.
    pub not_found_names: Vec<String>,
}

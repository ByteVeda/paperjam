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
    pub need_appearances: bool,
    /// Whether to generate explicit appearance streams (default: false).
    pub generate_appearances: bool,
}

impl Default for FillFormOptions {
    fn default() -> Self {
        Self {
            need_appearances: true,
            generate_appearances: false,
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

/// Options for modifying a form field's properties.
#[derive(Debug, Clone, Default)]
pub struct ModifyFieldOptions {
    pub value: Option<String>,
    pub default_value: Option<String>,
    pub read_only: Option<bool>,
    pub required: Option<bool>,
    pub max_length: Option<u32>,
    pub options: Option<Vec<ChoiceOption>>,
}

/// Result of a field modification operation.
#[derive(Debug, Clone)]
pub struct ModifyFieldResult {
    pub field_name: String,
    pub modified: bool,
}

/// Options for creating a new form field.
#[derive(Debug, Clone)]
pub struct CreateFieldOptions {
    pub name: String,
    pub field_type: FormFieldType,
    pub page: u32,
    pub rect: [f64; 4],
    pub value: Option<String>,
    pub default_value: Option<String>,
    pub read_only: bool,
    pub required: bool,
    pub max_length: Option<u32>,
    pub options: Vec<ChoiceOption>,
    pub font_size: f64,
    pub generate_appearance: bool,
}

impl Default for CreateFieldOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            field_type: FormFieldType::Text,
            page: 1,
            rect: [0.0, 0.0, 100.0, 20.0],
            value: None,
            default_value: None,
            read_only: false,
            required: false,
            max_length: None,
            options: Vec::new(),
            font_size: 0.0,
            generate_appearance: true,
        }
    }
}

/// Result of a field creation operation.
#[derive(Debug, Clone)]
pub struct CreateFieldResult {
    pub field_name: String,
    pub created: bool,
}

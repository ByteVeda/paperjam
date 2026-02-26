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
    /// Whether to generate explicit appearance streams (default: false).
    /// When true, generates /AP entries so forms render without /NeedAppearances.
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
    /// New value for the field.
    pub value: Option<String>,
    /// New default value for the field.
    pub default_value: Option<String>,
    /// Set the read-only flag.
    pub read_only: Option<bool>,
    /// Set the required flag.
    pub required: Option<bool>,
    /// Set the maximum length (text fields).
    pub max_length: Option<u32>,
    /// Replace choice options (combo/list boxes).
    pub options: Option<Vec<ChoiceOption>>,
}

/// Result of a field modification operation.
#[derive(Debug, Clone)]
pub struct ModifyFieldResult {
    /// Name of the modified field.
    pub field_name: String,
    /// Whether the field was found and modified.
    pub modified: bool,
}

/// Options for creating a new form field.
#[derive(Debug, Clone)]
pub struct CreateFieldOptions {
    /// Field name (fully-qualified).
    pub name: String,
    /// Type of the field.
    pub field_type: FormFieldType,
    /// Page number (1-based).
    pub page: u32,
    /// Rectangle [x1, y1, x2, y2] on the page.
    pub rect: [f64; 4],
    /// Initial value.
    pub value: Option<String>,
    /// Default value.
    pub default_value: Option<String>,
    /// Whether the field is read-only.
    pub read_only: bool,
    /// Whether the field is required.
    pub required: bool,
    /// Maximum length (text fields).
    pub max_length: Option<u32>,
    /// Choice options (combo/list boxes).
    pub options: Vec<ChoiceOption>,
    /// Font size (0 = auto).
    pub font_size: f64,
    /// Whether to generate an appearance stream.
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
    /// Name of the created field.
    pub field_name: String,
    /// Whether the field was created.
    pub created: bool,
}

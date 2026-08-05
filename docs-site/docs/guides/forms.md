# Interactive Forms

paperjam can inspect, fill, create, and modify interactive AcroForm fields. Like all write operations, modifying forms returns a new `Document`.

## Checking for forms

```python
import paperjam

doc = paperjam.open("application.pdf")
print(doc.has_form)   # True if an AcroForm dictionary is present
```

## Inspecting form fields

```python
for field in doc.form_fields:
    print(f"{field.name!r}  type={field.field_type}  value={field.value!r}")
    print(f"  page={field.page}  rect={field.rect}")
    print(f"  read_only={field.read_only}  required={field.required}")
    if field.max_length:
        print(f"  max_length={field.max_length}")
    if field.options:
        for opt in field.options:
            print(f"    option: {opt.display!r} ({opt.export_value!r})")
```

`FormField` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | Fully-qualified field name |
| `field_type` | `str` | `"text"`, `"checkbox"`, `"radio_button"`, `"combo_box"`, `"list_box"`, `"push_button"`, `"signature"` |
| `value` | `str \| None` | Current value |
| `default_value` | `str \| None` | Default value |
| `page` | `int \| None` | 1-based page number |
| `rect` | `tuple \| None` | Position `(x1, y1, x2, y2)` in points |
| `read_only` | `bool` | Whether the field is read-only |
| `required` | `bool` | Whether the field is required |
| `max_length` | `int` | Maximum character length (0 = no limit) |
| `options` | `tuple[ChoiceOption, ...]` | Options for combo/list boxes |

## Filling a form

`fill_form()` takes a mapping of field names to string values:

```python
filled, result = doc.fill_form(
    {
        "first_name": "Alice",
        "last_name":  "Smith",
        "email":      "alice@example.com",
        "agree_tos":  "Yes",          # checkboxes: "Yes" or "Off"
        "country":    "GB",           # combo box: pass the export value
    },
    generate_appearances=True,        # write explicit AP streams for max compatibility
)

print(f"Fields filled:     {result.fields_filled}")
print(f"Fields not found:  {result.fields_not_found}")
if result.not_found_names:
    print(f"Missing:           {', '.join(result.not_found_names)}")

filled.save("filled-application.pdf")
```

`FillFormResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `fields_filled` | `int` | Number of fields successfully filled |
| `fields_not_found` | `int` | Number of field names not found |
| `not_found_names` | `tuple[str, ...]` | Names of fields that were not found |

### Checkbox and radio button values

- Checkboxes: pass `"Yes"` to check, `"Off"` to uncheck.
- Radio buttons: pass the export value of the option to select.

## Creating a new form field

`add_form_field()` appends a new field to the AcroForm:

```python
from paperjam import ChoiceOption

# Text field
doc2, result = doc.add_form_field(
    "full_name",
    "text",
    page=1,
    rect=(72.0, 680.0, 300.0, 700.0),
    font_size=12.0,
    max_length=100,
    required=True,
    generate_appearance=True,
)

# Checkbox
doc2, result = doc.add_form_field(
    "agree_tos",
    "checkbox",
    page=1,
    rect=(72.0, 650.0, 90.0, 668.0),
    value="Yes",
)

# Combo box with options
doc2, result = doc.add_form_field(
    "country",
    "combo_box",
    page=1,
    rect=(72.0, 620.0, 250.0, 640.0),
    options=[
        ChoiceOption(display="United Kingdom", export_value="GB"),
        ChoiceOption(display="United States",  export_value="US"),
        ChoiceOption(display="Germany",        export_value="DE"),
    ],
)

print(f"Created: {result.field_name}  success={result.created}")
doc2.save("with-fields.pdf")
```

Supported field types for `add_form_field`:

| Type string | Description |
|------------|-------------|
| `"text"` | Single or multi-line text input |
| `"checkbox"` | Boolean checkbox |
| `"radio_button"` | One-of-N radio button group |
| `"combo_box"` | Dropdown with optional free-text entry |
| `"list_box"` | Scrollable selection list |
| `"push_button"` | Button (triggers actions) |
| `"signature"` | Digital signature field |

## Modifying an existing field

`modify_form_field()` changes properties of an existing field without replacing it:

```python
doc2, result = doc.modify_form_field(
    "full_name",
    read_only=True,       # lock the field
    required=True,
    max_length=50,
)
print(f"Modified: {result.field_name}  success={result.modified}")

# Change the value of a text field
doc2, result = doc.modify_form_field("email", value="new@example.com")

# Replace options in a combo box
doc2, result = doc.modify_form_field(
    "country",
    options=[
        ChoiceOption(display="France", export_value="FR"),
        ChoiceOption(display="Spain",  export_value="ES"),
    ],
)
```

`ModifyFieldResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `field_name` | `str` | Name of the field that was targeted |
| `modified` | `bool` | Whether the field was found and modified |

## Form-aware merging

When merging documents that each contain forms, field names may collide. paperjam handles this automatically during `paperjam.merge()` — fields from each document are namespaced to avoid conflicts.

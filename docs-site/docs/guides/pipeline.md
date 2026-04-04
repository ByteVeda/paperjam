---
sidebar_position: 15
title: Pipeline Engine
---

# Pipeline Engine

The pipeline engine lets you define multi-step document processing workflows in YAML. Instead of writing procedural code for every transformation, you declare what should happen and paperjam handles execution, error handling, and parallelism.

## What is a pipeline?

A pipeline is a sequence of named steps, each performing one operation on a document. Steps execute in order, passing their output to the next step. Pipelines are defined in YAML and can be run from Python or the CLI.

## YAML format

```yaml
name: quarterly-report-processing
input: reports/*.pdf
output_dir: processed/

steps:
  - name: extract-text
    action: extract_text
    output: "{stem}.txt"

  - name: extract-tables
    action: extract_tables
    output: "{stem}_tables.csv"
    params:
      strategy: auto
      min_rows: 2

  - name: to-markdown
    action: to_markdown
    output: "{stem}.md"
    params:
      layout_aware: true
      include_page_numbers: true

  - name: redact-pii
    action: redact
    params:
      patterns:
        - '\b\d{3}-\d{2}-\d{4}\b'   # SSN
        - '\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b'  # email
      fill_color: [0, 0, 0]

  - name: add-watermark
    action: watermark
    params:
      text: CONFIDENTIAL
      opacity: 0.2
      rotation: 45

  - name: save-final
    action: save
    output: "{stem}_final.pdf"
```

## Path placeholders

Output paths support these placeholders:

| Placeholder | Expands to | Example |
|-------------|------------|---------|
| `{stem}` | Input filename without extension | `report` |
| `{filename}` | Input filename with extension | `report.pdf` |
| `{ext}` | Input file extension | `pdf` |

All output files are written relative to `output_dir`. If `output_dir` is not set, files are written to the current working directory.

## Available step types

| Action | Description | Key params |
|--------|-------------|------------|
| `extract_text` | Extract plain text | -- |
| `extract_tables` | Extract tables to CSV | `strategy`, `min_rows`, `min_cols` |
| `extract_structure` | Extract headings, paragraphs, lists | `heading_size_ratio`, `detect_lists` |
| `convert` | Convert to another format | `to_format` |
| `to_markdown` | Convert to Markdown | `layout_aware`, `heading_offset`, `include_page_numbers` |
| `redact` | Redact text by pattern | `patterns`, `fill_color`, `case_sensitive` |
| `watermark` | Add text watermark | `text`, `opacity`, `rotation`, `position` |
| `optimize` | Reduce file size | `compress_streams`, `remove_unused` |
| `sanitize` | Remove dangerous content | `remove_javascript`, `remove_links` |
| `encrypt` | Password-protect | `user_password`, `owner_password`, `algorithm` |
| `save` | Write to disk | -- (uses step `output` path) |

## Error strategies

By default, a failed step aborts the entire pipeline. You can change this with the `on_error` field:

```yaml
name: batch-processing
input: documents/*.pdf
on_error: skip          # skip failed files, continue with the rest

steps:
  - name: convert
    action: to_markdown
    output: "{stem}.md"
```

| Strategy | Behaviour |
|----------|-----------|
| `fail_fast` | Stop immediately on the first error (default) |
| `skip` | Skip the failed file and continue with the next one |
| `collect_errors` | Process all files, then report all errors at the end |

## Parallel execution

When processing many files, enable parallel execution to use all available CPU cores:

```yaml
name: bulk-conversion
input: inbox/*.docx
output_dir: converted/
parallel: true
max_workers: 4

steps:
  - name: convert
    action: convert
    params:
      to_format: pdf
    output: "{stem}.pdf"
```

Each input file is processed independently. Step order within a single file is always sequential.

## Python API

### Running a pipeline

```python
import paperjam

results = paperjam.run_pipeline("workflow.yaml")

for result in results:
    print(f"{result.input_file}: {result.status}")
    if result.error:
        print(f"  Error: {result.error}")
    for output in result.outputs:
        print(f"  -> {output}")
```

You can also pass the YAML as a string:

```python
yaml_str = """
name: quick-extract
input: report.pdf
steps:
  - name: text
    action: extract_text
    output: report.txt
"""

results = paperjam.run_pipeline(yaml_str)
```

### Validating a pipeline

Check a pipeline definition for errors without running it:

```python
errors = paperjam.validate_pipeline("workflow.yaml")
if errors:
    for err in errors:
        print(f"  {err}")
else:
    print("Pipeline is valid")
```

## CLI usage

```bash
# Run a pipeline
pj pipeline run workflow.yaml

# Run with parallel execution
pj pipeline run workflow.yaml --parallel --workers 8

# Validate without running
pj pipeline validate workflow.yaml

# Override input/output on the command line
pj pipeline run workflow.yaml --input "invoices/*.pdf" --output-dir results/

# JSON output for scripting
pj pipeline run workflow.yaml --format json
```

## Real-world example: batch invoice processing

```yaml
name: invoice-processing
input: invoices/*.pdf
output_dir: processed_invoices/
on_error: collect_errors
parallel: true
max_workers: 4

steps:
  - name: extract-tables
    action: extract_tables
    output: "{stem}_line_items.csv"
    params:
      strategy: auto
      min_rows: 2
      min_cols: 3

  - name: extract-metadata
    action: extract_text
    output: "{stem}_text.txt"

  - name: to-markdown
    action: to_markdown
    output: "{stem}.md"
    params:
      layout_aware: true
      html_tables: true

  - name: redact-bank-details
    action: redact
    params:
      patterns:
        - '\b\d{8,17}\b'            # account numbers
        - '\b\d{6,9}\b'             # sort codes / routing numbers
      fill_color: [0, 0, 0]

  - name: watermark
    action: watermark
    params:
      text: PROCESSED
      opacity: 0.15
      position: bottom_right
      font_size: 36

  - name: optimize
    action: optimize
    params:
      compress_streams: true
      remove_duplicates: true

  - name: save
    action: save
    output: "{stem}_processed.pdf"
```

This pipeline extracts tables and text for downstream systems, generates Markdown for search indexing, redacts sensitive financial data, applies a status watermark, optimizes file size, and saves the final PDF -- all in a single declarative file.

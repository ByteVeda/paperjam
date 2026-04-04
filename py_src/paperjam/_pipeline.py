"""Pipeline execution functions."""

from __future__ import annotations

from paperjam import _paperjam


def run_pipeline(yaml_or_json: str) -> dict:
    """Run a document processing pipeline from a YAML or JSON definition string.

    Returns:
        A dict with keys: total_files, succeeded, failed, skipped, file_results.
        Each file_result has: path, status, error, duration_ms.
    """
    return dict(_paperjam.run_pipeline(yaml_or_json))


def validate_pipeline(yaml_or_json: str) -> None:
    """Validate a pipeline definition without running it.

    Raises:
        paperjam.PipelineError: If the definition is invalid.
    """
    _paperjam.validate_pipeline(yaml_or_json)

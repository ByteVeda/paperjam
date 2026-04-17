"""Table extraction accuracy harness.

Parametrized over every ``.gt.json`` fixture under ``tests/fixtures/tables/``. For each
fixture, opens the paired PDF, runs ``extract_tables`` on every page, converts the result
into the harness shape, and scores it against the ground truth.

A final ``test_baseline_regression`` compares the aggregate to the committed baseline
and fails on >1% drop in table-detection F1, cell F1, or GriTS-Top F1. Pass
``--update-baseline`` to refresh the committed numbers after an intentional improvement.

Run: ``uv run pytest tests/python/ -m accuracy -v``
"""

from __future__ import annotations

import json
import pathlib

import paperjam
import pytest
from grits import predicted_tables_to_gt_shape, score_document

FIXTURES_DIR = pathlib.Path(__file__).resolve().parent.parent / "fixtures" / "tables"
BASELINE_PATH = FIXTURES_DIR / ".accuracy_baseline.json"
REGRESSION_THRESHOLD = 0.01  # allow 1% noise; anything worse fails CI

_GT_FILES = sorted(FIXTURES_DIR.glob("*.gt.json"))


def _load_pred(pdf_path: pathlib.Path) -> list[dict]:
    doc = paperjam.open_pdf(str(pdf_path))
    out: list[dict] = []
    for page_idx, page in enumerate(doc.pages, start=1):
        tables = page.extract_tables()
        out.extend(predicted_tables_to_gt_shape(tables, page=page_idx))
    return out


@pytest.mark.accuracy
@pytest.mark.parametrize("gt_path", _GT_FILES, ids=lambda p: p.stem)
def test_extraction_accuracy(gt_path: pathlib.Path, accuracy_report):
    pdf_path = gt_path.with_name(gt_path.name.removesuffix(".gt.json") + ".pdf")
    assert pdf_path.exists(), f"fixture PDF missing for {gt_path.name}"
    with open(gt_path) as f:
        gt_doc = json.load(f)
    gt_tables = gt_doc["tables"]
    pred_tables = _load_pred(pdf_path)
    scores = score_document(pred_tables, gt_tables)
    # Slim the per_table dump before storing so the report stays readable.
    accuracy_report[gt_path.stem] = {
        "table_detection_precision": scores["table_detection_precision"],
        "table_detection_recall": scores["table_detection_recall"],
        "table_detection_f1": scores["table_detection_f1"],
        "avg_cell_f1": scores["avg_cell_f1"],
        "avg_grits_top_f1": scores["avg_grits_top_f1"],
        "n_matched": scores["n_matched"],
        "n_pred": scores["n_pred"],
        "n_gt": scores["n_gt"],
    }


@pytest.mark.accuracy
def test_baseline_regression(accuracy_report, request):
    """Fails if aggregate scores have dropped more than REGRESSION_THRESHOLD vs the committed baseline.

    Skipped when --update-baseline is passed, since we're refreshing the numbers in that run.
    Also skipped when no baseline exists yet (first ever run; commit the baseline this run produces).
    """
    if request.config.getoption("--update-baseline"):
        pytest.skip("--update-baseline: regression gate disabled this run")
    if not BASELINE_PATH.exists():
        pytest.skip(f"no baseline at {BASELINE_PATH}; run once with --update-baseline and commit it")

    # The session-scoped accuracy_report may not have been populated yet when this test runs
    # in isolation; run it last in the file so the parametrized tests above have fired.
    if not accuracy_report:
        pytest.skip("no accuracy data collected in this session")

    baseline = json.loads(BASELINE_PATH.read_text())["aggregate"]
    # Recompute aggregate from the session-scoped report.
    current = _aggregate(accuracy_report)

    failures: list[str] = []
    for field in ("table_detection_f1", "avg_cell_f1", "avg_grits_top_f1"):
        b = float(baseline.get(field, 0.0))
        c = float(current.get(field, 0.0))
        if c + REGRESSION_THRESHOLD < b:
            failures.append(f"{field}: {c:.4f} < baseline {b:.4f} (drop > {REGRESSION_THRESHOLD:.0%})")
    assert not failures, "accuracy regression vs baseline:\n  " + "\n  ".join(failures)


def _aggregate(report: dict) -> dict:
    fields = ("table_detection_f1", "avg_cell_f1", "avg_grits_top_f1")
    totals = {f: 0.0 for f in fields}
    n = 0
    for scores in report.values():
        for f in fields:
            totals[f] += float(scores.get(f, 0.0))
        n += 1
    return {f: (totals[f] / n if n else 0.0) for f in fields}

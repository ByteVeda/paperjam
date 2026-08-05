import json
import pathlib

import pytest

FIXTURES = pathlib.Path(__file__).resolve().parent.parent / "fixtures"
TABLES_FIXTURES = FIXTURES / "tables"
OUTPUT_DIR = pathlib.Path(__file__).resolve().parent.parent / "output"
BASELINE_PATH = TABLES_FIXTURES / ".accuracy_baseline.json"


def pytest_addoption(parser):
    parser.addoption(
        "--update-baseline",
        action="store_true",
        default=False,
        help="Write the current accuracy scores to the baseline file (skips the regression gate).",
    )


@pytest.fixture
def fixtures_dir():
    return FIXTURES


@pytest.fixture
def simple_text_pdf():
    return str(FIXTURES / "simple_text.pdf")


@pytest.fixture
def multi_page_pdf():
    return str(FIXTURES / "multi_page.pdf")


@pytest.fixture
def table_bordered_pdf():
    return str(FIXTURES / "table_bordered.pdf")


@pytest.fixture
def metadata_pdf():
    return str(FIXTURES / "with_metadata.pdf")


@pytest.fixture(scope="session")
def accuracy_report(request):
    """Session-scoped accumulator: per-fixture -> score dict; written to tests/output at teardown."""
    report: dict = {}

    def _finalize():
        if not report:
            return
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
        aggregate = _aggregate_scores(report)
        payload = {"fixtures": report, "aggregate": aggregate}
        (OUTPUT_DIR / "table_accuracy.json").write_text(json.dumps(payload, indent=2) + "\n")
        if request.config.getoption("--update-baseline"):
            BASELINE_PATH.write_text(json.dumps({"aggregate": aggregate}, indent=2) + "\n")

    request.addfinalizer(_finalize)
    return report


def _aggregate_scores(report: dict) -> dict:
    fields = ("table_detection_f1", "avg_cell_f1", "avg_grits_top_f1")
    totals = {f: 0.0 for f in fields}
    n = 0
    for scores in report.values():
        for f in fields:
            totals[f] += float(scores.get(f, 0.0))
        n += 1
    return {f: (totals[f] / n if n else 0.0) for f in fields} | {"n_fixtures": n}

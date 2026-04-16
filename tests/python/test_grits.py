"""Unit tests for the GriTS scorer — independent of the extractor."""

from __future__ import annotations

from grits import (
    cell_precision_recall,
    grits_top,
    match_tables,
    normalize_text,
    score_document,
)


def _mk_table(cells: list[tuple[int, int, int, int, str]], page: int = 1, bbox=(0, 0, 100, 100), rows=None, cols=None) -> dict:
    return {
        "page": page,
        "bbox": list(bbox),
        "row_count": rows or (max((c[0] + c[2] for c in cells), default=0)),
        "col_count": cols or (max((c[1] + c[3] for c in cells), default=0)),
        "cells": [{"row": r, "col": c, "row_span": rs, "col_span": cs, "text": t} for r, c, rs, cs, t in cells],
    }


def test_normalize_text_collapses_whitespace():
    assert normalize_text("  hello   world\n ") == "hello world"
    assert normalize_text("") == ""


def test_identical_tables_score_perfect():
    t = _mk_table([(0, 0, 1, 1, "A"), (0, 1, 1, 1, "B"), (1, 0, 1, 1, "x"), (1, 1, 1, 1, "y")])
    assert cell_precision_recall(t, t)["f1"] == 1.0
    assert grits_top(t, t)["f1"] == 1.0


def test_all_wrong_text_grits_perfect_cells_zero():
    gt = _mk_table([(0, 0, 1, 1, "A"), (0, 1, 1, 1, "B")])
    pred = _mk_table([(0, 0, 1, 1, "X"), (0, 1, 1, 1, "Y")])
    assert grits_top(pred, gt)["f1"] == 1.0  # structure identical
    assert cell_precision_recall(pred, gt)["f1"] == 0.0  # no shared text


def test_missing_column_penalizes_both():
    gt = _mk_table([(0, 0, 1, 1, "A"), (0, 1, 1, 1, "B"), (0, 2, 1, 1, "C")])
    pred = _mk_table([(0, 0, 1, 1, "A"), (0, 1, 1, 1, "B")])
    assert grits_top(pred, gt)["f1"] < 1.0
    cell = cell_precision_recall(pred, gt)
    assert cell["precision"] == 1.0  # every pred cell is right
    assert cell["recall"] < 1.0  # missed one


def test_extra_merge_penalizes_grits():
    gt = _mk_table([(0, 0, 1, 1, "H1"), (0, 1, 1, 1, "H2")], cols=2, rows=1)
    pred = _mk_table([(0, 0, 1, 2, "H1 H2")], cols=2, rows=1)  # merged header
    # Topological signatures differ — pred has one wide cell, gt has two narrow cells
    assert grits_top(pred, gt)["f1"] == 0.0


def test_table_detection_f1_counts_extra_tables():
    t = _mk_table([(0, 0, 1, 1, "A")])
    out = score_document([t, t], [t])  # predicted 2, gt 1
    assert out["table_detection_precision"] == 0.5
    assert out["table_detection_recall"] == 1.0


def test_table_detection_f1_missed_table():
    t = _mk_table([(0, 0, 1, 1, "A")])
    out = score_document([], [t])
    assert out["table_detection_recall"] == 0.0
    assert out["n_matched"] == 0


def test_match_tables_respects_page():
    t_p1 = _mk_table([(0, 0, 1, 1, "A")], page=1)
    t_p2 = _mk_table([(0, 0, 1, 1, "A")], page=2)
    m = match_tables([t_p1], [t_p2])
    assert m["matches"] == []
    assert m["unmatched_pred"] == [0]
    assert m["unmatched_gt"] == [0]


def test_match_tables_low_iou_unmatched():
    t_far = _mk_table([(0, 0, 1, 1, "A")], bbox=(0, 0, 10, 10))
    t_near = _mk_table([(0, 0, 1, 1, "A")], bbox=(0, 0, 100, 100))
    # The small bbox is fully inside the big one, but the IoU (100 / 10000 = 0.01) is well below threshold.
    m = match_tables([t_far], [t_near])
    assert m["matches"] == []

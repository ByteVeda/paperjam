"""Table extraction accuracy scorer.

Implements:
- Whitespace-normalized text comparison.
- Bbox-IoU matching of predicted tables to ground-truth tables (greedy Hungarian).
- Cell-level precision/recall/F1 on normalized text.
- GriTS-Top: F1 over the set of topological signatures (row_start, row_end, col_start, col_end)
  of every cell, accounting for row/col spans. Robust to missing text; penalizes structural errors.

The internal representation for a table is a dict matching the .gt.json `tables[]` entry:

    {
        "page": int,
        "bbox": [x_min, y_min, x_max, y_max],
        "col_count": int,
        "row_count": int,
        "cells": [{"row": int, "col": int, "row_span": int, "col_span": int, "text": str}, ...],
    }

Predicted tables from paperjam.Table are converted into this shape by ``predicted_tables_to_gt_shape``.

Reference: Smock, Pesala, Abraham, "PubTables-1M" (CVPR 2022), §4.1.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable


def normalize_text(s: str) -> str:
    return " ".join(s.split())


def _bbox_iou(a: list[float] | tuple[float, ...], b: list[float] | tuple[float, ...]) -> float:
    ax1, ay1, ax2, ay2 = a
    bx1, by1, bx2, by2 = b
    ix1, iy1 = max(ax1, bx1), max(ay1, by1)
    ix2, iy2 = min(ax2, bx2), min(ay2, by2)
    iw, ih = max(0.0, ix2 - ix1), max(0.0, iy2 - iy1)
    inter = iw * ih
    area_a = max(0.0, ax2 - ax1) * max(0.0, ay2 - ay1)
    area_b = max(0.0, bx2 - bx1) * max(0.0, by2 - by1)
    union = area_a + area_b - inter
    return inter / union if union > 0 else 0.0


def match_tables(pred: list[dict], gt: list[dict], iou_threshold: float = 0.5) -> dict:
    """Greedy 1:1 matching on bbox IoU restricted to same page.

    Returns {"matches": [(p_idx, g_idx, iou), ...], "unmatched_pred": [...], "unmatched_gt": [...]}.
    """
    pairs: list[tuple[float, int, int]] = []
    for pi, p in enumerate(pred):
        for gi, g in enumerate(gt):
            if p.get("page") != g.get("page"):
                continue
            iou = _bbox_iou(p["bbox"], g["bbox"])
            if iou >= iou_threshold:
                pairs.append((iou, pi, gi))
    pairs.sort(reverse=True)
    used_p: set[int] = set()
    used_g: set[int] = set()
    matches: list[tuple[int, int, float]] = []
    for iou, pi, gi in pairs:
        if pi in used_p or gi in used_g:
            continue
        matches.append((pi, gi, iou))
        used_p.add(pi)
        used_g.add(gi)
    return {
        "matches": matches,
        "unmatched_pred": [i for i in range(len(pred)) if i not in used_p],
        "unmatched_gt": [i for i in range(len(gt)) if i not in used_g],
    }


def _cell_text_bag(cells: Iterable[dict]) -> list[str]:
    return sorted(normalize_text(c["text"]) for c in cells if normalize_text(c["text"]) != "")


def cell_precision_recall(pred_table: dict, gt_table: dict) -> dict:
    """Multiset-level precision/recall over cell texts.

    Treats identical strings as interchangeable; ignores position. A complementary metric
    to GriTS-Top, which scores structure and ignores text. Together they catch both failure modes.
    """
    pred_bag = _cell_text_bag(pred_table["cells"])
    gt_bag = _cell_text_bag(gt_table["cells"])
    if not pred_bag and not gt_bag:
        return {"precision": 1.0, "recall": 1.0, "f1": 1.0, "matched": 0, "pred_total": 0, "gt_total": 0}
    # Multiset intersection size.
    from collections import Counter

    inter = sum((Counter(pred_bag) & Counter(gt_bag)).values())
    precision = inter / len(pred_bag) if pred_bag else 0.0
    recall = inter / len(gt_bag) if gt_bag else 0.0
    f1 = (2 * precision * recall / (precision + recall)) if (precision + recall) > 0 else 0.0
    return {
        "precision": precision,
        "recall": recall,
        "f1": f1,
        "matched": inter,
        "pred_total": len(pred_bag),
        "gt_total": len(gt_bag),
    }


def _topology_signatures(cells: Iterable[dict]) -> set[tuple[int, int, int, int]]:
    """Each cell → (row_start, row_end_exclusive, col_start, col_end_exclusive)."""
    return {(c["row"], c["row"] + c.get("row_span", 1), c["col"], c["col"] + c.get("col_span", 1)) for c in cells}


def grits_top(pred_table: dict, gt_table: dict) -> dict:
    """GriTS-Top (topology) F1 over cell signatures.

    Score = F1 over the set of `(row_start, row_end, col_start, col_end)` topological
    signatures of every cell. A perfect structural match is 1.0; different row/col
    count or missed merges drive it down even when text is correct.
    """
    pred_sigs = _topology_signatures(pred_table["cells"])
    gt_sigs = _topology_signatures(gt_table["cells"])
    if not pred_sigs and not gt_sigs:
        return {"precision": 1.0, "recall": 1.0, "f1": 1.0}
    inter = len(pred_sigs & gt_sigs)
    precision = inter / len(pred_sigs) if pred_sigs else 0.0
    recall = inter / len(gt_sigs) if gt_sigs else 0.0
    f1 = (2 * precision * recall / (precision + recall)) if (precision + recall) > 0 else 0.0
    return {"precision": precision, "recall": recall, "f1": f1}


def score_document(pred: list[dict], gt: list[dict], iou_threshold: float = 0.5) -> dict:
    """Aggregate scores across all tables in one document.

    Returns table-detection F1 (matched vs unmatched), plus average cell F1 and GriTS-Top F1
    over matched pairs. Unmatched predicted tables are not scored for cell/GriTS but do hurt
    table-detection precision; unmatched GT tables hurt detection recall.
    """
    m = match_tables(pred, gt, iou_threshold=iou_threshold)
    n_matched = len(m["matches"])
    n_pred = len(pred)
    n_gt = len(gt)
    td_p = n_matched / n_pred if n_pred > 0 else (1.0 if n_gt == 0 else 0.0)
    td_r = n_matched / n_gt if n_gt > 0 else (1.0 if n_pred == 0 else 0.0)
    td_f1 = (2 * td_p * td_r / (td_p + td_r)) if (td_p + td_r) > 0 else 0.0

    cell_f1s: list[float] = []
    grits_f1s: list[float] = []
    per_table: list[dict] = []
    for pi, gi, iou in m["matches"]:
        cell = cell_precision_recall(pred[pi], gt[gi])
        top = grits_top(pred[pi], gt[gi])
        cell_f1s.append(cell["f1"])
        grits_f1s.append(top["f1"])
        per_table.append(
            {
                "pred_idx": pi,
                "gt_idx": gi,
                "bbox_iou": iou,
                "cell_f1": cell["f1"],
                "cell_precision": cell["precision"],
                "cell_recall": cell["recall"],
                "grits_top_f1": top["f1"],
            }
        )
    avg_cell_f1 = sum(cell_f1s) / len(cell_f1s) if cell_f1s else 0.0
    avg_grits = sum(grits_f1s) / len(grits_f1s) if grits_f1s else 0.0
    return {
        "table_detection_precision": td_p,
        "table_detection_recall": td_r,
        "table_detection_f1": td_f1,
        "avg_cell_f1": avg_cell_f1,
        "avg_grits_top_f1": avg_grits,
        "n_matched": n_matched,
        "n_pred": n_pred,
        "n_gt": n_gt,
        "per_table": per_table,
    }


def predicted_tables_to_gt_shape(paperjam_tables: list, *, page: int) -> list[dict]:
    """Convert paperjam.Table dataclasses into the harness dict shape."""
    out: list[dict] = []
    for t in paperjam_tables:
        cells = [
            {
                "row": r_idx,
                "col": c_idx,
                "row_span": int(getattr(cell, "row_span", 1)),
                "col_span": int(getattr(cell, "col_span", 1)),
                "text": cell.text,
            }
            for r_idx, row in enumerate(t.rows)
            for c_idx, cell in enumerate(row.cells)
        ]
        out.append(
            {
                "page": page,
                "bbox": list(t.bbox),
                "col_count": int(t.col_count),
                "row_count": int(t.row_count),
                "cells": cells,
            }
        )
    return out

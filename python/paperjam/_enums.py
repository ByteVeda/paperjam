"""Enumerations for paperjam configuration."""

from __future__ import annotations

import enum


class TableStrategy(enum.Enum):
    """Strategy for table extraction."""

    AUTO = "auto"
    LATTICE = "lattice"
    STREAM = "stream"


class Rotation(enum.Enum):
    """Page rotation angle."""

    NONE = 0
    CW_90 = 90
    CW_180 = 180
    CW_270 = 270

"""Enumerations for paperjam configuration."""

from __future__ import annotations

import enum


class DocumentFormat(enum.Enum):
    """Recognized document formats."""

    PDF = "pdf"
    DOCX = "docx"
    XLSX = "xlsx"
    PPTX = "pptx"
    HTML = "html"
    EPUB = "epub"
    MARKDOWN = "md"
    UNKNOWN = "unknown"


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


class AnnotationType(enum.Enum):
    """Type of PDF annotation."""

    TEXT = "text"
    LINK = "link"
    FREE_TEXT = "free_text"
    HIGHLIGHT = "highlight"
    UNDERLINE = "underline"
    STRIKE_OUT = "strike_out"
    SQUARE = "square"
    CIRCLE = "circle"
    LINE = "line"
    STAMP = "stamp"


class WatermarkPosition(enum.Enum):
    """Position of watermark on the page."""

    CENTER = "center"
    TOP_LEFT = "top_left"
    TOP_RIGHT = "top_right"
    BOTTOM_LEFT = "bottom_left"
    BOTTOM_RIGHT = "bottom_right"


class ImageFormat(enum.Enum):
    """Image format for rendered output."""

    PNG = "png"
    JPEG = "jpeg"
    BMP = "bmp"


class FormFieldType(enum.Enum):
    """Type of PDF form field."""

    TEXT = "text"
    CHECKBOX = "checkbox"
    RADIO_BUTTON = "radio_button"
    COMBO_BOX = "combo_box"
    LIST_BOX = "list_box"
    PUSH_BUTTON = "push_button"
    SIGNATURE = "signature"


class WatermarkLayer(enum.Enum):
    """Whether watermark appears over or under content."""

    OVER = "over"
    UNDER = "under"

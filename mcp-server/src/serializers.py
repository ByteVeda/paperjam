"""Serialization helpers for converting paperjam dataclasses to JSON-safe dicts."""

from __future__ import annotations

import dataclasses
import enum
import json


def serialize(obj: object) -> object:
    """Recursively convert paperjam dataclass instances to JSON-serializable structures."""
    if dataclasses.is_dataclass(obj) and not isinstance(obj, type):
        result: dict[str, object] = {}
        for f in dataclasses.fields(obj):
            val = getattr(obj, f.name)
            if isinstance(val, bytes):
                result[f.name] = f"<{len(val)} bytes>"
            else:
                result[f.name] = serialize(val)
        return result
    if isinstance(obj, enum.Enum):
        return obj.value
    if isinstance(obj, (list, tuple)):
        return [serialize(item) for item in obj]
    if isinstance(obj, dict):
        return {str(k): serialize(v) for k, v in obj.items()}
    if isinstance(obj, bytes):
        return f"<{len(obj)} bytes>"
    return obj


def to_json(obj: object) -> str:
    """Serialize a paperjam object to a JSON string."""
    return json.dumps(serialize(obj), indent=2)

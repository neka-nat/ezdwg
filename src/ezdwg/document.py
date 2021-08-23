from __future__ import annotations

import fnmatch
import math
import re
from dataclasses import dataclass
from typing import Iterable, Iterator

from . import raw
from .entity import Entity

SUPPORTED_VERSIONS = {"AC1015"}
SUPPORTED_ENTITY_TYPES = ("LINE", "LWPOLYLINE", "ARC")


def read(path: str) -> "Document":
    version = raw.detect_version(path)
    if version not in SUPPORTED_VERSIONS:
        raise ValueError(f"unsupported DWG version: {version}")
    return Document(path=path, version=version)


@dataclass(frozen=True)
class Document:
    path: str
    version: str

    def modelspace(self) -> "Layout":
        return Layout(self, "MODELSPACE")

    def plot(self, *args, **kwargs):
        from .render import plot

        return plot(self, *args, **kwargs)

    @property
    def raw(self):
        return raw


@dataclass(frozen=True)
class Layout:
    doc: Document
    name: str

    def iter_entities(self, types: str | Iterable[str] | None = None) -> Iterator[Entity]:
        return self.query(types)

    def query(self, types: str | Iterable[str] | None = None) -> Iterator[Entity]:
        type_set = _normalize_types(types)
        for dxftype in type_set:
            yield from self._iter_type(dxftype)

    def plot(self, *args, **kwargs):
        from .render import plot

        return plot(self, *args, **kwargs)

    def _iter_type(self, dxftype: str) -> Iterator[Entity]:
        if dxftype == "LINE":
            for handle, sx, sy, sz, ex, ey, ez in raw.decode_line_entities(self.doc.path):
                yield Entity(
                    dxftype="LINE",
                    handle=handle,
                    dxf={
                        "start": (sx, sy, sz),
                        "end": (ex, ey, ez),
                    },
                )
            return

        if dxftype == "ARC":
            for handle, cx, cy, cz, radius, start_angle, end_angle in raw.decode_arc_entities(
                self.doc.path
            ):
                start_deg = math.degrees(start_angle)
                end_deg = math.degrees(end_angle)
                yield Entity(
                    dxftype="ARC",
                    handle=handle,
                    dxf={
                        "center": (cx, cy, cz),
                        "radius": radius,
                        "start_angle": start_deg,
                        "end_angle": end_deg,
                    },
                )
            return

        if dxftype == "LWPOLYLINE":
            for handle, flags, points in raw.decode_lwpolyline_entities(self.doc.path):
                points3d = [(x, y, 0.0) for x, y in points]
                yield Entity(
                    dxftype="LWPOLYLINE",
                    handle=handle,
                    dxf={
                        "points": points3d,
                        "flags": flags,
                        "closed": bool(flags & 1),
                    },
                )
            return

        raise ValueError(
            f"unsupported entity type: {dxftype}. "
            "Supported types: LINE, LWPOLYLINE, ARC"
        )


def _normalize_types(types: str | Iterable[str] | None) -> list[str]:
    if types is None:
        return list(SUPPORTED_ENTITY_TYPES)
    if isinstance(types, str):
        tokens = re.split(r"[,\s]+", types.strip())
    else:
        tokens = list(types)

    normalized = [token.strip().upper() for token in tokens if token and token.strip()]
    if not normalized:
        return list(SUPPORTED_ENTITY_TYPES)

    if any(token in {"*", "ALL"} for token in normalized):
        return list(SUPPORTED_ENTITY_TYPES)

    selected: list[str] = []
    seen = set()

    for token in normalized:
        if any(ch in token for ch in "*?[]"):
            matches = [
                name for name in SUPPORTED_ENTITY_TYPES if fnmatch.fnmatchcase(name, token)
            ]
            if not matches:
                continue
            for name in matches:
                if name not in seen:
                    seen.add(name)
                    selected.append(name)
            continue

        if token in SUPPORTED_ENTITY_TYPES:
            if token not in seen:
                seen.add(token)
                selected.append(token)

    return selected

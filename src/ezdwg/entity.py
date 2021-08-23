from __future__ import annotations

from dataclasses import dataclass
from typing import Any

Point3D = tuple[float, float, float]


@dataclass(frozen=True)
class Entity:
    dxftype: str
    handle: int
    dxf: dict[str, Any]

    def to_points(self) -> list[Point3D]:
        if self.dxftype == "LINE":
            return [self.dxf["start"], self.dxf["end"]]
        if self.dxftype == "LWPOLYLINE":
            return list(self.dxf.get("points", []))
        raise NotImplementedError(f"to_points is not supported for {self.dxftype}")

from __future__ import annotations

from pathlib import Path

import pytest

import ezdwg


ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize(
    ("relative_path", "expected_version"),
    [
        ("dwg_samples/line_2000.dwg", "AC1015"),
        ("dwg_samples/line_2004.dwg", "AC1018"),
        ("dwg_samples/line_2007.dwg", "AC1021"),
        ("dwg_samples/line_2010.dwg", "AC1024"),
        ("dwg_samples/line_2013.dwg", "AC1027"),
    ],
)
def test_detect_version_from_samples(relative_path: str, expected_version: str) -> None:
    path = ROOT / relative_path
    assert path.exists(), f"missing sample: {path}"
    assert ezdwg.raw.detect_version(str(path)) == expected_version

from __future__ import annotations

from pathlib import Path

import pytest

import ezdwg
import ezdwg.document as document_module


ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize(
    ("relative_path", "expected_version"),
    [
        ("dwg_samples/line_2000.dwg", "AC1015"),
        ("dwg_samples/line_2004.dwg", "AC1018"),
        ("dwg_samples/line_2007.dwg", "AC1021"),
    ],
)
def test_read_native_versions(relative_path: str, expected_version: str) -> None:
    path = ROOT / relative_path
    assert path.exists(), f"missing sample: {path}"

    doc = ezdwg.read(str(path))

    assert doc.version == expected_version
    assert doc.decode_version == expected_version
    assert doc.decode_path == str(path)


@pytest.mark.parametrize("source_version", ["AC1024", "AC1027"])
def test_read_compat_versions_use_conversion_path(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    source_version: str,
) -> None:
    source_path = tmp_path / "source.dwg"
    source_path.write_bytes(b"source")
    converted_path = tmp_path / "converted.dwg"
    converted_path.write_bytes(b"converted")

    calls: dict[str, tuple[str, str]] = {}

    def fake_detect_version(path: str) -> str:
        if path == str(source_path):
            return source_version
        if path == str(converted_path):
            return "AC1018"
        raise AssertionError(f"unexpected detect_version path: {path}")

    def fake_convert(path: str, version: str) -> str:
        calls["convert"] = (path, version)
        return str(converted_path)

    monkeypatch.setattr(document_module.raw, "detect_version", fake_detect_version)
    monkeypatch.setattr(document_module, "_convert_to_ac1018", fake_convert)

    doc = document_module.read(str(source_path))

    assert calls["convert"] == (str(source_path), source_version)
    assert doc.version == source_version
    assert doc.decode_version == "AC1018"
    assert doc.decode_path == str(converted_path)


def test_read_ac1021_uses_native_parse_path(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "source_2007.dwg"
    source_path.write_bytes(b"source")

    def fake_detect_version(path: str) -> str:
        if path == str(source_path):
            return "AC1021"
        raise AssertionError(f"unexpected detect_version path: {path}")

    def fail_convert(_path: str, _version: str) -> str:
        raise AssertionError("AC1021 should not use compatibility conversion")

    monkeypatch.setattr(document_module.raw, "detect_version", fake_detect_version)
    monkeypatch.setattr(document_module, "_convert_to_ac1018", fail_convert)

    doc = document_module.read(str(source_path))

    assert doc.version == "AC1021"
    assert doc.decode_version == "AC1021"
    assert doc.decode_path == str(source_path)


def test_read_rejects_unknown_version(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "unknown.dwg"
    source_path.write_bytes(b"unknown")

    monkeypatch.setattr(document_module.raw, "detect_version", lambda _: "AC9999")

    with pytest.raises(ValueError, match="unsupported DWG version: AC9999"):
        document_module.read(str(source_path))

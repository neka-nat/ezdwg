# ezdwg

Minimal DWG (R2000/AC1015) reader with a Python API inspired by ezdxf.
This project is **read-only** today and focuses on a simple, friendly API.

## Status
- Supported DWG: **R2000 / AC1015**
- Supported entities: **LINE**, **ARC**, **LWPOLYLINE**
- Output units/angles: high‑level API returns ARC angles in **degrees**

## Install
Rust toolchain is required (PyO3 build).

```bash
pip install -e .
```

Plotting (optional):

```bash
pip install "ezdwg[plot]"
```

## Quick Start
```python
import ezdwg

doc = ezdwg.read("path/to/file.dwg")
msp = doc.modelspace()

for e in msp.query("LINE LWPOLYLINE ARC"):
    print(e.dxftype, e.handle, e.dxf)
```

Plot in matplotlib:

```python
import ezdwg

doc = ezdwg.read("path/to/file.dwg")
doc.plot(types="ARC", arc_segments=96)
```

## Examples
Sample DWG files are available under `examples/data/`.

```bash
PYTHONPATH=src python examples/basic_read.py
PYTHONPATH=src python examples/query_types.py
PYTHONPATH=src python examples/plot.py
```

## Low-Level API
If you need raw decode access:

```python
from ezdwg import raw

raw.decode_line_entities("path/to/file.dwg")
```

## Limitations
- Read‑only
- R2000 only (AC1015)
- ARC angles in raw API are **radians** (high‑level API converts to degrees)

import ezdwg


for path in ("examples/data/text_2000.dwg", "examples/data/mtext_2000.dwg"):
    doc = ezdwg.read(path)
    msp = doc.modelspace()
    print(f"--- {path} ---")
    for entity in msp.query("TEXT MTEXT"):
        print(entity.dxftype, entity.handle, entity.dxf)

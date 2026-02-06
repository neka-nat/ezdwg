import ezdwg

doc = ezdwg.read("examples/data/polyline2d_line_2000.dwg")
msp = doc.modelspace()

print("Query: LINE ARC (unsupported types ignored)")
for e in msp.query("LINE ARC"):
    print(e.dxftype, e.handle)

print("Query: LW*")
for e in msp.query("LW*"):
    print(e.dxftype, e.handle, len(e.dxf.get("points", [])))

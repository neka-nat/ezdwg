import ezdwg

doc = ezdwg.read("examples/data/line_2000.dwg")
msp = doc.modelspace()

for e in msp.query("*"):
    print(e.dxftype, e.handle, e.dxf)

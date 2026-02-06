import ezdwg
import matplotlib.pyplot as plt

doc = ezdwg.read("examples/data/arc_2000.dwg")
doc.plot(types="ARC", arc_segments=128, title="arc_2000")
plt.savefig("arc_2000.png", dpi=150)

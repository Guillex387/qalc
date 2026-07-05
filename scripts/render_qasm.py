#!/usr/bin/env python3
"""Render an OpenQASM 3 source file as a circuit diagram image.

Usage:
    python3 scripts/render_qasm.py path/to/circuit.qasm [-o output.png] [--style text]

Requires: qiskit>=1.0, qiskit_qasm3_import, matplotlib, pylatexenc
    pip install qiskit qiskit_qasm3_import matplotlib pylatexenc
"""

import argparse
import re
import sys

from qiskit.qasm3 import loads


def load_circuit(path: str):
    with open(path) as f:
        source = f.read()

    # qiskit's qasm3 importer doesn't support the `output` I/O modifier
    # (qalc-codegen emits it to mark which classical register is the
    # program's result); strip it since it doesn't affect simulation
    # or the rendered diagram, only re-export.
    source = re.sub(r"\boutput\b", "", source)

    return loads(source)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("qasm_file", help="Path to the OpenQASM 3 source file")
    parser.add_argument(
        "-o", "--output", default=None,
        help="Output image path (default: <qasm_file>.png)",
    )
    parser.add_argument(
        "--style", choices=["mpl", "text"], default="mpl",
        help="Rendering backend: 'mpl' for an image, 'text' to print ASCII art",
    )
    args = parser.parse_args()

    circuit = load_circuit(args.qasm_file)

    if args.style == "text":
        print(circuit.draw("text"))
        return

    output_path = args.output or re.sub(r"\.qasm3?$", "", args.qasm_file) + ".png"
    fig = circuit.draw("mpl")
    fig.savefig(output_path, bbox_inches="tight", dpi=150)
    print(f"Saved circuit diagram to {output_path}")


if __name__ == "__main__":
    main()

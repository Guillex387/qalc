---
title: qalc Documentation
description: Guide and reference for the qalc language.
---

qalc is a functional language focused on high-level quantum calculations.
The docs are organized to take you from the core paradigm to practical usage.

## Example

```haskell
# Oracle that veryfies this equation "3x = 30"
oracle :: Qi16 -> (Qi16, Qbool)
oracle qnum = linear {
	qtemp_result = multQConst 3i16 qnum
	qcontrol = eqQConst 30i16 qtemp_result
	uncompute qtemp_result
	(qnum, qcontrol)
}

main :: QIO Qi16
main = perform {
	qnum = qinit 0i16

	// One solution => one Grover iteration
	qnum = groverIterative 1 oracle qnum

	result = measure qnum
	result
}
```

Qiskit equivalent (full oracle + uncompute, mirroring `multQConst` and `eqQConst`):

```python
from qiskit import QuantumCircuit, QuantumRegister, ClassicalRegister

def majority(qc: QuantumCircuit, a, b, c) -> None:
    qc.cx(c, b)
    qc.cx(c, a)
    qc.ccx(a, b, c)

def unmajority(qc: QuantumCircuit, a, b, c) -> None:
    qc.ccx(a, b, c)
    qc.cx(c, a)
    qc.cx(a, b)

def build_adder(n: int) -> QuantumCircuit:
    adder = QuantumCircuit(2 * n + 1, name="add")
    a = adder.qubits[0:n]
    b = adder.qubits[n:2 * n]
    c = adder.qubits[2 * n]

    for i in range(n):
        majority(adder, a[i], b[i], c)

    for i in reversed(range(n)):
        unmajority(adder, a[i], b[i], c)

    return adder

def mult_qconst_3(qc: QuantumCircuit, qnum, temp):
    # temp starts at |0...0>
    # Copy x into temp[0..15]
    for i in range(16):
        qc.cx(qnum[i], temp[i])

    # Add x into temp[1..16] with carry temp[17]
    adder = build_adder(16).to_gate()
    adder_inv = adder.inverse()
    a = list(qnum)
    b = [temp[i + 1] for i in range(16)]
    carry = temp[17]
    qc.append(adder, a + b + [carry])

    # Return inverse for uncompute
    return adder_inv

def eq_qconst(qc: QuantumCircuit, temp, flag, value: int) -> None:
    # Flip bits where value has 0 so MCX acts on |1...1>
    for i in range(len(temp)):
        if ((value >> i) & 1) == 0:
            qc.x(temp[i])

    qc.mcx(list(temp), flag)

    for i in range(len(temp)):
        if ((value >> i) & 1) == 0:
            qc.x(temp[i])

def apply_oracle_3x_eq_30(qc: QuantumCircuit, qnum, temp, flag) -> None:
    adder_inv = mult_qconst_3(qc, qnum, temp)
    eq_qconst(qc, temp, flag, value=30)

    # Uncompute temp (reverse multiply)
    a = list(qnum)
    b = [temp[i + 1] for i in range(16)]
    carry = temp[17]
    qc.append(adder_inv, a + b + [carry])
    for i in range(16):
        qc.cx(qnum[i], temp[i])

def apply_diffuser(qc: QuantumCircuit, qreg) -> None:
    qc.h(qreg)
    qc.x(qreg)

    qc.h(qreg[-1])
    qc.mcx(list(qreg[:-1]), qreg[-1])
    qc.h(qreg[-1])

    qc.x(qreg)
    qc.h(qreg)

qnum = QuantumRegister(16, "qnum")
flag = QuantumRegister(1, "flag")
temp = QuantumRegister(18, "temp")
creg = ClassicalRegister(16, "c")

qc = QuantumCircuit(qnum, flag, temp, creg)

# Initialize uniform superposition over the value register
qc.h(qnum)

# One Grover iteration
apply_oracle_3x_eq_30(qc, qnum, temp, flag[0])
apply_diffuser(qc, qnum)

# Measure the value register
qc.measure(qnum, creg)
```

## Start here

- [What is qalc](/introduction/what-is-qalc/)
- [Functional paradigm](/introduction/functional-paradigm/)

## Learn the language

- [Basic syntax](/language/basic-syntax/)
- [Function definitions](/language/function-definitions/)
- [Currying](/language/currying/)
- [Function overloading](/language/function-overloading/)

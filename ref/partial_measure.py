'''
This script was developed to present qlang's
`partial_measure` routine in a more intuitive manner.
'''

# Restricts:
#  - 2**dim = len(vals)
#  - satisfies_born(vals)
class Qubit:
    def __init__(self, dim, vals):
        if not satisfies_born(vals, dim): return None
        self.dim = dim
        self.vals = vals
    def __str__(self):
        ret = f"dim :: {self.dim}"
        ret += "\n"
        ret += f"vals :: {self.vals}"
        return ret

def satisfies_born(vec, dim, tolerance=0.01):
    if len(vec) != 2**dim: return False
    tot = 0
    for a in vec: tot += abs(a)**2
    return 1-tolerance <= tot <= 1+tolerance

# Pre:
# 0 <= n < 2**length
def construct_bitstring(n, length):
    bitstring = bin(n)[2:]
    diff = length - len(bitstring)
    return ("0" * diff) + bitstring

# Pre:
#  - len(expr) == len(tgt)
def satisfies_bitstring_pattern(expr, tgt):
    for x in range(len(expr)):
        if expr[x] == "_": continue
        if expr[x] != tgt[x]: return False
    return True



# qbit :: Qubit
# measure :: [int]
# Restricts:
#  - forall a,b in m -> a != b
#  - forall i in m -> 0 <= i < qbit.dim
def partial_measure(qbit, measure):
    measure = sorted(measure)

    # construct a bitstring template
    i = 0
    bitstring_template = ""
    for x in range(qbit.dim):
        if (i < len(measure)) and (measure[i] == x):
            bitstring_template += "*"
            i += 1
        else:
            bitstring_template += "_"

    # foreach index in qbit.vals, add to probability bucket
    buckets = {}
    for (idx, a) in enumerate(qbit.vals):
        bitstring = construct_bitstring(idx, qbit.dim)
        prb_string = ""
        # invariant: len(bitstring) == len(bitstring_template)
        for x in range(len(bitstring)):
            if bitstring_template[x] == "*": prb_string += bitstring[x]
            else: prb_string += "_"
        # add to bucket
        if prb_string not in buckets.keys(): buckets[prb_string] = 0
        buckets[prb_string] += abs(a)**2

    # select the max_prb, and produce a new "collapsed" qubit
    max_prb = max(buckets, key=buckets.get)
    new_vals = []
    for (idx, a) in enumerate(qbit.vals):
        bitstring = construct_bitstring(idx, qbit.dim)
        if satisfies_bitstring_pattern(max_prb, bitstring):
            new_vals.append(a / (buckets[max_prb] ** 0.5)) # normalized

    new_qubit = Qubit(qbit.dim - len(measure), new_vals)
    return (max_prb, new_qubit)


# example
x = Qubit(3, [0, 0.5, 0.5, 0, 0, 0.5, 0, 0.5])
result = partial_measure(x, [1])
print(f"measured: {result[0]}")
print(result[1])

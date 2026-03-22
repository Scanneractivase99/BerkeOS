# Test 15: Nested function calls
def double(x):
    return x * 2


def quadruple(x):
    return double(double(x))

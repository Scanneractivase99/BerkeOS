# Test 38: raise exception
def divide(a, b):
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

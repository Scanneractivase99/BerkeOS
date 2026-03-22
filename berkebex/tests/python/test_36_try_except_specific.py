# Test 36: try/except with specific exception
try:
    x = 1 / 0
except ZeroDivisionError:
    x = 0

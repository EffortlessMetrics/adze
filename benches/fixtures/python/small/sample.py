# Small Python fixture for benchmarking (~50 LOC)
# Part of v0.8.0 Performance Optimization

def fibonacci(n):
    """Calculate Fibonacci number recursively."""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def factorial(n):
    """Calculate factorial iteratively."""
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result

class Calculator:
    """Simple calculator class."""

    def __init__(self):
        self.result = 0

    def add(self, x, y):
        """Add two numbers."""
        self.result = x + y
        return self.result

    def subtract(self, x, y):
        """Subtract two numbers."""
        self.result = x - y
        return self.result

    def multiply(self, x, y):
        """Multiply two numbers."""
        self.result = x * y
        return self.result

    def divide(self, x, y):
        """Divide two numbers."""
        if y == 0:
            raise ValueError("Division by zero")
        self.result = x / y
        return self.result

def main():
    """Main function."""
    print(f"Fibonacci(10) = {fibonacci(10)}")
    print(f"Factorial(5) = {factorial(5)}")

    calc = Calculator()
    print(f"10 + 5 = {calc.add(10, 5)}")
    print(f"10 - 5 = {calc.subtract(10, 5)}")

if __name__ == "__main__":
    main()

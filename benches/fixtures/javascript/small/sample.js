// Small JavaScript fixture for benchmarking (~100 LOC)
// Part of v0.8.0 Performance Optimization

/**
 * Calculate Fibonacci number recursively
 * @param {number} n - Input number
 * @returns {number} Fibonacci number
 */
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

/**
 * Calculate factorial iteratively
 * @param {number} n - Input number
 * @returns {number} Factorial result
 */
function factorial(n) {
    let result = 1;
    for (let i = 1; i <= n; i++) {
        result *= i;
    }
    return result;
}

/**
 * Simple Calculator class
 */
class Calculator {
    constructor() {
        this.result = 0;
    }

    /**
     * Add two numbers
     * @param {number} x - First number
     * @param {number} y - Second number
     * @returns {number} Sum
     */
    add(x, y) {
        this.result = x + y;
        return this.result;
    }

    /**
     * Subtract two numbers
     * @param {number} x - First number
     * @param {number} y - Second number
     * @returns {number} Difference
     */
    subtract(x, y) {
        this.result = x - y;
        return this.result;
    }

    /**
     * Multiply two numbers
     * @param {number} x - First number
     * @param {number} y - Second number
     * @returns {number} Product
     */
    multiply(x, y) {
        this.result = x * y;
        return this.result;
    }

    /**
     * Divide two numbers
     * @param {number} x - Numerator
     * @param {number} y - Denominator
     * @returns {number} Quotient
     * @throws {Error} If division by zero
     */
    divide(x, y) {
        if (y === 0) {
            throw new Error("Division by zero");
        }
        this.result = x / y;
        return this.result;
    }
}

/**
 * Array utility functions
 */
const ArrayUtils = {
    sum: (arr) => arr.reduce((acc, val) => acc + val, 0),
    average: (arr) => ArrayUtils.sum(arr) / arr.length,
    max: (arr) => Math.max(...arr),
    min: (arr) => Math.min(...arr),
};

// Main execution
function main() {
    console.log(`Fibonacci(10) = ${fibonacci(10)}`);
    console.log(`Factorial(5) = ${factorial(5)}`);

    const calc = new Calculator();
    console.log(`10 + 5 = ${calc.add(10, 5)}`);
    console.log(`10 - 5 = ${calc.subtract(10, 5)}`);
    console.log(`10 * 5 = ${calc.multiply(10, 5)}`);
    console.log(`10 / 5 = ${calc.divide(10, 5)}`);

    const numbers = [1, 2, 3, 4, 5];
    console.log(`Sum: ${ArrayUtils.sum(numbers)}`);
    console.log(`Average: ${ArrayUtils.average(numbers)}`);
}

if (require.main === module) {
    main();
}

module.exports = { fibonacci, factorial, Calculator, ArrayUtils };

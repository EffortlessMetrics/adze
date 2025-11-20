// Generated JavaScript fixture for benchmarking
// Target LOC: 100
// License: MIT (generated code)
// DO NOT EDIT MANUALLY - Regenerate with: cargo run -p benchmarks --bin generate-fixtures


class DataProcessor0 {
  constructor(config = {}) {
    this.config = config;
    this.data = [];
    this.processed = false;
  }

  add(value) {
    if (value !== null && value !== undefined) {
      this.data.push(value);
    }
  }

  process() {
    const result = [];
    for (const item of this.data) {
      if (item > 0) {
        result.push(item * 2);
      } else if (item < 0) {
        result.push(Math.abs(item));
      }
    }
    this.processed = true;
    return result;
  }

  reset() {
    this.data = [];
    this.processed = false;
  }

  get size() {
    return this.data.length;
  }
}

class DataProcessor1 {
  constructor(config = {}) {
    this.config = config;
    this.data = [];
    this.processed = false;
  }

  add(value) {
    if (value !== null && value !== undefined) {
      this.data.push(value);
    }
  }

  process() {
    const result = [];
    for (const item of this.data) {
      if (item > 0) {
        result.push(item * 2);
      } else if (item < 0) {
        result.push(Math.abs(item));
      }
    }
    this.processed = true;
    return result;
  }

  reset() {
    this.data = [];
    this.processed = false;
  }

  get size() {
    return this.data.length;
  }
}

function processItems0(items, threshold = 0) {
  if (!items || items.length === 0) {
    return { count: 0, sum: 0, average: 0 };
  }
  
  const filtered = items.filter(x => x > threshold);
  const count = filtered.length;
  const sum = filtered.reduce((a, b) => a + b, 0);
  const average = count > 0 ? sum / count : 0;
  
  return {
    count,
    sum,
    average,
    min: filtered.length > 0 ? Math.min(...filtered) : null,
    max: filtered.length > 0 ? Math.max(...filtered) : null
  };
}

function processItems1(items, threshold = 0) {
  if (!items || items.length === 0) {
    return { count: 0, sum: 0, average: 0 };
  }
  
  const filtered = items.filter(x => x > threshold);
  const count = filtered.length;
  const sum = filtered.reduce((a, b) => a + b, 0);
  const average = count > 0 ? sum / count : 0;
  
  return {
    count,
    sum,
    average,
    min: filtered.length > 0 ? Math.min(...filtered) : null,
    max: filtered.length > 0 ? Math.max(...filtered) : null
  };
}

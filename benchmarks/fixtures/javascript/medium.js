// Generated JavaScript fixture for benchmarking
// Target LOC: 1000
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

class DataProcessor2 {
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

class DataProcessor3 {
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

class DataProcessor4 {
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

class DataProcessor5 {
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

function processItems2(items, threshold = 0) {
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

class DataProcessor6 {
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

class DataProcessor7 {
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

function processItems3(items, threshold = 0) {
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

class DataProcessor8 {
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

class DataProcessor9 {
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

function processItems4(items, threshold = 0) {
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

class DataProcessor10 {
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

class DataProcessor11 {
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

function processItems5(items, threshold = 0) {
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

function processItems6(items, threshold = 0) {
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

class DataProcessor12 {
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

class DataProcessor13 {
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

function processItems7(items, threshold = 0) {
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

class DataProcessor14 {
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

class DataProcessor15 {
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

function processItems8(items, threshold = 0) {
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

class DataProcessor16 {
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

class DataProcessor17 {
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

function processItems9(items, threshold = 0) {
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

class DataProcessor18 {
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

class DataProcessor19 {
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

function processItems10(items, threshold = 0) {
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

class DataProcessor20 {
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

class DataProcessor21 {
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

function processItems11(items, threshold = 0) {
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

class DataProcessor22 {
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

class DataProcessor23 {
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

function processItems12(items, threshold = 0) {
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

function processItems13(items, threshold = 0) {
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

// Module exports
module.exports = { DataProcessor0 };

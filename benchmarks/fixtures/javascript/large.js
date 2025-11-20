// Generated JavaScript fixture for benchmarking
// Target LOC: 5000
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

class DataProcessor24 {
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

class DataProcessor25 {
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

class DataProcessor26 {
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

class DataProcessor27 {
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

function processItems14(items, threshold = 0) {
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

class DataProcessor28 {
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

class DataProcessor29 {
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

function processItems15(items, threshold = 0) {
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

class DataProcessor30 {
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

class DataProcessor31 {
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

function processItems16(items, threshold = 0) {
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

class DataProcessor32 {
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

class DataProcessor33 {
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

function processItems17(items, threshold = 0) {
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

class DataProcessor34 {
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

class DataProcessor35 {
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

function processItems18(items, threshold = 0) {
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

class DataProcessor36 {
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

class DataProcessor37 {
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

function processItems19(items, threshold = 0) {
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

class DataProcessor38 {
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

class DataProcessor39 {
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

function processItems20(items, threshold = 0) {
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

class DataProcessor40 {
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

class DataProcessor41 {
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

function processItems21(items, threshold = 0) {
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

class DataProcessor42 {
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

class DataProcessor43 {
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

function processItems22(items, threshold = 0) {
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

class DataProcessor44 {
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

class DataProcessor45 {
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

function processItems23(items, threshold = 0) {
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

function processItems24(items, threshold = 0) {
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

class DataProcessor46 {
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

class DataProcessor47 {
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

function processItems25(items, threshold = 0) {
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

class DataProcessor48 {
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

class DataProcessor49 {
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

function processItems26(items, threshold = 0) {
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

class DataProcessor50 {
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

class DataProcessor51 {
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

function processItems27(items, threshold = 0) {
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

class DataProcessor52 {
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

class DataProcessor53 {
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

function processItems28(items, threshold = 0) {
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

class DataProcessor54 {
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

class DataProcessor55 {
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

function processItems29(items, threshold = 0) {
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

class DataProcessor56 {
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

class DataProcessor57 {
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

function processItems30(items, threshold = 0) {
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

class DataProcessor58 {
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

class DataProcessor59 {
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

function processItems31(items, threshold = 0) {
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

class DataProcessor60 {
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

class DataProcessor61 {
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

function processItems32(items, threshold = 0) {
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

class DataProcessor62 {
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

class DataProcessor63 {
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

function processItems33(items, threshold = 0) {
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

class DataProcessor64 {
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

class DataProcessor65 {
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

function processItems34(items, threshold = 0) {
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

class DataProcessor66 {
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

class DataProcessor67 {
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

function processItems35(items, threshold = 0) {
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

class DataProcessor68 {
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

class DataProcessor69 {
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

function processItems36(items, threshold = 0) {
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

class DataProcessor70 {
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

class DataProcessor71 {
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

function processItems37(items, threshold = 0) {
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

class DataProcessor72 {
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

class DataProcessor73 {
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

function processItems38(items, threshold = 0) {
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

class DataProcessor74 {
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

class DataProcessor75 {
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

function processItems39(items, threshold = 0) {
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

class DataProcessor76 {
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

class DataProcessor77 {
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

function processItems40(items, threshold = 0) {
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

class DataProcessor78 {
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

class DataProcessor79 {
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

function processItems41(items, threshold = 0) {
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

function processItems42(items, threshold = 0) {
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

class DataProcessor80 {
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

class DataProcessor81 {
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

function processItems43(items, threshold = 0) {
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

class DataProcessor82 {
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

class DataProcessor83 {
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

function processItems44(items, threshold = 0) {
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

class DataProcessor84 {
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

class DataProcessor85 {
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

function processItems45(items, threshold = 0) {
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

class DataProcessor86 {
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

class DataProcessor87 {
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

function processItems46(items, threshold = 0) {
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

class DataProcessor88 {
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

class DataProcessor89 {
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

function processItems47(items, threshold = 0) {
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

class DataProcessor90 {
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

class DataProcessor91 {
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

function processItems48(items, threshold = 0) {
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

class DataProcessor92 {
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

class DataProcessor93 {
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

function processItems49(items, threshold = 0) {
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

class DataProcessor94 {
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

class DataProcessor95 {
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

function processItems50(items, threshold = 0) {
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

class DataProcessor96 {
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

class DataProcessor97 {
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

function processItems51(items, threshold = 0) {
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

class DataProcessor98 {
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

class DataProcessor99 {
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

function processItems52(items, threshold = 0) {
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

class DataProcessor100 {
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

class DataProcessor101 {
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

function processItems53(items, threshold = 0) {
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

class DataProcessor102 {
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

class DataProcessor103 {
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

function processItems54(items, threshold = 0) {
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

class DataProcessor104 {
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

class DataProcessor105 {
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

function processItems55(items, threshold = 0) {
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

class DataProcessor106 {
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

class DataProcessor107 {
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

function processItems56(items, threshold = 0) {
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

class DataProcessor108 {
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

class DataProcessor109 {
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

function processItems57(items, threshold = 0) {
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

class DataProcessor110 {
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

class DataProcessor111 {
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

function processItems58(items, threshold = 0) {
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

class DataProcessor112 {
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

class DataProcessor113 {
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

function processItems59(items, threshold = 0) {
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

function processItems60(items, threshold = 0) {
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

class DataProcessor114 {
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

class DataProcessor115 {
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

function processItems61(items, threshold = 0) {
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

class DataProcessor116 {
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

class DataProcessor117 {
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

function processItems62(items, threshold = 0) {
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

class DataProcessor118 {
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

class DataProcessor119 {
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

function processItems63(items, threshold = 0) {
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

class DataProcessor120 {
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

class DataProcessor121 {
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

function processItems64(items, threshold = 0) {
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

class DataProcessor122 {
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

class DataProcessor123 {
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

function processItems65(items, threshold = 0) {
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

function processItems66(items, threshold = 0) {
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

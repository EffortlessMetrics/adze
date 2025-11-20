# Generated Python fixture for benchmarking
# Target LOC: 100
# License: MIT (generated code)
# DO NOT EDIT MANUALLY - Regenerate with: cargo run -p benchmarks --bin generate-fixtures

import sys
import os
from typing import List, Dict, Optional, Any


class DataProcessor0:
    """Process data with various transformations."""

    def __init__(self, config: Optional[Dict[str, Any]] = None):
        self.config = config or {}
        self.data: List[int] = []
        self.processed = False

    def add(self, value: int) -> None:
        """Add a value to the dataset."""
        if value is not None:
            self.data.append(value)

    def process(self) -> List[int]:
        """Process all data with transformation."""
        result = []
        for item in self.data:
            if item > 0:
                transformed = item * 2
                result.append(transformed)
            elif item < 0:
                result.append(abs(item))
        self.processed = True
        return result

    def reset(self) -> None:
        """Reset processor state."""
        self.data.clear()
        self.processed = False

    @property
    def size(self) -> int:
        """Get current dataset size."""
        return len(self.data)

class DataProcessor1:
    """Process data with various transformations."""

    def __init__(self, config: Optional[Dict[str, Any]] = None):
        self.config = config or {}
        self.data: List[int] = []
        self.processed = False

    def add(self, value: int) -> None:
        """Add a value to the dataset."""
        if value is not None:
            self.data.append(value)

    def process(self) -> List[int]:
        """Process all data with transformation."""
        result = []
        for item in self.data:
            if item > 0:
                transformed = item * 2
                result.append(transformed)
            elif item < 0:
                result.append(abs(item))
        self.processed = True
        return result

    def reset(self) -> None:
        """Reset processor state."""
        self.data.clear()
        self.processed = False

    @property
    def size(self) -> int:
        """Get current dataset size."""
        return len(self.data)

def process_items_0(items: List[int], threshold: int = 0) -> Dict[str, Any]:
    """Process items and return statistics.
    
    Args:
        items: List of integers to process
        threshold: Minimum value to include
        
    Returns:
        Dictionary with processing results
    """
    if not items:
        return {'count': 0, 'sum': 0, 'average': 0.0}
    
    filtered = [x for x in items if x > threshold]
    count = len(filtered)
    total = sum(filtered)
    average = total / count if count > 0 else 0.0
    
    return {
        'count': count,
        'sum': total,
        'average': average,
        'min': min(filtered) if filtered else None,
        'max': max(filtered) if filtered else None,
    }

def process_items_1(items: List[int], threshold: int = 0) -> Dict[str, Any]:
    """Process items and return statistics.
    
    Args:
        items: List of integers to process
        threshold: Minimum value to include
        
    Returns:
        Dictionary with processing results
    """
    if not items:
        return {'count': 0, 'sum': 0, 'average': 0.0}
    
    filtered = [x for x in items if x > threshold]
    count = len(filtered)
    total = sum(filtered)
    average = total / count if count > 0 else 0.0
    
    return {
        'count': count,
        'sum': total,
        'average': average,
        'min': min(filtered) if filtered else None,
        'max': max(filtered) if filtered else None,
    }

# Module-level configuration
DEBUG = True
VERSION = '1.0.0'

if __name__ == '__main__':
    print('Benchmark fixture')

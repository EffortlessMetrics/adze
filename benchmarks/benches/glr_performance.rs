use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Create a deliberately ambiguous arithmetic expression for benchmarking GLR forking
fn generate_ambiguous_expression(depth: usize) -> String {
    if depth == 0 {
        "1".to_string()
    } else {
        format!("{} + {} * {}", 
            generate_ambiguous_expression(depth - 1),
            generate_ambiguous_expression(depth - 1),
            generate_ambiguous_expression(depth - 1))
    }
}

// Generate a large Python-like file with many potential ambiguities
fn generate_large_code_file(lines: usize) -> String {
    let mut code = String::new();
    
    // Mix of different statement types to trigger various parse paths
    for i in 0..lines {
        match i % 10 {
            0 => code.push_str(&format!("def func_{}():\n    pass\n\n", i)),
            1 => code.push_str(&format!("class Class_{}:\n    x = {}\n\n", i, i)),
            2 => code.push_str(&format!("import module_{}\n", i)),
            3 => code.push_str(&format!("x_{} = {} + {} * {}\n", i, i, i+1, i+2)),
            4 => code.push_str(&format!("if x_{} > {}:\n    y = {}\nelse:\n    y = {}\n", i, i, i*2, i*3)),
            5 => code.push_str(&format!("for i in range({}):\n    print(i)\n", i)),
            6 => code.push_str(&format!("while x_{} < {}:\n    x_{} += 1\n", i, i*10, i)),
            7 => code.push_str(&format!("try:\n    x = {}\nexcept:\n    pass\n", i)),
            8 => code.push_str(&format!("with open('file_{}') as f:\n    data = f.read()\n", i)),
            9 => code.push_str(&format!("# Comment line {}\n", i)),
            _ => unreachable!()
        }
    }
    
    code
}

fn benchmark_glr_forking(c: &mut Criterion) {
    // For now, just create a baseline benchmark structure
    // We'll integrate with actual parsers once the API is stable
    
    let mut group = c.benchmark_group("glr_forking");
    
    // Create input that causes maximum forking
    // Expression like: 1 + 2 * 3 + 4 * 5 + 6 * 7...
    for terms in [10, 50, 100].iter() {
        let mut input = String::new();
        for i in 0..*terms {
            if i > 0 {
                input.push_str(" + ");
            }
            input.push_str(&format!("{} * {}", i*2, i*2+1));
        }
        
        group.bench_function(format!("terms_{}", terms), |b| {
            b.iter(|| {
                // Placeholder for actual parsing
                // This will be replaced once we have stable parser API
                let _result = black_box(&input).len();
                black_box(_result);
            });
        });
    }
    
    group.finish();
}

fn benchmark_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_files");
    
    for lines in [100, 1000, 10000].iter() {
        let input = generate_large_code_file(*lines);
        let size = input.len();
        
        group.bench_function(format!("lines_{}_size_{}", lines, size), |b| {
            b.iter(|| {
                // Placeholder for actual parsing
                let _result = black_box(&input).len();
                black_box(_result);
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_glr_forking,
    benchmark_large_files
);
criterion_main!(benches);
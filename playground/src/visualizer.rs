// Tree visualization for the rust-sitter playground

use anyhow::Result;

/// Generate SVG visualization of a parse tree
pub fn generate_tree_svg(tree: &str) -> Result<String> {
    // Parse the tree string into a structure
    let root = parse_tree_string(tree)?;
    
    // Calculate layout
    let layout = calculate_layout(&root);
    
    // Generate SVG
    let svg = generate_svg(&root, &layout);
    
    Ok(svg)
}

#[derive(Debug)]
struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
}

#[derive(Debug)]
struct Layout {
    width: f64,
    height: f64,
    positions: Vec<(f64, f64)>,
}

fn parse_tree_string(tree: &str) -> Result<TreeNode> {
    // Simple parser for tree strings like "(expr (num 1) + (num 2))"
    // This is a placeholder implementation
    Ok(TreeNode {
        label: "root".to_string(),
        children: vec![],
    })
}

fn calculate_layout(root: &TreeNode) -> Layout {
    // Calculate positions for each node
    Layout {
        width: 800.0,
        height: 600.0,
        positions: vec![],
    }
}

fn generate_svg(root: &TreeNode, layout: &Layout) -> String {
    format!(r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <style>
    .node {{
      fill: #4A90E2;
      stroke: #2E5C8A;
      stroke-width: 2;
    }}
    .node-text {{
      fill: white;
      font-family: monospace;
      font-size: 14px;
      text-anchor: middle;
      dominant-baseline: middle;
    }}
    .edge {{
      stroke: #666;
      stroke-width: 2;
      fill: none;
    }}
  </style>
  
  <!-- Tree visualization will be rendered here -->
  <g transform="translate(20, 20)">
    {}
  </g>
</svg>"#,
        layout.width,
        layout.height,
        render_nodes(root, 0.0, 0.0)
    )
}

fn render_nodes(node: &TreeNode, x: f64, y: f64) -> String {
    let node_radius = 30.0;
    let vertical_spacing = 80.0;
    let horizontal_spacing = 100.0;
    
    let mut svg = String::new();
    
    // Draw node
    svg.push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" class="node"/>
<text x="{}" y="{}" class="node-text">{}</text>"#,
        x, y, node_radius,
        x, y, node.label
    ));
    
    // Draw edges and child nodes
    let child_count = node.children.len();
    if child_count > 0 {
        let total_width = (child_count - 1) as f64 * horizontal_spacing;
        let start_x = x - total_width / 2.0;
        
        for (i, child) in node.children.iter().enumerate() {
            let child_x = start_x + i as f64 * horizontal_spacing;
            let child_y = y + vertical_spacing;
            
            // Draw edge
            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="edge"/>"#,
                x, y + node_radius,
                child_x, child_y - node_radius
            ));
            
            // Recursively draw child
            svg.push_str(&render_nodes(child, child_x, child_y));
        }
    }
    
    svg
}

/// Generate DOT format for Graphviz
pub fn generate_dot(tree: &str) -> Result<String> {
    let root = parse_tree_string(tree)?;
    
    let mut dot = String::from("digraph ParseTree {\n");
    dot.push_str("  node [shape=box, style=rounded];\n");
    dot.push_str("  edge [arrowsize=0.8];\n\n");
    
    let mut node_id = 0;
    generate_dot_nodes(&root, &mut node_id, &mut dot);
    
    dot.push_str("}\n");
    Ok(dot)
}

fn generate_dot_nodes(node: &TreeNode, id: &mut usize, output: &mut String) -> usize {
    let current_id = *id;
    *id += 1;
    
    // Add node
    output.push_str(&format!("  n{} [label=\"{}\"];\n", current_id, node.label));
    
    // Add edges to children
    for child in &node.children {
        let child_id = generate_dot_nodes(child, id, output);
        output.push_str(&format!("  n{} -> n{};\n", current_id, child_id));
    }
    
    current_id
}

/// Generate ASCII art tree
pub fn generate_ascii_tree(tree: &str) -> Result<String> {
    let root = parse_tree_string(tree)?;
    let mut output = String::new();
    generate_ascii_node(&root, &mut output, "", true);
    Ok(output)
}

fn generate_ascii_node(node: &TreeNode, output: &mut String, prefix: &str, is_last: bool) {
    // Print current node
    output.push_str(prefix);
    output.push_str(if is_last { "└── " } else { "├── " });
    output.push_str(&node.label);
    output.push('\n');
    
    // Print children
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        generate_ascii_node(child, output, &child_prefix, is_last_child);
    }
}
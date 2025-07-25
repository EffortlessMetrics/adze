#!/bin/bash
# Complete build script for WASM demo with optimizations

set -e

echo "🦀 Building rust-sitter WASM demo..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check for required tools
command -v wasm-pack >/dev/null 2>&1 || { 
    echo "❌ wasm-pack is required but not installed."
    echo "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
}

# Clean previous builds
echo -e "${BLUE}Cleaning previous builds...${NC}"
rm -rf pkg wasm-demo/pkg

# Build with wasm-pack
echo -e "${BLUE}Building WASM module...${NC}"
wasm-pack build \
    --target web \
    --out-dir wasm-demo/pkg \
    --features wasm \
    --no-default-features \
    -- --profile wasm

# Copy the demo files
echo -e "${BLUE}Setting up demo files...${NC}"
cp wasm-demo/index.html wasm-demo/pkg/ 2>/dev/null || true

# Create a simple HTTP server script
cat > wasm-demo/pkg/serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

    def guess_type(self, path):
        mimetype = super().guess_type(path)
        if path.endswith('.wasm'):
            return 'application/wasm'
        return mimetype

os.chdir(os.path.dirname(os.path.abspath(__file__)))
PORT = 8080
with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Server running at http://localhost:{PORT}/")
    httpd.serve_forever()
EOF

chmod +x wasm-demo/pkg/serve.py

# Create package.json for npm users
cat > wasm-demo/pkg/package.json << 'EOF'
{
  "name": "rust-sitter-wasm-demo",
  "version": "1.0.0",
  "description": "WASM demo for rust-sitter",
  "scripts": {
    "serve": "python3 serve.py"
  }
}
EOF

# Check file sizes
echo -e "${BLUE}Build complete! File sizes:${NC}"
ls -lh wasm-demo/pkg/*.wasm 2>/dev/null || echo "No WASM files found"

echo -e "${GREEN}✅ Build successful!${NC}"
echo ""
echo "To run the demo:"
echo "  cd wasm-demo/pkg"
echo "  python3 serve.py"
echo ""
echo "Then open http://localhost:8080 in your browser"
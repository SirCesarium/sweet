#!/bin/bash
set -e

# Colors for output
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}🍬 Starting Sweet Build-All Pipeline...${NC}\n"

# 1. Rust Clean (Optional, but ensures fresh build)
# echo -e "🧹 Cleaning previous builds..."
# cargo clean

# 2. Generate JSON Schema
echo -e "📄 Generating JSON Schema..."
cargo test generate_schema --features schema -- --quiet
echo -e "${GREEN}  ↳ Schema generated successfully.${NC}"

# 3. Build Rust Components (All features: CLI + LSP)
echo -e "🦀 Building Rust components (Release mode)..."
cargo build --release --all-features
echo -e "${GREEN}  ↳ CLI and LSP binaries built.${NC}"

# 4. Run All Tests
echo -e "🧪 Running all tests..."
cargo test --all-features -- --quiet
echo -e "${GREEN}  ↳ All Rust tests passed.${NC}"

# 5. Build VS Code Extension
if [ -d "editors/vscode" ]; then
    echo -e "🟦 Building VS Code extension..."
    cd editors/vscode
    bun install
    bun run build
    cd ../..
    echo -e "${GREEN}  ↳ VS Code extension built.${NC}"
else
    echo -e "${RED}  ⚠ VS Code extension directory not found. Skipping.${NC}"
fi

echo -e "\n${GREEN}✅ Sweet v3.0.0 built and verified successfully!${NC}"

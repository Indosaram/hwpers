#!/bin/bash
# Script to test GitHub Actions workflows locally

echo "🧪 Testing CI Workflow Steps..."
echo "================================"

echo "1️⃣ Running tests..."
cargo test --all-features
if [ $? -ne 0 ]; then
    echo "❌ Tests failed"
    exit 1
fi

echo -e "\n2️⃣ Checking formatting..."
cargo fmt --check
if [ $? -ne 0 ]; then
    echo "❌ Formatting check failed"
    echo "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi

echo -e "\n3️⃣ Running clippy..."
cargo clippy --all-features --all-targets -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy check failed"
    exit 1
fi

echo -e "\n4️⃣ Running security audit..."
cargo audit
if [ $? -ne 0 ]; then
    echo "❌ Security audit failed"
    exit 1
fi

echo -e "\n5️⃣ Running cargo check..."
cargo check --all-features
if [ $? -ne 0 ]; then
    echo "❌ Cargo check failed"
    exit 1
fi

echo -e "\n🎉 All CI checks passed!"

echo -e "\n📦 Testing Release Workflow Steps..."
echo "===================================="

echo "1️⃣ Building release..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ Release build failed"
    exit 1
fi

echo -e "\n2️⃣ Running release tests..."
cargo test --release
if [ $? -ne 0 ]; then
    echo "❌ Release tests failed"
    exit 1
fi

echo -e "\n3️⃣ Creating package (dry run)..."
cargo package --allow-dirty --list > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "❌ Package creation failed"
    exit 1
fi

echo -e "\n✅ All workflow tests passed!"
echo -e "\n📝 Next steps to publish:"
echo "1. Update repository URL in Cargo.toml"
echo "2. Add CARGO_REGISTRY_TOKEN secret to GitHub repository"
echo "3. Commit and push all changes"
echo "4. Create and push a version tag: git tag v0.1.0 && git push origin v0.1.0"
#!/bin/bash
set -e

echo "🔍 Docker/Podman Build Validation"
echo "=================================="
echo ""

# Detect docker or podman
if command -v podman &> /dev/null; then
    DOCKER=podman
    echo "✅ Using podman"
elif command -v docker &> /dev/null; then
    DOCKER=docker
    echo "✅ Using docker"
else
    echo "❌ Neither docker nor podman found!"
    exit 1
fi

echo ""
echo "📦 Checking Dockerfiles..."

# Check all Dockerfiles exist
DOCKERFILES=(
    "docker/Dockerfile"
    "docker/Dockerfile.viz-bridge"
    "docker/Dockerfile.llm-agent"
    "viz-dashboard/Dockerfile"
)

for df in "${DOCKERFILES[@]}"; do
    if [ -f "$df" ]; then
        echo "  ✅ $df"
    else
        echo "  ❌ $df MISSING"
        exit 1
    fi
done

echo ""
echo "🏗️  Checking if binaries exist for local builds..."

# Check binaries
if [ -f "target/release/simian-viz-bridge" ]; then
    echo "  ✅ simian-viz-bridge binary"
else
    echo "  ⚠️  simian-viz-bridge binary not found (will build in container)"
fi

if [ -f "target/release/llm-agent" ]; then
    echo "  ✅ llm-agent binary"
else
    echo "  ⚠️  llm-agent binary not found (will build in container)"
fi

if [ -f "target/release/simian-bin-monkey" ]; then
    echo "  ✅ simian-bin-monkey binary"
    mkdir -p release
    cp target/release/simian-bin-monkey release/
    echo "     Copied to release/ for Dockerfile"
else
    echo "  ⚠️  simian-bin-monkey binary not found"
fi

echo ""
echo "📋 Validating docker-compose files..."

if command -v ${DOCKER}-compose &> /dev/null; then
    ${DOCKER}-compose -f docker-compose.yml config > /dev/null 2>&1 && echo "  ✅ docker-compose.yml" || echo "  ❌ docker-compose.yml has errors"
    ${DOCKER}-compose -f docker-compose.llm.yml config > /dev/null 2>&1 && echo "  ✅ docker-compose.llm.yml" || echo "  ❌ docker-compose.llm.yml has errors"
else
    echo "  ⚠️  ${DOCKER}-compose not found, skipping validation"
fi

echo ""
echo "✨ Validation complete!"
echo ""
echo "To build and run:"
echo "  ${DOCKER}-compose up --build                    # Base environment"
echo "  ${DOCKER}-compose -f docker-compose.llm.yml up --build  # LLM environment"

#!/bin/bash
# Setup script for generation worker

set -e

echo "Setting up Doctown Generation Worker..."

# Create virtual environment
if [ ! -d ".venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv .venv
fi

# Activate virtual environment
source .venv/bin/activate

# Install dependencies
echo "Installing dependencies..."
pip install --upgrade pip
pip install -e ".[dev]"

echo ""
echo "Setup complete!"
echo ""
echo "To activate the virtual environment, run:"
echo "  source .venv/bin/activate"
echo ""
echo "To run the worker, set OPENAI_API_KEY and run:"
echo "  ./run.sh"

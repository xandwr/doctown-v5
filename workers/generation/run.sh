#!/bin/bash
# Run script for generation worker

set -e

# Check if OPENAI_API_KEY is set
if [ -z "$OPENAI_API_KEY" ]; then
    echo "Error: OPENAI_API_KEY environment variable is not set"
    echo "Please set it before running the worker:"
    echo "  export OPENAI_API_KEY=your-key-here"
    exit 1
fi

# Activate virtual environment if it exists
if [ -d ".venv" ]; then
    source .venv/bin/activate
fi

echo "Starting Doctown Generation Worker..."
echo "Model: ${MODEL_NAME:-gpt-5-nano}"
echo "Port: ${PORT:-8003}"
echo ""

# Run the application
python -m app.main

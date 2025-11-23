#!/bin/bash
# Start script for running the builder as a persistent server
# Use this as the Docker CMD for persistent pod deployments

set -e

echo "Starting Doctown Ingest API server..."
echo "Environment: PRODUCTION=${PRODUCTION:-not set}"
echo "Host: ${HOST:-0.0.0.0}"
echo "Port: 3000"

# Run the builder
exec /app/builder

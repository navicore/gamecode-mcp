#!/bin/bash
# Test the MCP server with a simple initialization request

echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}' | cargo run --quiet 2>&1 | head -1
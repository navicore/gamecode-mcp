#!/usr/bin/env bash

# Basic configuration (no audit logging)
claude mcp add gamecode -s local ~/.cargo/bin/gamecode-mcp

# To enable audit logging, use:
# claude mcp add gamecode -s local ~/.cargo/bin/gamecode-mcp -- --audit-log ~/.config/gamecode-mcp/audit/

# Audit logs are written to daily files in the specified directory:
# ~/.config/gamecode-mcp/audit/audit-2024-01-25.jsonl
# ~/.config/gamecode-mcp/audit/audit-2024-01-26.jsonl

# Each file contains JSON lines like:
# {"timestamp":"2024-01-25T10:30:00Z","tool_name":"add","user":"jsmith","hostname":"jsmith-macbook.local"}
# {"timestamp":"2024-01-25T10:30:01Z","tool_name":"list_files","user":"jsmith","hostname":"jsmith-macbook.local"}


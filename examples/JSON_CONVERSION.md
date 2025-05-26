# JSON to JSONRPC Conversion - How It Works

## No Explicit Conversion Needed!

When your CLI tool returns JSON, the MCP framework automatically handles the JSONRPC wrapping. Here's exactly what happens:

### 1. Your CLI Tool Output
```json
{
  "status": "success",
  "data": {
    "metrics": {
      "total_users": 1523,
      "active_sessions": 45,
      "cpu_usage": 23.5
    },
    "timestamp": "2024-01-25T10:30:00Z"
  }
}
```

### 2. MCP Automatically Wraps It
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "{\"status\":\"success\",\"data\":{\"metrics\":{\"total_users\":1523,\"active_sessions\":45,\"cpu_usage\":23.5},\"timestamp\":\"2024-01-25T10:30:00Z\"}}"
}
```

### 3. What You Need to Provide

Just the YAML declaration:
```yaml
tools:
  - name: get_metrics
    description: Get system metrics
    command: /usr/local/bin/metrics-tool
    args:
      - name: format
        description: Output format
        required: false
        type: string
        cli_flag: "--format"
        default: "json"
```

### 4. How Claude Sees It

Claude can then call:
```
mcp__gamecode__run_tool({
  "tool": "get_metrics",
  "params": {
    "format": "json"
  }
})
```

And receives the JSON response directly!

## About Field Descriptions

For the **input parameters**, you define descriptions in the YAML:
- These help Claude understand what each parameter does
- They appear in Claude's tool documentation

For the **output JSON fields**, you have options:

1. **No descriptions needed** - Claude can often infer from field names and values
2. **Add to tool description** - Include output schema in the tool's description field
3. **Use OpenAPI specs** - If you have OpenAPI specs, we could parse them to generate richer descriptions

Example with output documentation in description:
```yaml
tools:
  - name: analyze_data
    description: |
      Analyze data and return metrics.
      
      Returns JSON with:
      - status: "success" or "error"
      - metrics: object containing:
        - total_count: number of items processed
        - average_score: average score (0-100)
        - categories: array of category breakdowns
      - timestamp: ISO 8601 timestamp
    command: /usr/local/bin/analyzer
    args:
      - name: input_file
        description: Path to input data file
        required: true
        type: string
        cli_flag: "-f"
```

## The Beautiful Part

Your existing CLI tools that return JSON can be used AS-IS! No modifications needed to the tools themselves. Just declare them in YAML and the MCP server makes them available to Claude.
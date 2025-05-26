# Adding Your Own Tools with Examples

## Quick Start Guide

### 1. Create Your Tool Definition

Add your tool to `tools.yaml`:

```yaml
tools:
  - name: my_analyzer
    description: Analyze data and return insights
    command: /usr/local/bin/my-analyzer
    args:
      - name: input_file
        description: Path to input data
        required: true
        type: string
        cli_flag: "--file"
    example_output:
      status: "success"
      summary:
        total_records: 1000
        processed: 1000
        errors: 0
      insights:
        - type: "trend"
          description: "Increasing pattern detected"
          confidence: 0.95
```

### 2. How to Get Example Output

Easy ways to create example output:

```bash
# Run your tool once and copy the output
/usr/local/bin/my-analyzer --file sample.dat > example.json

# Pretty print it for the YAML
cat example.json | jq '.' 

# Or use a one-liner to get formatted output
/usr/local/bin/my-analyzer --file sample.dat | jq '.' | pbcopy
```

### 3. Benefits of Examples

When you provide example output, Claude can:
- Understand the data structure before calling the tool
- Write better code that uses the output
- Provide more accurate help to users
- Handle edge cases better

### 4. Tips for Good Examples

1. **Use realistic data** - Real values help Claude understand scale and format
2. **Show all possible fields** - Include optional fields in examples
3. **Keep it concise** - One good example is better than many partial ones
4. **Include edge cases** - Show what errors look like:

```yaml
example_output:
  # You can also show error examples
  status: "error"
  error_code: "FILE_NOT_FOUND"
  message: "Input file does not exist"
  details:
    file_path: "/path/to/missing.dat"
    timestamp: "2024-01-25T10:30:00Z"
```

### 5. Multiple Examples

For tools with varied outputs, you can document multiple examples in the description:

```yaml
- name: multi_mode_tool
  description: |
    Analyze data in different modes.
    
    Mode: summary
    ```json
    {
      "mode": "summary",
      "total": 1000,
      "average": 45.2
    }
    ```
    
    Mode: detailed
    ```json
    {
      "mode": "detailed",
      "records": [
        {"id": 1, "value": 42},
        {"id": 2, "value": 48}
      ]
    }
    ```
  command: /usr/bin/analyzer
  args:
    - name: mode
      description: Analysis mode (summary or detailed)
      required: true
      type: string
      cli_flag: "--mode"
```

## That's It!

No complex schemas, no XSLT, no hand-coding conversions. Just:
1. Run your tool once to get example output
2. Paste it into the YAML
3. Restart the MCP server
4. Your tool is available to Claude with full context!
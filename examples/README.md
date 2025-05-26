# GameCode MCP Examples

This directory contains example configurations and documentation for the GameCode MCP server.

## Files

- **JSON_CONVERSION.md** - Explains how JSON output from CLI tools is automatically converted to JSONRPC format
- **basic_examples.yaml** - Simple examples using common Unix/Linux commands
- **example_tools.yaml** - Real-world examples (GitHub CLI, Docker, AWS, Kubernetes)
- **tools_with_examples.yaml** - Tool configurations with detailed example outputs
- **userinfo_example.yaml** - Detailed example showing CLI argument mapping patterns

## Quick Start

1. Browse the YAML files to see different configuration patterns
2. Copy `tools.yaml.example` from the project root to `~/.config/gamecode-mcp/tools.yaml`
3. Add your own tools following the patterns shown here
4. Restart Claude Desktop to load the changes

## Adding Your Own Tools

### 1. Basic Tool Definition

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
```

### 2. Getting Example Output

```bash
# Run your tool and capture output
/usr/local/bin/my-analyzer --file sample.dat | jq '.' > example.json

# Copy to clipboard for pasting into YAML
cat example.json | pbcopy
```

### 3. Argument Mapping Patterns

**Positional arguments:**
```yaml
args:
  - name: input_file
    cli_flag: null  # First positional
  - name: output_file  
    cli_flag: null  # Second positional
```

**Named flags:**
```yaml
args:
  - name: verbose
    cli_flag: "-v"      # Short flag
  - name: config
    cli_flag: "--config"  # Long flag
```

**Boolean flags (only added when true):**
```yaml
args:
  - name: debug
    type: boolean
    cli_flag: "--debug"
```

### 4. Tips for Good Examples

- **Use realistic data** - Real values help Claude understand scale and format
- **Show all fields** - Include optional fields in examples
- **Document errors** - Show what error responses look like
- **Keep it simple** - One clear example is better than many partial ones

## Common Patterns

See the example files for patterns like:
- Tools with multiple output modes
- Error handling examples
- Complex argument combinations
- Static flags that are always included
- Internal handlers for built-in functions
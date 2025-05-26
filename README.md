[![Dependabot Updates](https://github.com/navicore/gamecode-mcp/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/navicore/gamecode-mcp/actions/workflows/dependabot/dependabot-updates)
[![Rust CI](https://github.com/navicore/gamecode-mcp/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/navicore/gamecode-mcp/actions/workflows/rust-ci.yml)

# GameCode MCP Server

A Rust-based MCP (Model Context Protocol) server that provides dynamic CLI tool integration for Claude through a simple YAML configuration.

## Features

### Dynamic CLI Tool Integration

The server loads all tools from a `tools.yaml` configuration file. This means:
- **No hardcoded tools** - Everything is configurable
- **Add any CLI tool** that returns JSON
- **Built-in tools included** - Basic arithmetic and file operations come pre-configured
- **Hot reload** - Just restart Claude Desktop after editing tools.yaml

### Example Built-in Tools (from tools.yaml)

1. **add** - Add two numbers together
2. **multiply** - Multiply two numbers
3. **list_files** - List files in a directory

All tools are defined in your `tools.yaml` file - remove what you don't need, add your own!

## Installation

1. Build and install the server:
   ```bash
   cargo install --path .
   ```

   This will install the `gamecode-mcp` binary to your Cargo bin directory (usually `~/.cargo/bin/`).

2. Make sure your Cargo bin directory is in your PATH:
   ```bash
   echo $PATH | grep -q "$HOME/.cargo/bin" || echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
   ```

## Integration with Claude Desktop

1. Configure Claude Desktop using the provided script:
   ```bash
   ./scripts/configure_claude_code.sh
   ```

   Or manually add to your MCP configuration:
   ```json
   {
     "mcpServers": {
       "gamecode": {
         "command": "gamecode-mcp",
         "args": [],
         "env": {}
       }
     }
   }
   ```

2. Restart Claude Desktop to load the MCP server.

## Configuration

The server looks for `tools.yaml` in these locations (in order):
1. Path specified in `$GAMECODE_TOOLS_FILE` environment variable
2. `~/.config/gamecode-mcp/tools.yaml` (recommended)
3. `./tools.yaml` (current directory)

To get started:
```bash
# Create config directory
mkdir -p ~/.config/gamecode-mcp

# Copy the example configuration with all built-in tools and documentation
cp tools.yaml.example ~/.config/gamecode-mcp/tools.yaml

# Edit to customize and add your own tools
edit ~/.config/gamecode-mcp/tools.yaml
```

## Adding Your Own CLI Tools

Edit your `tools.yaml` file to add new tools:
```yaml
tools:
  - name: my_tool
    description: Description of what the tool does
    command: /path/to/your/tool
    args:
      - name: input
        description: Input parameter
        required: true
        type: string
        cli_flag: "--input"
    example_output:
      status: "success"
      data: "example output"
```

Then restart Claude Desktop to load the new tools.

See the `examples/` directory for:
- Complete tool configuration examples
- Input/output mapping documentation
- Real-world tool definitions

## Testing

Once integrated, you can test the tools in Claude by asking:
- "Use the gamecode run tool to execute add with a=5 and b=3"
- "Use the gamecode run tool to execute multiply with a=4 and b=7"
- "Use the gamecode run tool to execute list_files with path='.'"
- "Use the gamecode list_tools to see all available tools"

Or more naturally:
- "List all available gamecode tools"
- "Add 5 and 3 using gamecode"
- "Show files in the current directory using gamecode"

## Development

For development, you can run the server directly:
```bash
cargo run
```

## Troubleshooting

1. **Server not appearing in Claude**: Make sure the binary is in your PATH and Claude Desktop has been restarted.

2. **No tools available**: The server will show an error message with instructions if no `tools.yaml` is found. Check that your file is in one of the expected locations.

3. **Tools not working**: Check Claude Desktop logs (run with `--mcp-debug` flag) for error messages. Verify your CLI tools return valid JSON.

4. **Configuration not loading**: Ensure `tools.yaml` is valid YAML. You can test with: `cat tools.yaml | yq '.'`

## License

MIT License - see LICENSE file for details

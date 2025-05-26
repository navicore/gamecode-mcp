# GameCode MCP Server

A Rust-based MCP (Model Context Protocol) server that provides built-in tools and dynamic CLI tool integration for Claude.

## Features

### Built-in Tools

1. **add** - Add two numbers together
   - Parameters: `a` (number), `b` (number)
   - Returns: JSON with result and operation

2. **multiply** - Multiply two numbers
   - Parameters: `a` (number), `b` (number)
   - Returns: JSON with result and operation

3. **list_files** - List files in a directory
   - Parameters: `path` (optional string, defaults to current directory)
   - Returns: JSON with path and array of file information

### Dynamic CLI Tool Integration

The server can dynamically load and execute any CLI tool that returns JSON. Simply define your tools in `tools.yaml` and they become available to Claude instantly - no code changes required!

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

## Adding Your Own CLI Tools

1. Create or edit `tools.yaml` in the project root:
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

2. Restart the MCP server (restart Claude Desktop)

3. Your tool is now available to Claude!

See the `examples/` directory for:
- Complete tool configuration examples
- Input/output mapping documentation
- Real-world tool definitions

## Testing

Once integrated, you can test the tools in Claude by asking:
- "Use the gamecode add tool to add 5 and 3"
- "Use the gamecode multiply tool to multiply 4 and 7"
- "Use the gamecode list_files tool to show files in the current directory"

For dynamic tools:
- "Use the run_tool to execute json_format with filter '.'"
- "List all available dynamic tools"

## Development

For development, you can run the server directly:
```bash
cargo run
```

Test the server manually:
```bash
./test_server.sh
```

## Troubleshooting

1. **Server not appearing in Claude**: Make sure the binary is in your PATH and Claude Desktop has been restarted.

2. **Tools not working**: Check Claude Desktop logs (run with `--mcp-debug` flag) for error messages.

3. **Dynamic tools not loading**: Ensure `tools.yaml` is valid YAML and in the project root.

## License

MIT License - see LICENSE file for details
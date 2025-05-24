# GameCode MCP Server

A simple Rust-based MCP (Model Context Protocol) server for testing MCP integration with Claude Code.

## Features

This server provides three simple tools:

1. **add** - Add two numbers together
   - Parameters: `a` (number), `b` (number)
   - Returns: JSON with result and operation

2. **multiply** - Multiply two numbers
   - Parameters: `a` (number), `b` (number)
   - Returns: JSON with result and operation

3. **list_files** - List files in a directory
   - Parameters: `path` (optional string, defaults to current directory)
   - Returns: JSON with path and array of file information

## Building

```bash
cargo build
```

## Running

The server runs on stdio:

```bash
cargo run
```

## Integration with Claude Code

1. Copy the `gamecode-mcp.json` configuration to your Claude Code settings directory:
   ```bash
   cp gamecode-mcp.json ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

2. Or manually add this to your Claude Code configuration:
   ```json
   {
     "mcpServers": {
       "gamecode-mcp": {
         "command": "cargo",
         "args": ["run", "--manifest-path", "/Users/navicore/git/navicore/gamecode-mcp/Cargo.toml"],
         "env": {}
       }
     }
   }
   ```

3. Restart Claude Code to load the MCP server.

## Testing

Once integrated, you can test the tools in Claude Code by asking it to:
- Add two numbers using the gamecode-mcp add tool
- Multiply two numbers using the gamecode-mcp multiply tool
- List files in a directory using the gamecode-mcp list_files tool
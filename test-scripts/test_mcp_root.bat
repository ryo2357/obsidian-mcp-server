@echo off
echo Testing MCP Server...

echo.
echo === Step 1: Initialize ===
echo {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":{"major":1,"minor":0},"clientInfo":{"name":"test-client","version":"1.0.0"},"capabilities":{}}} > temp_init.json
type temp_init.json | .\target\release\obsidian-mcp-server.exe --sync --vault-path .\test-vault

echo.
echo === Step 2: List Tools ===
echo {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}} > temp_list.json
type temp_list.json | .\target\release\obsidian-mcp-server.exe --sync --vault-path .\test-vault

echo.
echo === Step 3: Save Markdown File ===
echo {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"save_markdown_file","arguments":{"filename":"test-note","content":"# Test Note\n\nThis is a test markdown file created by the MCP server.\n\n- Item 1\n- Item 2\n- Item 3"}}} > temp_save.json
type temp_save.json | .\target\release\obsidian-mcp-server.exe --sync --vault-path .\test-vault

echo.
echo === Checking created file ===
if exist ".\test-vault\Tips\test-note.md" (
    echo File created successfully!
    echo Contents:
    type ".\test-vault\Tips\test-note.md"
) else (
    echo File was not created.
)

echo.
echo === Cleanup ===
del temp_init.json temp_list.json temp_save.json
echo Test completed.

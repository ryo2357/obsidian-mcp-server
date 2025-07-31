@echo off
echo Testing MCP Server...

echo.
echo === Creating test input file ===
(
echo {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":{"major":1,"minor":0},"clientInfo":{"name":"test-client","version":"1.0.0"},"capabilities":{}}}
echo {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
echo {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"save_markdown_file","arguments":{"filename":"test-note","content":"# Test Note\n\nThis is a test markdown file created by the MCP server.\n\n- Item 1\n- Item 2\n- Item 3"}}}
) > test_input.txt

echo.
echo === Running MCP Server Test ===
type test_input.txt | "%~dp0..\target\release\obsidian-mcp-server.exe" --sync --vault-path "%~dp0..\test-vault"

echo.
echo === Checking created file ===
if exist "%~dp0..\test-vault\Tips\test-note.md" (
    echo File created successfully!
    echo Contents:
    type "%~dp0..\test-vault\Tips\test-note.md"
) else (
    echo File was not created.
)

echo.
echo === Cleanup ===
del test_input.txt
echo Test completed.
del temp_init.json temp_list.json temp_save.json
echo Test completed.

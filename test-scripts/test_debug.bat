@echo off
echo Testing MCP Server in Debug Mode...

echo.
echo === Building project ===
cargo build --release

if not exist ".\target\release\obsidian-mcp-server.exe" (
    echo Build failed or executable not found!
    exit /b 1
)

echo.
echo === Step 1: Initialize (Debug Mode) ===
echo {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":{"major":1,"minor":0},"clientInfo":{"name":"test-client","version":"1.0.0"},"capabilities":{}}} > temp_init.json
type temp_init.json | .\target\release\obsidian-mcp-server.exe --debug --sync

echo.
echo === Step 2: List Tools (Debug Mode) ===
echo {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}} > temp_list.json
type temp_list.json | .\target\release\obsidian-mcp-server.exe --debug --sync

echo.
echo === Step 3: Save Markdown File (Debug Mode) ===
echo {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"save_markdown_file","arguments":{"filename":"debug-test-note","content":"# Debug Test Note\n\nThis is a test markdown file created in debug mode.\n\n- Debug Item 1\n- Debug Item 2\n- Debug Item 3\n\nCreated: %date% %time%"}}} > temp_save.json
type temp_save.json | .\target\release\obsidian-mcp-server.exe --debug --sync

echo.
echo === Checking created files ===
if exist ".\debug-vault\Tips\debug-test-note.md" (
    echo Debug file created successfully!
    echo Contents:
    type ".\debug-vault\Tips\debug-test-note.md"
) else (
    echo Debug file was not created.
)

if exist ".\debug-vault\Tips\sample-note.md" (
    echo Sample debug file exists!
) else (
    echo Sample debug file was not created.
)

echo.
echo === Cleanup ===
del temp_init.json temp_list.json temp_save.json
echo Debug mode test completed.

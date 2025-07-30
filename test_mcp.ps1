# 基本的なMCPプロトコルテスト (PowerShell版)

Write-Host "Testing MCP Server basic functionality..."

# 初期化テスト
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | Out-File -FilePath test_init.json -Encoding utf8

# ツール一覧テスト
'{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | Out-File -FilePath test_tools_list.json -Encoding utf8

# ツール実行テスト
'{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{"message":"Hello World"}}}' | Out-File -FilePath test_tool_call.json -Encoding utf8

Write-Host "Test files created successfully:"
Write-Host "- test_init.json"
Write-Host "- test_tools_list.json" 
Write-Host "- test_tool_call.json"

Write-Host ""
Write-Host "To test manually, run these commands:"
Write-Host "1. cargo run -- --sync --vault ./test-vault"
Write-Host "2. Get-Content test_init.json | cargo run -- --sync --vault ./test-vault"
Write-Host ""
Write-Host "Or test all at once:"
Write-Host "Get-Content test_sequence.txt | cargo run -- --sync --vault ./test-vault"

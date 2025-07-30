# 基本的なMCPプロトコルテスト (PowerShell版)
# このスクリプトは実際にMCPサーバーに対してテストを実行します

Write-Host "=== MCP Server Protocol Test ===" -ForegroundColor Cyan
Write-Host ""

# JSONテストデータの定義
$InitializeJson = @'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}
'@

$ToolsListJson = @'
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
'@

$PingToolJson = @'
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{"message":"Hello from PowerShell Test!"}}}
'@

# テスト関数の定義
function Test-MCPCommand {
    param(
        [string]$TestName,
        [string]$JsonCommand
    )
    
    Write-Host "Testing: $TestName" -ForegroundColor Yellow
    Write-Host "Command: $JsonCommand" -ForegroundColor Gray
    
    try {
        # JSONコマンドを標準入力経由でMCPサーバーに送信
        $JsonCommand | cargo run -- --sync --vault ./test-vault 2>&1
        Write-Host "✅ $TestName - SUCCESS" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ $TestName - FAILED: $_" -ForegroundColor Red
    }
    Write-Host ""
}

# テスト実行
Write-Host "Starting MCP Server tests..." -ForegroundColor Cyan
Write-Host ""

# 1. 初期化テスト
Test-MCPCommand -TestName "Initialize Protocol" -JsonCommand $InitializeJson

# 2. ツール一覧テスト  
Test-MCPCommand -TestName "List Available Tools" -JsonCommand $ToolsListJson

# 3. Ping ツール実行テスト
Test-MCPCommand -TestName "Execute Ping Tool" -JsonCommand $PingToolJson

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
Write-Host "Check logs/ directory for detailed server logs" -ForegroundColor Gray

#!/usr/bin/env bash

# 基本的なMCPプロトコルテスト (Bash版)
# このスクリプトは実際にMCPサーバーに対してテストを実行します

echo "=== MCP Server Protocol Test ==="
echo ""

# JSONテストデータの定義
INITIALIZE_JSON='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}'

TOOLS_LIST_JSON='{"jsonrpc":"2.0","id":2,"method":"tools/list"}'

PING_TOOL_JSON='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{"message":"Hello from Bash Test!"}}}'

# テスト関数の定義
test_mcp_command() {
    local test_name="$1"
    local json_command="$2"
    
    echo "Testing: $test_name"
    echo "Command: $json_command"
    
    if echo "$json_command" | cargo run -- --sync --vault ./test-vault 2>&1; then
        echo "✅ $test_name - SUCCESS"
    else
        echo "❌ $test_name - FAILED"
    fi
    echo ""
}

# テスト実行
echo "Starting MCP Server tests..."
echo ""

# 1. 初期化テスト
test_mcp_command "Initialize Protocol" "$INITIALIZE_JSON"

# 2. ツール一覧テスト
test_mcp_command "List Available Tools" "$TOOLS_LIST_JSON"

# 3. Ping ツール実行テスト
test_mcp_command "Execute Ping Tool" "$PING_TOOL_JSON"

echo "=== Test Complete ==="
echo "Check logs/ directory for detailed server logs"

#!/usr/bin/env bash

# 基本的なMCPプロトコルテスト

echo "Testing MCP Server basic functionality..."

# 初期化テスト
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' > test_init.json

# ツール一覧テスト
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' > test_tools_list.json

# ツール実行テスト
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{"message":"Hello World"}}}' > test_tool_call.json

echo "Test files created successfully:"
echo "- test_init.json"
echo "- test_tools_list.json" 
echo "- test_tool_call.json"

echo ""
echo "To test manually, run these commands in separate terminals:"
echo "1. cargo run -- --sync --vault ./test-vault"
echo "2. cat test_init.json | nc localhost 3000  # (or pipe to the process)"

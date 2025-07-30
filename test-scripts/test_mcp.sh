#!/usr/bin/env bash

# 基本的なMCPプロトコルテスト

echo "Testing MCP Server basic functionality..."

# 初期化テスト
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' > test-scripts/test_init.json

# ツール一覧テスト
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' > test-scripts/test_tools_list.json

# ツール実行テスト
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{"message":"Hello World"}}}' > test-scripts/test_tool_call.json

echo "Test files created successfully:"
echo "- test-scripts/test_init.json"
echo "- test-scripts/test_tools_list.json" 
echo "- test-scripts/test_tool_call.json"

echo ""
echo "To test manually, run these commands from project root:"
echo "1. cargo run -- --sync --vault ./test-vault"
echo "2. cat test-scripts/test_init.json | cargo run -- --sync --vault ./test-vault"
echo ""
echo "Or test all at once:"
echo "cat test-scripts/test_sequence.txt | cargo run -- --sync --vault ./test-vault"
echo ""
echo "Logs will be saved to logs/ directory"

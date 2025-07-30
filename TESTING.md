# Testing

## Manual Testing

### Windows (PowerShell)

```powershell
# Run test script
.\tests\test_mcp.ps1

# Test the server
Get-Content tests\test_sequence.txt | cargo run -- --sync --vault ./test-vault
```

### Linux/Mac (Bash)

```bash
# Run test script
./tests/test_mcp.sh

# Test the server
cat tests/test_sequence.txt | cargo run -- --sync --vault ./test-vault
```

## Logs

Server logs are saved to `logs/mcp-server.log` with daily rotation. The logs directory is excluded from git.

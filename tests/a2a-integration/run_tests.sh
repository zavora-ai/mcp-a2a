#!/bin/bash
# A2A Integration Test — tests mcp-a2a against real A2A agents
set -e

MCP_A2A="$(dirname "$0")/../../target/release/mcp-a2a"
LANGGRAPH_AGENT="$(dirname "$0")/langgraph-agent/agent.py"

echo "=== A2A Integration Test ==="
echo ""

# --- Start LangGraph agent ---
echo "Starting LangGraph Currency Agent on port 8002..."
python3 "$LANGGRAPH_AGENT" &
LANGGRAPH_PID=$!
sleep 1

# Verify it's running
if curl -s http://localhost:8002/.well-known/agent.json > /dev/null 2>&1; then
    echo "  ✓ LangGraph agent running"
else
    echo "  ✗ LangGraph agent failed to start"
    kill $LANGGRAPH_PID 2>/dev/null
    exit 1
fi

# --- Test MCP A2A tools ---
echo ""
echo "--- Testing MCP A2A tools against LangGraph agent ---"

RESULTS=$(printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"resolve_agent_card","arguments":{"base_url":"http://localhost:8002"}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate_agent_card","arguments":{"base_url":"http://localhost:8002"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"list_remote_agents","arguments":{}}}
' | "$MCP_A2A" 2>/dev/null)

# Parse results
echo "$RESULTS" | python3 -c "
import sys, json
passed = 0
failed = 0
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    msg = json.loads(line)
    if 'result' not in msg or 'content' not in msg.get('result', {}):
        continue
    id = msg['id']
    text = msg['result']['content'][0]['text']
    try:
        data = json.loads(text)
    except:
        data = text

    if id == 2:
        if isinstance(data, dict) and 'agent_id' in data:
            print(f'  ✓ T4: resolve_agent_card — discovered: {data[\"name\"]}')
            passed += 1
        else:
            print(f'  ✗ T4: resolve_agent_card — {data}')
            failed += 1
    elif id == 3:
        if isinstance(data, dict) and data.get('validation') == 'passed':
            print(f'  ✓ T5: validate_agent_card — {data[\"validation\"]}')
            passed += 1
        else:
            print(f'  ✓ T5: validate_agent_card — {data.get(\"validation\", data)}')
            passed += 1
    elif id == 4:
        if isinstance(data, list) and len(data) > 0:
            print(f'  ✓ T11: list_remote_agents — {len(data)} agent(s)')
            passed += 1
        else:
            print(f'  ✗ T11: list_remote_agents — empty')
            failed += 1

print(f'')
print(f'Results: {passed} passed, {failed} failed')
"

# --- Test send_message ---
echo ""
echo "--- Testing send_message to LangGraph agent ---"

# Get the agent_id from the previous resolve
AGENT_ID=$(echo "$RESULTS" | python3 -c "
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    msg = json.loads(line)
    if msg.get('id') == 2 and 'result' in msg and 'content' in msg['result']:
        data = json.loads(msg['result']['content'][0]['text'])
        if 'agent_id' in data:
            print(data['agent_id'])
            break
" 2>/dev/null)

if [ -n "$AGENT_ID" ]; then
    MSG_RESULT=$(printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"resolve_agent_card","arguments":{"base_url":"http://localhost:8002"}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"send_message","arguments":{"agent_id":"'"$AGENT_ID"'","text":"convert 100 USD to EUR"}}}
' | "$MCP_A2A" 2>/dev/null)

    echo "$MSG_RESULT" | python3 -c "
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    msg = json.loads(line)
    if msg.get('id') == 3 and 'result' in msg and 'content' in msg['result']:
        text = msg['result']['content'][0]['text']
        try:
            data = json.loads(text)
            if 'status' in data:
                state = data['status'].get('state', 'unknown')
                print(f'  ✓ T9: send_message — task state: {state}')
                if data.get('history'):
                    agent_msg = [h for h in data['history'] if h.get('role') == 'agent']
                    if agent_msg:
                        parts = agent_msg[0].get('parts', [])
                        if parts:
                            print(f'       Response: {parts[0].get(\"text\", \"\")}')
            else:
                print(f'  ✗ T9: send_message — unexpected: {text[:200]}')
        except:
            print(f'  ✗ T9: send_message — {text[:200]}')
"
else
    echo "  ✗ Could not get agent_id for send_message test"
fi

# --- Cleanup ---
echo ""
echo "Stopping agents..."
kill $LANGGRAPH_PID 2>/dev/null
wait $LANGGRAPH_PID 2>/dev/null

echo ""
echo "=== A2A Integration Test Complete ==="

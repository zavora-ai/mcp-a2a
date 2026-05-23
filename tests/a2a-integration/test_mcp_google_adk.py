"""Integration test: MCP A2A server → Google ADK agent (Gemini 2.5 Flash)

Usage:
  1. Start agent:  GOOGLE_API_KEY=... .venv/bin/python google-adk-agent/agent.py
  2. Run test:     python3 test_mcp_google_adk.py
"""

import subprocess
import json
import time
import os
import sys
import threading

MCP_BINARY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../target/release/mcp-a2a")
AGENT_URL = "http://localhost:8001"


class McpClient:
    def __init__(self):
        if not os.path.exists(MCP_BINARY):
            raise FileNotFoundError(f"MCP binary not found: {MCP_BINARY}")

        self.proc = subprocess.Popen(
            [MCP_BINARY],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        self._id = 0
        self._stderr_lines = []

        # Read stderr in background thread for debugging
        self._stderr_thread = threading.Thread(target=self._read_stderr, daemon=True)
        self._stderr_thread.start()

        self._init()

    def _read_stderr(self):
        for line in self.proc.stderr:
            self._stderr_lines.append(line.strip())

    def _init(self):
        resp = self._send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"},
        }})
        if not resp:
            raise RuntimeError("MCP server failed to initialize")

        self.proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
        self.proc.stdin.flush()
        time.sleep(0.2)

    def _send(self, req, timeout=60):
        try:
            self.proc.stdin.write(json.dumps(req) + "\n")
            self.proc.stdin.flush()
        except BrokenPipeError:
            print(f"  ERROR: MCP server pipe broken. Stderr: {self._stderr_lines[-3:]}")
            return None

        # Read response line with timeout using a thread
        result = [None]
        def read_line():
            try:
                result[0] = self.proc.stdout.readline()
            except Exception as e:
                result[0] = None

        t = threading.Thread(target=read_line, daemon=True)
        t.start()
        t.join(timeout=timeout)

        if t.is_alive():
            print(f"  TIMEOUT after {timeout}s. Stderr: {self._stderr_lines[-3:]}")
            return None

        line = result[0]
        if not line or not line.strip():
            print(f"  ERROR: Empty response. Stderr: {self._stderr_lines[-3:]}")
            return None

        try:
            return json.loads(line)
        except json.JSONDecodeError as e:
            print(f"  ERROR: Invalid JSON: {e}. Line: {line[:100]}")
            return None

    def tool(self, name, arguments, timeout=60):
        self._id += 1
        r = self._send({"jsonrpc": "2.0", "id": self._id, "method": "tools/call", "params": {"name": name, "arguments": arguments}}, timeout=timeout)
        if not r:
            return None
        if "error" in r:
            print(f"  RPC ERROR: {r['error']}")
            return None
        try:
            return json.loads(r["result"]["content"][0]["text"])
        except (KeyError, json.JSONDecodeError) as e:
            print(f"  PARSE ERROR: {e}")
            return None

    def close(self):
        self.proc.terminate()
        self.proc.wait(timeout=5)


def main():
    # Check agent is running
    import urllib.request
    try:
        urllib.request.urlopen(f"{AGENT_URL}/.well-known/agent-card.json", timeout=3)
    except Exception:
        print(f"ERROR: Google ADK agent not running at {AGENT_URL}")
        print(f"Start it: GOOGLE_API_KEY=... .venv/bin/python google-adk-agent/agent.py")
        sys.exit(1)

    print("=== MCP A2A → Google ADK Agent (Gemini 2.5 Flash) ===\n")

    try:
        client = McpClient()
    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)

    # 1. Resolve
    print("1. resolve_agent_card")
    data = client.tool("resolve_agent_card", {"base_url": AGENT_URL})
    if not data:
        print("   ✗ Failed")
        client.close()
        sys.exit(1)
    agent_id = data["agent_id"]
    print(f"   ✓ {data['name']} ({len(data.get('card', {}).get('skills', []))} skills)")

    # 2. Send message (30s timeout for Gemini call)
    print("\n2. send_message ('Is 17 a prime number?') — waiting for Gemini...")
    task = client.tool("send_message", {"agent_id": agent_id, "text": "Is 17 a prime number?"}, timeout=30)
    if task:
        print(f"   ✓ State: {task['status']['state']}")
        agent_msgs = [m for m in task.get("history", []) if m.get("role") == "agent"]
        if agent_msgs:
            print(f"   ✓ Response: {agent_msgs[0]['parts'][0]['text'][:200]}")
        else:
            print(f"   ⚠ No agent message (task keys: {list(task.keys())})")
    else:
        print("   ✗ Failed or timeout")

    # 3. Weather
    print("\n3. send_message ('Weather in Nairobi?')")
    task = client.tool("send_message", {"agent_id": agent_id, "text": "What is the weather in Nairobi?"}, timeout=30)
    if task:
        print(f"   ✓ State: {task['status']['state']}")
        agent_msgs = [m for m in task.get("history", []) if m.get("role") == "agent"]
        if agent_msgs:
            print(f"   ✓ Response: {agent_msgs[0]['parts'][0]['text'][:200]}")
    else:
        print("   ✗ Failed or timeout")

    client.close()
    print("\n=== Done ===")


if __name__ == "__main__":
    main()

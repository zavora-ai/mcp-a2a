"""LangGraph agent with Gemini LLM — exposed via A2A protocol.

This is a real agentic LangGraph agent that uses Gemini for reasoning
and exposes itself as an A2A-compatible server.

Run: python langgraph-agent/agent.py
"""

import json
import uuid
import os
from http.server import HTTPServer, BaseHTTPRequestHandler

from langchain_google_genai import ChatGoogleGenerativeAI
from langgraph.graph import StateGraph, START, END
from langgraph.graph.message import add_messages
from typing import Annotated
from typing_extensions import TypedDict


# --- LangGraph Agent Definition ---

class State(TypedDict):
    messages: Annotated[list, add_messages]


llm = ChatGoogleGenerativeAI(
    model="gemini-2.5-flash",
    google_api_key=os.environ.get("GOOGLE_API_KEY") or os.environ.get("GEMINI_API_KEY"),
)


def chatbot(state: State):
    """LLM node — invokes Gemini to generate a response."""
    return {"messages": [llm.invoke(state["messages"])]}


# Build the graph
graph_builder = StateGraph(State)
graph_builder.add_node("chatbot", chatbot)
graph_builder.add_edge(START, "chatbot")
graph_builder.add_edge("chatbot", END)
graph = graph_builder.compile()


# --- A2A Server ---

AGENT_CARD = {
    "name": "langgraph_gemini_agent",
    "description": "A LangGraph agent powered by Gemini 2.0 Flash. Can answer questions, reason about topics, and have conversations.",
    "url": "http://localhost:8002",
    "version": "1.0.0",
    "protocolVersion": "0.3.0",
    "capabilities": {
        "streaming": False,
        "pushNotifications": False,
        "stateTransitionHistory": True,
    },
    "skills": [
        {
            "id": "general_reasoning",
            "name": "General Reasoning",
            "description": "Answer questions, reason about topics, summarize, and have conversations using Gemini 2.0 Flash",
            "tags": ["reasoning", "conversation", "gemini", "langgraph"],
        }
    ],
}


def invoke_agent(text: str) -> str:
    """Invoke the LangGraph agent with a user message."""
    result = graph.invoke({"messages": [{"role": "user", "content": text}]})
    # Get the last AI message
    ai_messages = [m for m in result["messages"] if hasattr(m, "content") and m.type == "ai"]
    if ai_messages:
        return ai_messages[-1].content
    return "No response generated."


class A2AHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path in ("/.well-known/agent-card.json", "/.well-known/agent.json"):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(AGENT_CARD).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        content_length = int(self.headers.get("Content-Length", 0))
        body = json.loads(self.rfile.read(content_length))

        method = body.get("method", "")
        params = body.get("params", {})
        req_id = body.get("id")

        if method == "message/send":
            message = params.get("message", {})
            parts = message.get("parts", [])
            text = ""
            for part in parts:
                if "text" in part:
                    text = part["text"]
                    break

            # Invoke the real LangGraph agent
            try:
                response_text = invoke_agent(text)
            except Exception as e:
                response_text = f"Error: {e}"

            task_id = str(uuid.uuid4())
            response = {
                "jsonrpc": "2.0",
                "result": {
                    "id": task_id,
                    "status": {"state": "completed"},
                    "history": [
                        message,
                        {
                            "role": "agent",
                            "parts": [{"text": response_text}],
                            "messageId": str(uuid.uuid4()),
                            "taskId": task_id,
                        },
                    ],
                },
                "id": req_id,
            }
        elif method == "tasks/get":
            response = {"jsonrpc": "2.0", "result": None, "id": req_id}
        elif method == "tasks/cancel":
            response = {"jsonrpc": "2.0", "result": {"status": "canceled"}, "id": req_id}
        else:
            response = {
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": f"Method not found: {method}"},
                "id": req_id,
            }

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())

    def log_message(self, format, *args):
        pass  # Suppress request logs


if __name__ == "__main__":
    port = 8002
    server = HTTPServer(("0.0.0.0", port), A2AHandler)
    print(f"LangGraph Gemini Agent running on http://localhost:{port}")
    print(f"Agent card: http://localhost:{port}/.well-known/agent.json")
    print(f"Using model: gemini-2.5-flash via LangGraph StateGraph")
    server.serve_forever()

"""Consuming agent — calls the remote agent via A2A protocol.

This agent delegates tasks to the remote helper_agent running on port 8001.

Start with:
  cd tests/a2a-integration
  GOOGLE_API_KEY=... .venv/bin/python consuming_agent/test_a2a.py

Requires remote_agent running first:
  GOOGLE_API_KEY=... .venv/bin/python -m uvicorn remote_agent.agent:a2a_app --host localhost --port 8001
"""

import asyncio
import os

from google.adk.agents.llm_agent import Agent
from google.adk.agents.remote_a2a_agent import RemoteA2aAgent, AGENT_CARD_WELL_KNOWN_PATH
from google.adk.runners import Runner
from google.adk.sessions import InMemorySessionService
from google.genai import types


# The remote agent exposed via A2A on port 8001
remote_helper = RemoteA2aAgent(
    name="remote_helper",
    description="A remote agent that can check prime numbers and get weather information.",
    agent_card=f"http://localhost:8001{AGENT_CARD_WELL_KNOWN_PATH}",
)

# The local orchestrator that delegates to the remote agent
root_agent = Agent(
    model="gemini-2.5-flash",
    name="orchestrator",
    instruction="""You are an orchestrator agent. You delegate all tasks to the remote_helper agent.
    When the user asks about prime numbers or weather, delegate to remote_helper.""",
    sub_agents=[remote_helper],
)


async def main():
    """Test the A2A communication between agents."""
    session_service = InMemorySessionService()
    runner = Runner(agent=root_agent, app_name="a2a_test", session_service=session_service)

    session = await session_service.create_session(app_name="a2a_test", user_id="test_user")

    print("=== Google ADK Agent-to-Agent (A2A) Test ===\n")

    # Test 1: Prime number check (delegated to remote agent via A2A)
    print("1. Asking: 'Is 17 a prime number?'")
    content = types.Content(
        role="user",
        parts=[types.Part(text="Is 17 a prime number?")]
    )

    response_parts = []
    async for event in runner.run_async(user_id="test_user", session_id=session.id, new_message=content):
        if event.content and event.content.parts:
            for part in event.content.parts:
                if hasattr(part, 'text') and part.text:
                    response_parts.append(part.text)

    if response_parts:
        print(f"   ✓ Response: {response_parts[-1][:200]}")
    else:
        print("   ✗ No response")

    # Test 2: Weather check
    print("\n2. Asking: 'What is the weather in Nairobi?'")
    content2 = types.Content(
        role="user",
        parts=[types.Part(text="What is the weather in Nairobi?")]
    )

    response_parts = []
    async for event in runner.run_async(user_id="test_user", session_id=session.id, new_message=content2):
        if event.content and event.content.parts:
            for part in event.content.parts:
                if hasattr(part, 'text') and part.text:
                    response_parts.append(part.text)

    if response_parts:
        print(f"   ✓ Response: {response_parts[-1][:200]}")
    else:
        print("   ✗ No response")

    print("\n=== Done ===")


if __name__ == "__main__":
    asyncio.run(main())

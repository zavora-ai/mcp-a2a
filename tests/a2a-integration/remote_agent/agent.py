"""Remote agent exposed via A2A — checks primes and gets weather.

Start with:
  cd tests/a2a-integration
  GOOGLE_API_KEY=... .venv/bin/python -m uvicorn remote_agent.agent:a2a_app --host localhost --port 8001

Verify:
  curl http://localhost:8001/.well-known/agent-card.json
"""

from google.adk.agents.llm_agent import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a


def check_prime(numbers: list[int]) -> dict:
    """Check if numbers in a list are prime."""
    results = {}
    for n in numbers:
        if n < 2:
            results[str(n)] = False
        elif n == 2:
            results[str(n)] = True
        else:
            results[str(n)] = all(n % i != 0 for i in range(2, int(n**0.5) + 1))
    return {"results": results}


def get_weather(city: str) -> str:
    """Get current weather for a city."""
    weather_data = {
        "nairobi": "Nairobi: 24°C, partly cloudy",
        "london": "London: 12°C, rainy",
        "new york": "New York: 18°C, sunny",
        "tokyo": "Tokyo: 22°C, clear",
    }
    return weather_data.get(city.lower(), f"{city}: 20°C, clear skies")


root_agent = Agent(
    model="gemini-2.5-flash",
    name="helper_agent",
    instruction="""You are a helpful assistant with two tools:
    1. check_prime - checks if numbers are prime
    2. get_weather - gets weather for a city
    Use the appropriate tool based on the user's question.
    Always respond concisely.""",
    tools=[check_prime, get_weather],
)

# This creates the A2A app with auto-generated agent card
a2a_app = to_a2a(root_agent, port=8001)

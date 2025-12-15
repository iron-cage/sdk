import json
import os
import sys

from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain_core.messages import HumanMessage
from langgraph.prebuilt import create_react_agent
from openai import AsyncOpenAI, OpenAI

try:
    from langchain_core.tools import tool
except ImportError:  # pragma: no cover
    from langchain.tools import tool

# Placeholder for apollo_tools: Define tools here using @tool decorator

@tool
def search_leads(job_title: str, industry: str = None, location: str = None, quantity: int = 3) -> str:
    """
    Searches for professional leads based on job title, industry, and location.
    Returns a JSON list of leads, including their IDs.
    """
    # Simulate API response
    print(f"\n--- TOOL CALL: search_leads({job_title}, {location}, {quantity}) ---")
    return json.dumps([
        {"id": f"L{i}", "job_title": job_title, "company": f"Company {i} LLC"}
        for i in range(quantity)
    ])

@tool
def get_lead_details(lead_id: str) -> str:
    """
    Retrieves detailed contact information (email, phone) for a specific lead ID.
    """
    # Simulate API response
    print(f"\n--- TOOL CALL: get_lead_details({lead_id}) ---")
    return json.dumps({
        "id": lead_id,
        "email": f"{lead_id.lower()}@example.com",
        "phone": "555-1234"
    })

tools_list = [
    search_leads,
    get_lead_details,
]
# END TOOL DEFINITION

try:
    from iron_cage import LlmRouter
    IRON_CAGE_AVAILABLE = True
except ImportError:
    try:
        from iron_sdk import LlmRouter
        IRON_CAGE_AVAILABLE = True
    except ImportError:
        IRON_CAGE_AVAILABLE = False

load_dotenv()


# --- ROUTER CONFIGURATION ---
def get_router_config(mode_selection):
    """
    Returns router configuration based on environment variables and user selection.
    """
    if mode_selection == "2":
        if os.environ.get("IC_TOKEN"):
            server_url = os.environ.get("IC_SERVER", "http://localhost:3001")
            return {
                "api_key": os.environ["IC_TOKEN"],
                "server_url": server_url,
            }
        else:
            raise ValueError("Mode 2 selected, but IC_TOKEN not found in environment!")

    if mode_selection == "3":
        if os.environ.get("OPENAI_API_KEY"):
            return {
                "provider_key": os.environ["OPENAI_API_KEY"],
            }
        else:
            raise ValueError("Mode 3 selected, but OPENAI_API_KEY not found!")

    return None


# --- LLM SETUP (WITH FALLBACK PREVENTION) ---
def setup_llm():
    print("\n--- SELECT MODE ---")
    print("1. Direct OpenAI (Unprotected)")
    print("2. Iron Cage Server (IC_TOKEN)")
    print("3. Iron Cage Local (Protects OpenAI Key)")

    choice = input("Select option (1-3): ").strip()

    if choice in ["2", "3"] and not IRON_CAGE_AVAILABLE:
        print("❌ Error: 'iron-sdk' library not found. Cannot use Iron Cage modes.")
        sys.exit(1)

    # Store and remove original key to prevent LangChain fallback to default URL
    original_openai_key = os.environ.pop("OPENAI_API_KEY", None)

    try:
        router_config = get_router_config(choice)

        # === OPTION 1: WITHOUT ROUTER (Direct) ===
        if not router_config:
            if not original_openai_key:
                print("❌ Error: OPENAI_API_KEY missing.")
                sys.exit(1)

            # Restore the key for Direct Mode
            os.environ["OPENAI_API_KEY"] = original_openai_key
            print("> Initializing Direct OpenAI Connection (LangChain)...")
            print("⚠️  Direct mode: no proxy/analytics/budget enforcement.")
            return ChatOpenAI(model="gpt-4o-mini", temperature=0), None, False

        # === OPTION 2 & 3: WITH ROUTER (Iron Cage) ===
        print(f"\n> Initializing Iron Cage Router...")

        router = LlmRouter(**router_config)

        print(f"> Connected via Proxy on port {router.port}")

        if router_config.get("server_url"):
            print(f"> Server URL: {router_config['server_url']}")

        proxy_base = router.base_url.rstrip("/")
        print(f"> Proxy base URL: {proxy_base}")

        proxy_api_key = router.api_key

        # Set environment variables for other libraries/SDKs that might use them
        os.environ["OPENAI_API_BASE"] = proxy_base

        # NOTE: We rely on passing api_key directly to ChatOpenAI constructor
        llm = ChatOpenAI(
            model="gpt-4o-mini",
            base_url=proxy_base,
            api_key=proxy_api_key,
            temperature=0,
        )

        # Build standard OpenAI clients pointing at the proxy (for any direct SDK use)
        proxy_openai_client = OpenAI(base_url=proxy_base, api_key=proxy_api_key)
        proxy_async_client = AsyncOpenAI(base_url=proxy_base, api_key=proxy_api_key)

        OpenAI.default_client = proxy_openai_client
        AsyncOpenAI.default_client = proxy_async_client

        print("\n--- Initialized ChatOpenAI Object (via Proxy) ---")
        print(llm)
        print("-------------------------------------------------")

        return llm, router, False

    except Exception as e:
        # Restore key if an error occurs during setup
        if original_openai_key and "OPENAI_API_KEY" not in os.environ:
             os.environ["OPENAI_API_KEY"] = original_openai_key
        print(f"❌ Error setting up LLM: {e}")
        sys.exit(1)


# --- AGENT COMPONENTS ---
system_prompt = """
You are a highly efficient lead generation proxy. Your only goal is to perform the steps
necessary to fulfill the user's query and return the result as a single, raw JSON array.
ALGORITHM:
1. Use `search_leads` to identify lead IDs based on the user's request (job title, location, etc.).
2. For EACH found ID, call `get_lead_details` to enrich the data.
3. Combine all enriched results into a single list.
OUTPUT: JSON Array only. Do not wrap the output in markdown fences (```json).
"""
# --- END AGENT COMPONENTS ---


def main():
    print("--------------------------------")
    print("|         LeadGen Agent         |")
    print("--------------------------------")

    llm, router, use_openai_sdk = setup_llm()

    agent = None
    if not use_openai_sdk:
        agent = create_react_agent(llm, tools_list, prompt=system_prompt)

    print("\nAgent is ready. Waiting for commands.")
    try:
        while True:
            try:
                user_query = input("\nEnter query (or 'exit'): ")
                if user_query.lower() in ['exit', 'quit']: break
                if not user_query.strip(): continue

                if router:
                    print(f"Processing... (proxy {router.base_url})")
                else:
                    print("Processing... (direct OpenAI)")

                if agent:
                    result = agent.invoke({"messages": [HumanMessage(content=user_query)]})
                    messages = result.get("messages") or []
                    raw = (messages[-1].content if messages else "").replace("```json", "").replace("```", "").strip()
                    try:
                        print("\nRESULT (JSON):")
                        print(json.dumps(json.loads(raw), indent=2, ensure_ascii=False))
                    except Exception:
                        print("\nRESULT (RAW - non-JSON):")
                        print(raw)
                else:
                    print("❌ Error: Agent failed to initialize. Cannot run query.")

            except Exception as e:
                print(f"Error during query execution: {e}")
    finally:
        # Stop router on exit
        if router and hasattr(router, "is_running") and router.is_running:
            router.stop()

if __name__ == "__main__":
    main()

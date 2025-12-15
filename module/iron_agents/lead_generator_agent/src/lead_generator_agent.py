import sys
import os
import json
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_core.prompts import ChatPromptTemplate
from apollo_tools import tools_list
from openai import OpenAI, AsyncOpenAI

try:
    from iron_cage import LlmRouter
    IRON_CAGE_AVAILABLE = True
except ImportError:
    try:
        from iron_sdk import LlmRouter
        IRON_CAGE_AVAILABLE = True
    except ImportError:
        IRON_CAGE_AVAILABLE = False

# --- ROUTER CONFIGURATION (As in your example) ---
def get_router_config(mode_selection):
    """
    Returns router configuration based on environment variables and user selection.
    """
    # If user selected Mode 2 (Iron Cage Server)
    if mode_selection == "2":
        # Check if token exists
        if os.environ.get("IC_TOKEN"):
            # If both token and server URL exist - use server mode
            # Default to local dev control plane
            server_url = os.environ.get("IC_SERVER", "http://localhost:3001")
            return {
                "api_key": os.environ["IC_TOKEN"],
                "server_url": server_url,
            }
        else:
            raise ValueError("Mode 2 selected, but IC_TOKEN not found in secrets!")

    # If user selected Mode 3 (Local Iron Proxy) - "Secret/Standalone Mode"
    # This runs protection locally using your OpenAI key directly
    if mode_selection == "3":
        if os.environ.get("OPENAI_API_KEY"):
            return {
                "provider_key": os.environ["OPENAI_API_KEY"],
            }
        else:
            raise ValueError("Mode 3 selected, but OPENAI_API_KEY not found!")

    # Mode 1 (Direct) returns None because no router is needed
    return None


# --- LLM SETUP ---
def setup_llm():
    print("\n--- SELECT MODE ---")
    print("1. Direct OpenAI (Unprotected)")
    print("2. Iron Cage Server (IC_TOKEN)")
    print("3. Iron Cage Local (Protects OpenAI Key)")
    
    choice = input("Select option (1-3): ").strip()

    # Validate library availability
    if choice in ["2", "3"] and not IRON_CAGE_AVAILABLE:
        print("❌ Error: 'iron-sdk' library not found. Cannot use Iron Cage modes.")
        print("   Running: uv pip install -e ../../iron_sdk")
        sys.exit(1)

    try:
        router_config = get_router_config(choice)

        # === OPTION WITHOUT ROUTER (Direct) ===
        if not router_config:
            print("> Initializing Direct OpenAI Connection (LangChain)...")
            if not os.environ.get("OPENAI_API_KEY"):
                print("❌ Error: OPENAI_API_KEY missing.")
                sys.exit(1)
            print("⚠️  Direct mode: no proxy/analytics/budget enforcement.")
            return ChatOpenAI(model="gpt-4o-mini", temperature=0), None, False

        # === OPTION WITH ROUTER (Iron Cage) ===
        print(f"\n> Initializing Iron Cage Router...")

        router = LlmRouter(**router_config)

        print(f"> Connected via Proxy on port {router.port}")
        # Budget is server-controlled in mode 2; local budget only applies in mode 3 if provided.
        if router_config.get("server_url"):
            print(f"> Server URL: {router_config['server_url']}")
        proxy_base = router.base_url.rstrip("/")
        print(f"> Proxy base URL: {proxy_base}")

        # Mode-specific client:
        if router_config.get("server_url"):
            # Mode 2 (server): use raw OpenAI SDK to match E2E behavior
            os.environ["OPENAI_BASE_URL"] = proxy_base
            os.environ["OPENAI_API_KEY"] = router.api_key
            client = OpenAI(base_url=proxy_base, api_key=router.api_key)
            AsyncOpenAI.default_client = AsyncOpenAI(base_url=proxy_base, api_key=router.api_key)
            print("\n--- Initialized OpenAI Client (SDK) ---")
            print(client)
            print("---------------------------------------")
            return client, router, True
        else:
            # Mode 3 (local proxy): keep LangChain client
            llm = ChatOpenAI(
                model="gpt-4o-mini",
                base_url=proxy_base,
                api_key=router.api_key    # proxy expects this token
            )
            OpenAI.default_client = OpenAI(base_url=proxy_base, api_key=router.api_key)
            AsyncOpenAI.default_client = AsyncOpenAI(base_url=proxy_base, api_key=router.api_key)
            print("\n--- Initialized ChatOpenAI Object Log ---")
            print(llm)
            print("-----------------------------------------")
            return llm, router, False

    except Exception as e:
        print(f"❌ Error setting up LLM: {e}")
        sys.exit(1)


# --- MAIN AGENT LOGIC ---
system_prompt = """
You are an API proxy. Your only goal is to return raw data in JSON format.
ALGORITHM:
1. Use `search_leads` to get IDs.
2. For EACH found ID, call `get_lead_details`.
3. Collect all results.
OUTPUT: JSON Array only. No markdown.
"""

def main():
    print("--------------------------------")
    print("|         LeadGen Agent         |")
    print("--------------------------------")
    
    llm, router, use_openai_sdk = setup_llm()

    if not use_openai_sdk:
        prompt = ChatPromptTemplate.from_messages([
            ("system", system_prompt),
            ("human", "{input}"),
            ("placeholder", "{agent_scratchpad}"),
        ])
        
        agent = create_tool_calling_agent(llm, tools_list, prompt)
        
        agent_executor = AgentExecutor(
            agent=agent, 
            tools=tools_list, 
            verbose=True,
            handle_parsing_errors=True
        )
    
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

                if use_openai_sdk:
                    # Simple SDK call (no tools) in server mode
                    response = llm.chat.completions.create(
                        model="gpt-4o-mini",
                        messages=[
                            {"role": "system", "content": system_prompt},
                            {"role": "user", "content": user_query},
                        ],
                    )
                    answer = response.choices[0].message.content
                    print("\nRESULT:")
                    print(answer)
                else:
                    result = agent_executor.invoke({"input": user_query})
                    
                    raw = result["output"].replace("```json", "").replace("```", "").strip()
                    try:
                        print("\nRESULT (JSON):")
                        print(json.dumps(json.loads(raw), indent=2, ensure_ascii=False))
                    except:
                        print(raw)
            except Exception as e:
                print(f"Error: {e}")
    finally:
        # Keep router alive for the session; stop it on exit.
        if router and getattr(router, "is_running", False):
            router.stop()

if __name__ == "__main__":
    main()

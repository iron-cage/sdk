import sys
import os
import json
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_core.prompts import ChatPromptTemplate
from apollo_tools import tools_list

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
            server_url = os.environ.get("IC_SERVER", "http://localhost:8080")
            return {
                "api_key": os.environ["IC_TOKEN"],
                "server_url": server_url,
                "budget": 10.0
            }
        else:
            raise ValueError("Mode 2 selected, but IC_TOKEN not found in secrets!")

    # If user selected Mode 3 (Local Iron Proxy) - "Secret/Standalone Mode"
    # This runs protection locally using your OpenAI key directly
    if mode_selection == "3":
        if os.environ.get("OPENAI_API_KEY"):
            return {
                "provider_key": os.environ["OPENAI_API_KEY"],
                "budget": 5.0
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
            print("> Initializing Direct OpenAI Connection...")
            if not os.environ.get("OPENAI_API_KEY"):
                print("❌ Error: OPENAI_API_KEY missing.")
                sys.exit(1)
            return ChatOpenAI(model="gpt-4o-mini", temperature=0)

        # === OPTION WITH ROUTER (Iron Cage) ===
        print(f"\n> Initializing Iron Cage Router...")
        
        # Here we simply pass the dictionary as arguments (**kwargs)
        # This works for both {api_key, server_url} and {provider_key} configs
        router = LlmRouter(**router_config)
        
        print(f"> Connected via Proxy on port {router.port}")
        if router.budget:
            print(f"> Budget Limit: ${router.budget}")

        return ChatOpenAI(
            model="gpt-4o-mini",
            temperature=0,
            base_url=router.base_url,
            api_key=router.api_key
        )

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
    
    llm = setup_llm()

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
    while True:
        try:
            user_query = input("\nEnter query (or 'exit'): ")
            if user_query.lower() in ['exit', 'quit']: break
            if not user_query.strip(): continue

            print("Processing...")
            result = agent_executor.invoke({"input": user_query})
            
            raw = result["output"].replace("```json", "").replace("```", "").strip()
            try:
                print("\nRESULT (JSON):")
                print(json.dumps(json.loads(raw), indent=2, ensure_ascii=False))
            except:
                print(raw)
        except Exception as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    main()
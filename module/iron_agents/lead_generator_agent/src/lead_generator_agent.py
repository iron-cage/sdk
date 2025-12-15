import sys
import os
import json
from langchain_openai import ChatOpenAI
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_core.prompts import ChatPromptTemplate
from apollo_tools import tools_list
from config import IC_TOKEN, OPENAI_API_KEY

# --- 1. INTELLIGENT PATH SETUP ---
# Check if Iron Cage library is available
def setup_python_paths():
    current = os.path.dirname(os.path.abspath(__file__))
    module_root = None

    while True:
        possible_sdk = os.path.join(current, "iron_sdk")
        if os.path.exists(possible_sdk):
            module_root = current
            break
        parent = os.path.dirname(current)
        if parent == current: break
        current = parent

    if module_root:
        print(f"DEBUG: Found module root at: {module_root}")
        
        if module_root not in sys.path:
            sys.path.insert(0, module_root)

        runtime_python_path = os.path.join(module_root, "iron_runtime", "python")
        if os.path.exists(runtime_python_path):
            print(f"DEBUG: Found Python runtime wrapper at: {runtime_python_path}")
            sys.path.insert(0, runtime_python_path)
        else:
            print("DEBUG: iron_runtime/python folder not found (maybe pure Rust?)")
            
        return True
    else:
        print("DEBUG: Could not auto-detect 'module' folder structure.")
        return False

paths_configured = setup_python_paths()

LlmRouter = None
IRON_CAGE_AVAILABLE = False

if paths_configured:
    try:
        try:
            import iron_runtime
            print("✅ Successfully imported iron_runtime")
        except ImportError as e:
            print(f"⚠️  Still cannot import iron_runtime: {e}")

        from iron_sdk import LlmRouter
        
        IRON_CAGE_AVAILABLE = True
        print("✅ Iron SDK & Runtime loaded.")
        
    except ImportError as e:
        print(f"⚠️  Import Failed: {e}")
        print("   Running in Direct Mode only.")
        IRON_CAGE_AVAILABLE = False

# Validation
if not OPENAI_API_KEY:
    print("CRITICAL ERROR: OPENAI_API_KEY is missing via config.")
    sys.exit(1)

# --- System Prompt for the Agent ---
# Defines the agent's role, algorithm, and strict output format.
system_prompt = """
You are an API proxy. Your only goal is to return raw data in JSON format.

ALGORITHM:
1. Use `search_leads` to get IDs.
2. For EACH found ID, call `get_lead_details`.
3. Collect all results from `get_lead_details` as a JSON.

OUTPUT FORMAT (CRITICAL):
- Return all and ONLY a valid JSON Array (list of objects).
- Do not write any text, do not say hello, do not explain anything.
- Do not use Markdown (no ```json).
- Do not truncate data.
- Do not revrite or modify the data.

Example output:
[
  {{ "id": "123", "name": "John" }},
  {{ "id": "456", "name": "Criss" }}
]
"""

# --- Create Prompt Template ---
# Combines the system instruction, user input, and a placeholder for intermediate steps.
def setup_llm():
    """
    Interactively asks the user which mode to use.
    """
    print("\n--- SELECT MODE ---")
    print("1. OpenAI API mode")
    print("2. Iron Cage  mode")
    
    choice = input("Select option (1 or 2): ").strip()
    
    # --- OPTION 2: IRON CAGE ---
    if choice == "2":
        if not IC_TOKEN:
            print("Error: IC_TOKEN is missing in secrets")
            sys.exit(1)

        print("\n> Initializing Iron Cage Router...")
        
        server_url = "http://localhost:8080" 
        
        try:
            router = LlmRouter(
                api_key=IC_TOKEN,
                server_url=server_url, 
                budget=10.0
            )
            
            print(f"> Connected via Iron Cage Proxy on port {router.port}")
            return ChatOpenAI(
                model="gpt-5-nano",
                temperature=0,
                base_url=router.base_url,
                api_key=router.api_key
            )
        except Exception as e:
            print(f"Error initializing Router: {e}")
            sys.exit(1)

    # --- OPTION 1: DIRECT MODE ---
    else:
        print("\n> Initializing Direct OpenAI Connection...")
        return ChatOpenAI(
            model="gpt-5-nano", 
            temperature=0
        )

# --- Main Execution Function ---
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
        max_iterations=20,
        handle_parsing_errors=True
    )
    
    print("\nAgent is ready. Waiting for commands.")
    print("--------------------------------")
    while True:
        try:
            user_query = input("\nEnter query (or 'exit'): ")
            if user_query.lower() in ['exit', 'quit']: break
            if not user_query.strip(): continue

            print("Processing request...")
            
            # --- Run the Agent to Process the Request ---
            # Passes the user's query to the agent for execution.
            result = agent_executor.invoke({"input": user_query})
            
            # --- Process and Print the Result ---
            # Cleans up the agent's response to remove extra characters.
            raw_output = result["output"].replace("```json", "").replace("```", "").strip()
            
            try:
                # Attempts to parse the text response as JSON and print it.
                data = json.loads(raw_output)
                
                print("\nRESULT (JSON):")
                print(json.dumps(data, indent=2, ensure_ascii=False))
                
            except json.JSONDecodeError:
                # If the response is not valid JSON, prints the raw text.
                print("\nAgent did not return valid JSON. Raw text:")
                print(raw_output)
                
        except Exception as e:
            print(f"Error: {e}")

# --- Program Entry Point ---
# Runs the main function if the script is executed directly.
if __name__ == "__main__":
    main()

import sys
import json
from langchain_openai import ChatOpenAI
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_core.prompts import ChatPromptTemplate
from apollo_tools import tools_list
from config import IC_TOKEN, OPENAI_API_KEY

# Check if Iron Cage library is available
try:
    from iron_cage import LlmRouter
    IRON_CAGE_AVAILABLE = True
except ImportError:
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
        if not IRON_CAGE_AVAILABLE:
            print("Error: 'iron_cage' library is not installed (pip install iron-cage).")
            sys.exit(1)
            
        if not IC_TOKEN:
            print("Error: IC_TOKEN is missing in secrets file")
            sys.exit(1)

        print("\n> Initializing Iron Cage Router...")
        router = LlmRouter(
            api_key=IC_TOKEN,
            budget_usd=10.0,
            pii_detection=True,
            circuit_breaker=True
        )
        
        print("> Connected via Iron Cage Proxy")
        return ChatOpenAI(
            model="gpt-5-nano",
            temperature=0,
            base_url=router.base_url,
            api_key=router.api_key
        )

    # --- OPTION 1: DIRECT (Default) ---
    else:
        print("\n> Initializing Direct OpenAI Connection...")
        return ChatOpenAI(
            model="gpt-4o-mini", 
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

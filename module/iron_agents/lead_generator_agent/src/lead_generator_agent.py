import os
import json
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_core.prompts import ChatPromptTemplate
from apollo_tools import tools_list
from iron_cage import LlmRouter

# --- Load Configuration ---
# Get path to this script
script_dir = os.path.dirname(os.path.abspath(__file__))

# Build path to secrets file
secrets_path = os.path.join(script_dir, "..", "secret", "-secrets.sh")

# Load variables
load_dotenv(secrets_path, override=True)

# Check for OpenAI Token
if not os.getenv("OPENAI_API_KEY"):
    raise ValueError("CRITICAL ERROR: OPENAI_API_KEY is missing.")

IC_TOKEN = os.getenv("IC_TOKEN")
if not IC_TOKEN:
    raise ValueError(f"CRITICAL ERROR: IC_TOKEN is missing in {secrets_path}")

# --- Initialize Iron Cage Router ---
print("Initializing Iron Cage Router...")
router = LlmRouter(
    api_key=IC_TOKEN,         
    budget_usd=10.0,          # Hard limit
    pii_detection=True,       # Auto-redact emails/phones
    circuit_breaker=True      # Handle API failures
)

# --- Initialize Language Model ---
llm = ChatOpenAI(
    model="gpt-5-nano",
    temperature=0,
    api_key=router.api_key,
    base_url=router.base_url
)

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
prompt = ChatPromptTemplate.from_messages([
    ("system", system_prompt),
    ("human", "{input}"),
    ("placeholder", "{agent_scratchpad}"),
])

# --- Create Agent and Executor ---
# Creates the agent's logic by combining the model, tools, and prompt.
agent = create_tool_calling_agent(llm, tools_list, prompt)

# Creates the executor, which runs the agent and manages its workflow.
agent_executor = AgentExecutor(
    agent=agent, 
    tools=tools_list, 
    verbose=True,
    max_iterations=20,
    max_execution_time=120.0,
    handle_parsing_errors=True
)

# --- Main Execution Function ---
# Starts the user interaction loop.
def main():
    print("--------------------------------")
    print("|         LeadGen Agent         |")
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

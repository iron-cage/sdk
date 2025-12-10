import os
import json
import re
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_core.prompts import ChatPromptTemplate
from apollo_tools import tools_list

load_dotenv()

llm = ChatOpenAI(model="gpt-4o", temperature=0)

system_prompt = """
You are an API proxy. Your only goal is to return raw data in JSON format.

ALGORITHM:
1. Use `search_leads` to get IDs.
2. For EACH found ID, call `get_lead_details`.
3. Collect all results from `get_lead_details` into a single list.

OUTPUT FORMAT (CRITICAL):
- Return ONLY a valid JSON Array (list of objects).
- Do not write any text, do not say hello, do not explain anything.
- Do not use Markdown (no ```json).
- Do not truncate data.

Example output:
[
  {{ "id": "123", "name": "Ivan" }},
  {{ "id": "456", "name": "Petro" }}
]
"""

prompt = ChatPromptTemplate.from_messages([
    ("system", system_prompt),
    ("human", "{input}"),
    ("placeholder", "{agent_scratchpad}"),
])

agent = create_tool_calling_agent(llm, tools_list, prompt)

agent_executor = AgentExecutor(
    agent=agent, 
    tools=tools_list, 
    verbose=False,
    max_iterations=20
)

def main():
    print("Lead Gen Agent (Raw JSON Output)")
    print("--------------------------------")
    
    while True:
        try:
            user_query = input("\nEnter query (or 'exit'): ")
            if user_query.lower() in ['exit', 'quit']: break
            if not user_query.strip(): continue

            print("Processing request...")
            
            result = agent_executor.invoke({"input": user_query})
            
            raw_output = result["output"].replace("```json", "").replace("```", "").strip()
            
            try:
                data = json.loads(raw_output)
                
                print("\nRESULT (JSON):")
                print(json.dumps(data, indent=2, ensure_ascii=False))
                
            except json.JSONDecodeError:
                print("\nAgent did not return valid JSON. Raw text:")
                print(raw_output)
                
        except Exception as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    main()
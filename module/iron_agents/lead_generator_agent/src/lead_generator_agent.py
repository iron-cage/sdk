import json
import os
import sys
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain_core.messages import HumanMessage
from langgraph.prebuilt import create_react_agent
from apollo_tools import tools_list
from iron_cage import LlmRouter

current = os.path.dirname(os.path.abspath(__file__))
load_dotenv(os.path.join(current, "../secret/-secrets.sh"), override=True)

router = LlmRouter(
    api_key=os.environ["IC_TOKEN"],
    server_url=os.environ["IC_SERVER"],
    budget=1.0
)

llm = ChatOpenAI(
    model="gpt-5-nano",
    temperature=1,
    base_url=router.base_url,
    api_key=router.api_key
)

system_prompt = """
You are a lead generation assistant.
1. Use search_leads to get IDs.
2. Call get_lead_details for each ID.
3. Return a full JSON Array of results.
"""

agent = create_react_agent(llm, tools_list, prompt=system_prompt)

def main():
    try:
        while True:
            try:
                query = input("\nQuery: ")
                if query in ['exit', 'quit']: break
                result = agent.invoke({"messages": [HumanMessage(content=query)]})         
                raw = result["messages"][-1].content
                print(raw.replace("```json", "").replace("```", "").strip())
                
            except Exception as e:
                print(e)
    finally:
        router.stop()

if __name__ == "__main__":
    main()
import json
import os
<<<<<<< HEAD
import re
import sys

from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from langchain_core.messages import HumanMessage, SystemMessage
from langgraph.prebuilt import create_react_agent
from openai import AsyncOpenAI, OpenAI

OUTREACH_MODEL = os.environ.get("OUTREACH_MODEL", "gpt-5-mini")
OUTREACH_MAX_COMPLETION_TOKENS = int(os.environ.get("OUTREACH_MAX_COMPLETION_TOKENS", "12000"))
OUTREACH_BODY_MAX_WORDS = int(os.environ.get("OUTREACH_BODY_MAX_WORDS", "500"))

try:
    from apollo_tools import tools_list  # real Apollo-backed tools
except ImportError:  # pragma: no cover
    from .apollo_tools import tools_list

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
            server_url = (os.environ.get("IC_SERVER") or "http://localhost:3001").strip()
            if server_url and not server_url.startswith(("http://", "https://")):
                server_url = f"http://{server_url}"
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
            llm = ChatOpenAI(model="gpt-4o-mini", temperature=0)
            outreach_llm = ChatOpenAI(
                model=OUTREACH_MODEL,
                temperature=0,
                max_completion_tokens=OUTREACH_MAX_COMPLETION_TOKENS,
            )
            return llm, outreach_llm, None, False

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
        outreach_llm = ChatOpenAI(
            model=OUTREACH_MODEL,
            base_url=proxy_base,
            api_key=proxy_api_key,
            temperature=0,
            max_completion_tokens=OUTREACH_MAX_COMPLETION_TOKENS,
        )

        # Build standard OpenAI clients pointing at the proxy (for any direct SDK use)
        proxy_openai_client = OpenAI(base_url=proxy_base, api_key=proxy_api_key)
        proxy_async_client = AsyncOpenAI(base_url=proxy_base, api_key=proxy_api_key)

        OpenAI.default_client = proxy_openai_client
        AsyncOpenAI.default_client = proxy_async_client

        print("\n--- Initialized ChatOpenAI Object (via Proxy) ---")
        print(llm)
        print("-------------------------------------------------")

        return llm, outreach_llm, router, False

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
2. For EACH found ID, call `get_lead_details` (pass the Apollo person `apollo_id`) to enrich the data.
3. Combine all enriched results into a single list.
OUTPUT:
- JSON Array only. Do not wrap the output in markdown fences (```json).
- Each item must be a simplified lead object with ONLY these keys:
  {"id","first_name","last_name","title","company","industry","location","email","linkedin_url"}
- Do not invent values; use null when missing.
"""
# --- END AGENT COMPONENTS ---


def _strip_markdown_fences(text: str) -> str:
    return text.replace("```json", "").replace("```", "").strip()


def _parse_quantity_from_query(user_query: str) -> tuple[str, int | None]:
    """
    Supports: `qty=10`, `quantity: 10`, `q=10`, `--qty 10`, `--quantity 10`.
    Returns (cleaned_query, quantity_or_none).
    """
    query = user_query.strip()

    m = re.search(r"(?i)\\b(?:qty|quantity|q)\\s*[:=]\\s*(\\d{1,4})\\b", query)
    if m:
        qty = int(m.group(1))
        cleaned = (query[: m.start()] + query[m.end() :]).strip()
        return cleaned, qty

    m = re.search(r"(?i)(?:^|\\s)--(?:qty|quantity)\\s+(\\d{1,4})\\b", query)
    if m:
        qty = int(m.group(1))
        cleaned = (query[: m.start()] + query[m.end() :]).strip()
        return cleaned, qty

    return query, None


def generate_outreach_message_for_lead(llm: ChatOpenAI, lead: dict, user_query: str) -> tuple[dict | None, str]:
    system = SystemMessage(
        content=(
            "You are an SDR assistant. Given a user's request and a single lead JSON object, "
            "generate one outbound email.\n"
            "Rules:\n"
            "- Output JSON object only (no markdown).\n"
            "- Do not invent personal details; use only fields present in the lead.\n"
            f"- Keep body under {OUTREACH_BODY_MAX_WORDS} words.\n"
            "Schema: {\"id\": string, \"to\": string|null, \"subject\": string, \"body\": string}."
        )
    )
    human = HumanMessage(
        content=(
            f"User request:\n{user_query}\n\n"
            f"Lead JSON:\n{json.dumps(lead, ensure_ascii=False)}"
        )
    )
    message = llm.invoke([system, human])
    raw = _strip_markdown_fences(getattr(message, "content", "") or "")
    try:
        parsed = json.loads(raw)
        if isinstance(parsed, dict):
            return parsed, raw
    except Exception:
        pass
    return None, raw


def generate_outreach_messages(llm: ChatOpenAI, leads: list[dict], user_query: str) -> tuple[list[dict] | None, str]:
    """
    Second stage: take the leads JSON and generate example outreach messages.
    Returns (parsed_json_or_none, raw_text).
    """
    system = SystemMessage(
        content=(
            "You are an SDR assistant. Given a user's request and a JSON array of leads, "
            "generate one concise outbound email per lead.\n"
            "Rules:\n"
            "- Output JSON array only (no markdown).\n"
            "- One item per lead.\n"
            "- Do not invent personal details; use only fields present in the lead.\n"
            f"- Keep body under {OUTREACH_BODY_MAX_WORDS} words.\n"
            "Schema per item: {\"id\": string, \"to\": string|null, \"subject\": string, \"body\": string}."
        )
    )
    human = HumanMessage(
        content=(
            f"User request:\n{user_query}\n\n"
            f"Leads JSON:\n{json.dumps(leads, ensure_ascii=False)}"
        )
    )
    message = llm.invoke([system, human])
    raw = _strip_markdown_fences(getattr(message, "content", "") or "")
    try:
        parsed = json.loads(raw)
        if isinstance(parsed, list):
            return parsed, raw
    except Exception:
        pass
    return None, raw


def main():
    print("--------------------------------")
    print("|         LeadGen Agent         |")
    print("--------------------------------")

    llm, outreach_llm, router, use_openai_sdk = setup_llm()

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

                cleaned_query, desired_quantity = _parse_quantity_from_query(user_query)
                if desired_quantity is None:
                    qty_in = input("Quantity (default 3): ").strip()
                    if qty_in:
                        try:
                            desired_quantity = int(qty_in)
                        except ValueError:
                            print("Invalid quantity; using default.")
                            desired_quantity = None

                if router:
                    print(f"Processing... (proxy {router.base_url})")
                else:
                    print("Processing... (direct OpenAI)")

                if agent:
                    if desired_quantity is not None:
                        content = (
                            f"{cleaned_query}\n\n"
                            f"Desired quantity: {desired_quantity}.\n"
                            f"Call `search_leads` with quantity={desired_quantity}."
                        )
                    else:
                        content = cleaned_query

                    result = agent.invoke({"messages": [HumanMessage(content=content)]})
                    messages = result.get("messages") or []
                    raw = _strip_markdown_fences(messages[-1].content if messages else "")
                    leads = None
                    try:
                        leads = json.loads(raw)
                    except Exception:
                        pass

                    if not isinstance(leads, list):
                        print("\nRESULT (RAW - non-JSON):")
                        print(raw)
                        continue

                    print("\nRESULT (JSON):")
                    print(json.dumps(leads, indent=2, ensure_ascii=False))

                    if leads:
                        try:
                            outreach_json, outreach_raw = generate_outreach_messages(outreach_llm, leads, user_query)
                        except Exception as e:
                            print(f"\nOUTREACH ERROR ({OUTREACH_MODEL}): {e}")
                            outreach_json, outreach_raw = generate_outreach_messages(llm, leads, user_query)

                        if outreach_json is not None:
                            if len(outreach_json) != len(leads):
                                stitched: list[dict] = []
                                for lead in leads:
                                    msg_json, msg_raw = generate_outreach_message_for_lead(outreach_llm, lead, user_query)
                                    if msg_json is not None:
                                        stitched.append(msg_json)
                                    elif msg_raw:
                                        stitched.append(
                                            {
                                                "id": lead.get("id"),
                                                "to": lead.get("email"),
                                                "subject": "Outreach",
                                                "body": msg_raw,
                                            }
                                        )
                                outreach_json = stitched
                            print("\nOUTREACH (JSON):")
                            print(json.dumps(outreach_json, indent=2, ensure_ascii=False))
                        elif outreach_raw:
                            print("\nOUTREACH (RAW - non-JSON):")
                            print(outreach_raw)
                else:
                    print("❌ Error: Agent failed to initialize. Cannot run query.")

            except Exception as e:
                print(f"Error during query execution: {e}")
    finally:
        # Stop router on exit
        if router and hasattr(router, "is_running") and router.is_running:
            router.stop()
=======
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
>>>>>>> f326cba9b63f81a68e9971089276fd64a0ba039f

if __name__ == "__main__":
    main()
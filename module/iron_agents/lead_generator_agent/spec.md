# AI Lead Generator Agent Specification

## 1. Project Overview
The **AI Lead Generator Agent** is a command-line interface (CLI) tool designed to automate the process of finding and enriching business leads. It leverages the **Apollo.io API** for data retrieval and **OpenAI's GPT-4o** for orchestration and natural language understanding. The agent autonomously searches for professionals based on user criteria, retrieves their contact details, and formats the output as a clean JSON array.

## 2. Tech Stack & Environment
*   **Language:** Python 3.12+
*   **Package Manager:** `uv`
*   **Core Libraries:**
    *   `langchain` / `langchain-openai` (Agent orchestration)
    *   `requests` (HTTP calls)
    *   `pydantic` (Data validation)
    *   `python-dotenv` (Configuration)
*   **Testing:** `pytest`, `unittest.mock`
*   **External APIs:**
    *   OpenAI API (Model: `gpt-4o`)
    *   Apollo.io API (Endpoints: Search, People Details)

## 3. Architecture

The project is structured into two main modules located in the `src/` directory (or root, depending on configuration):

### 3.1. Tool Layer (`apollo_tools.py`)
Encapsulates direct interactions with the Apollo API. These functions are decorated with `@tool` to be callable by the LLM.

*   **`search_leads`**
    *   **Purpose:** Finds lead profiles based on job title, industry, and location.
    *   **Input:** `job_title` (str), `industry` (str, optional), `location` (str, optional), `quantity` (int, default=3).
    *   **API Endpoint:** `POST /v1/mixed_people/search`
    *   **Output:** A simplified JSON string containing a list of leads (ID, Name, Organization, Title).
    *   **Error Handling:** Handles non-200 statuses and empty results.

*   **`get_lead_details`**
    *   **Purpose:** Retrieves enriched data (specifically emails) for a specific lead ID.
    *   **Input:** `apollo_id` (str).
    *   **API Endpoint:** `GET /v1/people/{id}`
    *   **Parameters:** `reveal_personal_emails=true`.
    *   **Output:** Full person object in JSON format.

### 3.2. Agent Layer (`lead_generator_agent.py`)
Manages the decision-making process and user interaction.

*   **LLM Configuration:** Uses `gpt-4o` with `temperature=0` for deterministic outputs.
*   **System Prompt Strategy:**
    *   Defines a strict **3-step Algorithm**:
        1.  Call `search_leads` to get IDs.
        2.  Call `get_lead_details` for each ID found.
        3.  Aggregate results into a single list.
    *   **Output Format:** Strictly enforces a valid JSON Array response (no Markdown formatting, no conversational text).
*   **Execution:** Uses `AgentExecutor` with `max_iterations=20` and error parsing enabled.
*   **CLI Loop:** Accepts user input, invokes the agent, cleans raw output (removes Markdown code blocks), and prints formatted JSON.

## 4. Data Flow

1.  **User Input:** User enters a query (e.g., "Find CEOs in generic SaaS companies in Ukraine").
2.  **LLM Processing:** The Agent analyzes the request and decides to call `search_leads`.
3.  **Search Execution:** `apollo_tools` queries Apollo API and returns a list of candidate IDs.
4.  **Enrichment Loop:** The Agent iterates through the IDs and calls `get_lead_details` for each one.
5.  **Aggregation:** The Agent combines the enriched data.
6.  **Final Output:** The Agent returns a raw JSON string.
7.  **Post-Processing:** The script strips ` ```json ` tags and parses the string.

## 5. Configuration
The application requires a `.env` file in the root directory with the following keys:

```ini
OPENAI_API_KEY=sk-...
APOLLO_API_KEY=...
```

## 6. Testing Strategy
Tests are located in the `tests/` directory and run via `uv run pytest`.

*   **Mocking:** All external API calls (Apollo) and LLM interactions (OpenAI) are mocked using `unittest.mock`. No real credits are consumed during testing.
*   **Coverage:**
    *   **`test_apollo_tools.py`**: Verifies API payload construction, success responses, empty results, and error handling.
    *   **`test_agent.py`**: Verifies JSON parsing logic and the agent executor's ability to handle the flow (using mocked executor objects to bypass Pydantic validation constraints).

## 7. Setup & Run

**Installation:**
```bash
uv init
uv sync
```

**Running the Agent:**
```bash
uv run python lead_generator_agent.py
# Or if structured with src:
uv run python -m src.lead_generator_agent
```

**Running Tests:**
```bash
uv run pytest
```
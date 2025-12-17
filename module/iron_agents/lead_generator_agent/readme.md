# Apollo Lead Generation Agent

Agent for searching, generating, and enriching information about leads

### Overview

This module is a Python-based AI agent designed to automate lead generation using the Apollo.io API and OpenAI's GPT-4o. It functions as an intelligent proxy that accepts natural language queries, translates them into API calls, and returns enriched lead data in a raw JSON format.

The agent follows a two-step process:
1.  **Search:** Identifies potential leads based on job titles, industries, and locations to retrieve unique Apollo IDs.
2.  **Enrichment:** Uses the retrieved IDs to fetch detailed profile information, ensuring high accuracy and full data retrieval.

### Prerequisites

-   **Python:** Version 3.10 or higher (3.12+ recommended).
-   **Package Manager:** `uv`.
-   **API Tokens:**
    -   OpenAI API Token (GPT-4o model access).
    -   Apollo.io API Token (Master Token recommended).
    -   Iron Cage API Token (Master Token recommended).

### Installation

This project is configured to use `uv` for dependency management.

  **Sync the project:**
    ```bash
    uv sync
    ```

### Configuration



Create a file named `-agent_secrets.sh` in the secret folder with tokens. You can use agent_secrets.template to copypaste variables.

**File: -agent_secrets.sh**
```
OPENAI_API_KEY=sk-your_openai_token_here
APOLLO_API_KEY=your_apollo_api_token_here
IC_TOKEN=your_ic_token_here
```

### Project Structure

```text
.
├── .venv/                               # Virtual environment managed by uv
├── pyproject.toml                       # Project dependencies definition
├── uv.lock                              # Dependency lock file
├── docs
│   └── readme.md                        # Guideline for lead generation agent
├── src
│   ├── apollo_tools.py                  # Tool definitions for Apollo API interaction
│   ├── config.py                        # Tool for finding secrets folder and imports tokens
│   ├── readme.md                        # File Responsibility Table
│   └── lead_generator_agent.py          # Main entry point and agent logic
├── tests
│   ├── test_agent.py                    # Tests for agent
│   └── test_apollo_tools.py             # Tests for apollo tools
├── license                              # License
├── spec.md                              # Specification of project
└── readme.md                            # Project documentation
```

The module consists of two main Python files:

#### 1. `apollo_tools.py`
This file contains the core logic for interacting with the Apollo.io API. It defines two specific tools decorated with `@tool` for LangChain integration.

*   **`search_leads`**:
    *   **Purpose:** Searches for people based on job title, industry keywords, and location.
    *   **Parameters:** `job_title` (str), `industry` (str, optional), `location` (str, optional), `quantity` (int).
    *   **Output:** A list of simplified lead objects containing the Apollo `id`.
    *   **Implementation Note:** API tokens are passed via the `X-Api-Key` header to comply with Apollo's security standards.

*   **`get_lead_details`**:
    *   **Purpose:** Retrieves the full profile of a specific lead using their Apollo ID.
    *   **Parameters:** `apollo_id` (str).
    *   **Output:** A raw JSON object containing all available data for the lead (emails, social links, employment history, company details).
    *   **Configuration:** `reveal_personal_emails` is set to `true`, while `reveal_phone_number` is set to `false` to avoid webhook requirements.

#### 2. `lead_generator_agent.py`
This is the entry point of the application. It initializes the OpenAI model and the LangChain AgentExecutor.

*   **Model:** Uses `gpt-5-nano` with `temperature=0` and `max-retries=2` for deterministic and precise output.
*   **System Prompt:** Strictly enforces a JSON-only output format. It instructs the agent to perform the search first and then iterate through every found ID to fetch details.
*   **Output Handling:** The script captures the agent's output, strips any Markdown formatting, validates the JSON, and prints the result to the console.

### Agent prompt

```You are an API proxy. Your only goal is to return raw data in JSON format.

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
```

### Usage

### How to run an agent:
1. Sync all by `uv sync`
2. Create a file -secrets.sh and paste related tokets
3. Install iron_cage library
```md
 cd module/iron_runtime

  # Create venv and install dependencies
  uv venv
  uv pip install maturin

  # Then build
  uv run maturin develop
```
3. Move to folder `cd module/iron_agent/lead_generator_agent/src` and type `uv run lead_generator_agent.py` in a console
4. Type a prompt in a console, for example: `Find 3 Jewelry Founders in Germany`
5. Wait for answer

#### Interactive Mode
Once running, the script will prompt you for a query.

**Example Query:**
> Find 3 Jewelry Founders in Germany

**Process Flow:**
1.  The agent parses the query to extract:
    *   Job Title: "Founder"
    *   Industry: "Jewelry"
    *   Location: "Germany"
    *   Quantity: 3
2.  It calls `search_leads` with these parameters.
3.  It receives a list of IDs (e.g., `["id_1", "id_2", "id_3"]`).
4.  It sequentially calls `get_lead_details` for each ID.
5.  It aggregates the results into a single JSON array.

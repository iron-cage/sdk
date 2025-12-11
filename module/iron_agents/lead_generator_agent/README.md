# Apollo Lead Generation Agent

## Overview

This module is a Python-based AI agent designed to automate lead generation using the Apollo.io API and OpenAI's GPT-4o. It functions as an intelligent proxy that accepts natural language queries, translates them into API calls, and returns enriched lead data in a raw JSON format.

The agent follows a two-step process:
1.  **Search:** Identifies potential leads based on job titles, industries, and locations to retrieve unique Apollo IDs.
2.  **Enrichment:** Uses the retrieved IDs to fetch detailed profile information, ensuring high accuracy and full data retrieval.

## Prerequisites

-   **Python:** Version 3.10 or higher (3.12+ recommended).
-   **Package Manager:** `uv`.
-   **API Keys:**
    -   OpenAI API Key (GPT-4o model access).
    -   Apollo.io API Key (Master Key recommended).

## Installation

This project is configured to use `uv` for dependency management.

  **Initialize the project (if starting fresh):**
    ```bash
    uv init
    ```

## Configuration

Create a file named `.env` in the root directory of the project. Add your API keys to this file.

**File: .env**
```env
OPENAI_API_KEY=sk-your_openai_key_here
APOLLO_API_KEY=your_apollo_api_key_here
```

## Project Structure

```text
.
├── .env                    # Environment variables (API Keys)
├── .venv/                  # Virtual environment managed by uv
├── pyproject.toml          # Project dependencies definition
├── uv.lock                 # Dependency lock file
├── apollo_tools.py         # Tool definitions for Apollo API interaction
├── lead_generator_agent.py # Main entry point and agent logic
└── README.md               # Project documentation
```

The module consists of two main Python files:

### 1. `apollo_tools.py`
This file contains the core logic for interacting with the Apollo.io API. It defines two specific tools decorated with `@tool` for LangChain integration.

*   **`search_leads`**:
    *   **Purpose:** Searches for people based on job title, industry keywords, and location.
    *   **Parameters:** `job_title` (str), `industry` (str, optional), `location` (str, optional), `quantity` (int).
    *   **Output:** A list of simplified lead objects containing the Apollo `id`.
    *   **Implementation Note:** API keys are passed via the `X-Api-Key` header to comply with Apollo's security standards.

*   **`get_lead_details`**:
    *   **Purpose:** Retrieves the full profile of a specific lead using their Apollo ID.
    *   **Parameters:** `apollo_id` (str).
    *   **Output:** A raw JSON object containing all available data for the lead (emails, social links, employment history, company details).
    *   **Configuration:** `reveal_personal_emails` is set to `true`, while `reveal_phone_number` is set to `false` to avoid webhook requirements.

### 2. `lead_generator_agent.py`
This is the entry point of the application. It initializes the OpenAI model and the LangChain AgentExecutor.

*   **Model:** Uses `gpt-4o` with `temperature=0` for deterministic and precise output.
*   **System Prompt:** Strictly enforces a JSON-only output format. It instructs the agent to perform the search first and then iterate through every found ID to fetch details.
*   **Output Handling:** The script captures the agent's output, strips any Markdown formatting, validates the JSON, and prints the result to the console.

## Usage

To run the agent, execute the following command in your terminal:

```bash
uv run lead_generator_agent.py
```

### Interactive Mode
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

### Output Format
The output will be a raw JSON array printed to the console:

```json
[
  {
    "id": "64b...",
    "first_name": "Name",
    "last_name": "Surname",
    "organization": {
      "name": "Company Name",
      "website_url": "http://example.com"
    },
    "email": "name@example.com",
    "linkedin_url": "http://linkedin.com/in/profile",
    "employment_history": [...]
  }
]
```

## Troubleshooting

### Common Errors

1.  **ImportError: cannot import name 'create_tool_calling_agent'**
    *   **Cause:** Outdated version of `langchain`.
    *   **Solution:** Update dependencies using `uv add langchain --upgrade`.

2.  **Apollo API Error 422: INVALID_API_KEY_LOCATION**
    *   **Cause:** The API key was sent in the request body instead of the headers.
    *   **Solution:** Ensure you are using the provided version of `apollo_tools.py` which passes the key in the `X-Api-Key` header.

3.  **Match Error 400: Please add a valid 'webhook_url'**
    *   **Cause:** Attempting to reveal phone numbers without a configured webhook.
    *   **Solution:** Ensure `reveal_phone_number` is set to `"false"` in `apollo_tools.py`.

4.  **Agent returns text instead of JSON**
    *   **Cause:** The LLM ignored the system prompt instructions.
    *   **Solution:** The current script includes logic to strip Markdown tags (` ```json `). If persistence occurs, ensure `temperature` is set to `0` in `lead_generator_agent.py`.
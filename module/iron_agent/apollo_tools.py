import os
import requests
import json
from langchain.agents import tool
from dotenv import load_dotenv

load_dotenv()
APOLLO_API_KEY = os.getenv("APOLLO_API_KEY")

@tool
def search_leads(job_title: str, industry: str | None = None, location: str | None = None, quantity: int = 3):
    """
    Searches for people in the Apollo database.
    
    IMPORTANT:
    - job_title: ONLY the title (e.g., "Owner", "CEO", "Marketing Manager").
    - industry: Company industry or keyword (e.g., "Jewelry", "SaaS", "Real Estate").
    - location: Country or city.
    """
    url = "https://api.apollo.io/v1/mixed_people/search"
    
    headers = {
        "Content-Type": "application/json",
        "Cache-Control": "no-cache",
        "X-Api-Key": APOLLO_API_KEY
    }
    
    payload = {
        "person_titles": [job_title],
        "page": 1,
        "per_page": quantity,
        "contact_email_status": ["verified", "likely_to_engage"]
    }
    
    if location and isinstance(location, str) and location.strip():
        payload["person_locations"] = [location]
        
    if industry and isinstance(industry, str) and industry.strip():
        payload["q_organization_keyword_tags"] = [industry]

    try:
        response = requests.post(url, headers=headers, json=payload)
        
        if response.status_code != 200:
            return f"Apollo Search Error {response.status_code}: {response.text}"
            
        data = response.json()
        
        clean_results = []
        for p in data.get('people', []):
            clean_results.append({
                "id": p.get("id"),
                "first_name": p.get("first_name"),
                "last_name": p.get("last_name"),
                "organization_name": p.get("organization", {}).get("name"),
                "title": p.get("title")
            })
            
        if not clean_results:
            return json.dumps({"message": "0 leads found. Try broader keywords."}, ensure_ascii=False)

        return json.dumps(clean_results, ensure_ascii=False)
        
    except Exception as e:
        return f"System Error: {str(e)}"

@tool
def get_lead_details(apollo_id: str):
    """
    Retrieves full details about a lead using their Apollo ID.
    Use the ID returned by search_leads.
    """
    url = f"https://api.apollo.io/v1/people/{apollo_id}"
    
    headers = {
        "Content-Type": "application/json",
        "Cache-Control": "no-cache",
        "X-Api-Key": APOLLO_API_KEY
    }
    
    params = {
        "reveal_personal_emails": "true",
        "reveal_phone_number": "false"
    }
    
    try:
        response = requests.get(url, headers=headers, params=params)
        
        if response.status_code != 200:
             return f"Details Error {response.status_code}: {response.text}"

        data = response.json()
        person_data = data.get('person')

        if not person_data:
            return json.dumps({"error": "Lead ID not found or empty response"}, ensure_ascii=False)

        return json.dumps(person_data, ensure_ascii=False)
            
    except Exception as e:
        return json.dumps({"error": str(e)}, ensure_ascii=False)

tools_list = [search_leads, get_lead_details]
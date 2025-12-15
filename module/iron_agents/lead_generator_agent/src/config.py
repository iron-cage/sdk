import os
from dotenv import load_dotenv

# --- Internal Configuration Logic ---
def _load_config():
    """
    Searches for the 'secret' folder and '-agent_secret.sh' file 
    by moving up the directory tree.
    """
    # Start from the directory where this script is located
    current_path = os.path.dirname(os.path.abspath(__file__))
    
    while True:
        # Construct the path: current_dir/secret/-agent_secret.sh
        secret_file = os.path.join(current_path, "secret", "-agent_secret.sh")
        
        if os.path.exists(secret_file):
            print(f"DEBUG: Config loaded from: {secret_file}")
            # override=True is important to ensure variables are updated
            load_dotenv(secret_file, override=True)
            return True
        
        # Move up one directory level
        parent = os.path.dirname(current_path)
        
        # Check if we have reached the root of the drive (e.g., C:\)
        if parent == current_path: 
            print("WARNING: Config file 'secret/-agent_secret.sh' not found in any parent directory!")
            return False
            
        current_path = parent

# Execute logic immediately upon import
_load_config()

# --- EXPORT VARIABLES ---
# Other files will import these variables directly
APOLLO_API_KEY = os.getenv("APOLLO_API_KEY")
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
IC_TOKEN = os.getenv("IC_TOKEN")

# Optional: Debug check
if not APOLLO_API_KEY:
    print("Warning: APOLLO_API_KEY is missing in environment.")
if not OPENAI_API_KEY:
    print("Warning: OPENAI_API_KEY is missing in environment.")
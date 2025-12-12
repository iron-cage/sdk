# Secrets Folder

This folder contains sensitive information related to the Lead Generator Agent API tokens.

## Contents

- **secrets.template.sh**: File that contain API tokens template. Copy information from that file and create secrets.sh with your API tokens.
- **secrets.template.sh**: File with real API tokens that works with agent.

## Best Practices

- **Do Not Share**: Never share the contents of this folder publicly or with unauthorized personnel.
- **Use Environment Variables**: Store sensitive information in `secrets.sh`
- **Git**: Don't push any real tokens into a git repository!
- **Use Template**: Use template with all required varibels and paste API tokens. For example: OPENAI_API_KEY=your_token;
APOLLO_API_KEY=your_token.

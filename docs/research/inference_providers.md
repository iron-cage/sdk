# Inference providers analysis

**Research Date:** 2025-12-09

**Purpose:** Determine what inference providers have programmatic API Key management, ability to set hard spending limits and adjustable rate-limits.

---

## Programmatic Key Management

### Providers WITH Full Programmatic Key Management 
- AWS Bedrock [AWS documentation](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys-generate.html#api-keys-generate-api-long-term)
- xAI Grok (Keys can be created pragrammatically with [management key](https://console.x.ai/team/4e8bc96d-e5d9-4f29-a19e-6048a550906c/settings/management-keys/create and [RESTful API call](https://docs.x.ai/docs/key-information/using-management-api#create-an-api-key))

### Providers WITH Partial Programmatic Key Management
- Azure OpenAI ([Forum answer](https://learn.microsoft.com/en-us/answers/questions/1630958/will-the-azure-openai-api-key-be-expired-and-is-th))
- Google Gemini ([Can be configured in `gcloud` CLI, but needs Google Authorization first](https://docs.cloud.google.com/iam/docs/keys-create-delete#rest))

### Providers WITHOUT Programmatic Key Management
 - OpenAI (Has [administrator RESTful API](https://platform.openai.com/docs/api-reference/project-api-keys), but doesn't allow to create project keys.)
 - Anthropic (Has [endpoints](https://platform.claude.com/docs/en/api/admin/api_keys/retrieve) only for read-update)
 - Groq (Only [web UI](https://console.groq.com/keys))
 - Cohere (Only [web UI](https://dashboard.cohere.com/api-keys))
 - Mistral AI (Only [web UI](https://admin.mistral.ai/organization/api-keys))
 - Together AI (Only [web UI](https://docs.together.ai/docs/together-and-llamarank#1-get-your-together-api-key))
 - Replicate (Only [web UI](https://replicate.com/docs/reference/http))

**Final Outcome:** Cloud providers have their own RESTful API and CLI for service control that includes AI service, but setup needs extensive authorization procedure (e. g. Google Sign In). Other popular services rely on web UI for token setup and control. xAI grok supports programmatic key management which is useful in automatic key rotation, although it is needed to create a management key first.

## Hard Limit availability (Billing limit)

### Providers WITH Billing limit using budget in USD
 - OpenAI (soft and hard limits ([blog](https://mehmetbaykar.com/posts/setting-openai-api-key-limits-by-project/)))
 - Groq (blocking hard limit in [docs](https://console.groq.com/docs/spend-limits))
 - Cohere (spending limit in [billing](https://dashboard.cohere.com/billing?tab=spending-limit))
 - Mistral AI (Organization limits in [Limits](https://admin.mistral.ai/plateforme/limits))

### Providers WITH Per-Project billing (Clouds)
 - AWS Bedrock (kill-switch can be developed for [policy](https://docs.aws.amazon.com/pdfs/bedrock/latest/userguide/bedrock-ug.pdf) - page 57)
 - Google Gemini (kill-switch in [cloud](https://docs.cloud.google.com/billing/docs/how-to/budgets-programmatic-notifications#cap_and_disable_billing_to_stop_usage))
 - Azure OpenAI (kill-switch via [action groups](https://learn.microsoft.com/en-us/azure/cost-management-billing/costs/tutorial-acm-create-budgets?tabs=psbudget#trigger-an-action-group))

### Providers WITH Credit System but without explicit budget limit
 - Antropic ([Credits](https://platform.claude.com/settings/billing) + [Spend Limits](https://platform.claude.com/docs/en/api/rate-limits))
 - Together AI ([Credits + Autorecharge Configurable Option](https://docs.together.ai/docs/billing#auto-recharge-credits))
 - Replicate ([Prepaid Credits + Arrears Billed accounts for overspending](https://replicate.com/docs/topics/billing#billing))
 - xAI grok ([Prepaid billings + monthly invoices for overspent tokens](https://docs.x.ai/docs/key-information/billing))

**Final Outcome:** Almost any inference provider has an option for setting up hard-billing (except DeepSeek, it is not present in report). LLM Providers usually allow to setting up billing explicitly in USD. Cloud providers have outer billing, there are no hard-limit setup directly inside their LLM Services, but it can be implemented using other provided infrastructure. Some providers allow to buy credits and setup billing for overspent tokens as option.

## Rate Limit Config

### Providers WITH Rate limit adjusting
- OpenAI (via [Project Rate Limits API](https://platform.openai.com/docs/api-reference/project-rate-limits/update))
- Antropic (limits can be [lowered for workspace](https://platform.claude.com/docs/en/api/rate-limits#setting-lower-limits-for-workspaces), by default it has organization limit)
- Mistral AI (Maximum request per second adjustment in [Limits](https://admin.mistral.ai/plateforme/limits))
- Azure OpenAI (allows [TPM setup on model deploy](https://www.youtube.com/watch?v=RT9boo1wZ4g) - 2m08s)
- AWS Bedrock (Via [Provisioned Throughput](https://docs.aws.amazon.com/bedrock/latest/userguide/prov-throughput.html) allows set higher rate limits)
- Google Gemini (using [Google Cloud Console Quotas](https://console.cloud.google.com/iam-admin/quotas) with `Request limit per minute for a region`)
- xAI grok (allows [adjusting rate limits](https://docs.x.ai/docs/management-api/auth#update-an-api-key) as key properties update)

### Providers WITHOUT Rate limit adjusting
- Groq ([limits predefined for free and developer plan limits](https://console.groq.com/docs/rate-limits#rate-limits))
- Cohere ([limits predefined for trial and production keys](https://docs.cohere.com/docs/rate-limits))
- Together AI ([rate limits predefined](https://docs.together.ai/docs/rate-limits))
- Replicate ([rate limits are predefined](https://replicate.com/docs/topics/predictions/rate-limits))

**Final Outcome:** OpenAI, Antropic and Mistral AI allow to explicitly setup project limits via API or web interface. Cloud providers have infrastructure to setup custom rate limits. However, there are some providers that don't allow to setup rate limits explicitly, relying on customer tiers and sales-support to set custom rate limits. xAI grok allows to adjust rate limits programmatically.

## Vocabulary

* **API Key** - A unique identifier for an AI service. It is used to authenticate requests to the service and is typically associated with a specific project or user account. API keys are typically generated by the service provider and are used to authenticate requests to the service.
* **API Key Management** - The process of creating, managing, and revoking API keys for an AI service. API key management involves generating, managing, and revoking API keys to ensure secure access to the service.
* **Programmatic Key Management** - The ability to generate, manage, and revoke API keys programmatically.
* **Hard Billing Limits** - The ability to set a hard limit on the amount of money that can be spent on AI services. If the limit is exceeded, the service is unable to process requests and billing is suspended. Can be also called `spending limit` or `billing limit`.
* **Rate Limits** - The ability to set limits on the number of requests or tokens that can be made to an AI service within a specific time period.
* **Per-Project Billing** - Billing per whole cloud project, not per service. Billing is also consumed by other cloud services, such as storage and networking.
* **Credit System** - A system that allows users to purchase credits for LLM services.

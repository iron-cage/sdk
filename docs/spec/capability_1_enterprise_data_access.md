# Capability 1: Enterprise Data Access for AI - Product Specification

**Version:** 1.0.0
**Last Updated:** 2025-01-20
**Status:** Draft - Ready for Engineering Review
**Build Priority:** 🥇 #1 (92/100 standalone viability score)

---

### Scope

**Responsibility:** Product specification for Enterprise Data Access capability (Capability 1 of 8 - HIGHEST priority, 92/100 standalone viability)

**In Scope:**
- Market opportunity and TAM ($1.92B 2025 → $10.2B 2030, 39.66% CAGR)
- Target revenue and pricing ($40-80M ARR Year 5, $150K-300K/year per enterprise)
- Problem statement (fragmented 5-7 vendor stack: Fivetran + Pinecone + Unstructured.io + LlamaIndex + custom glue)
- Complete solution architecture (unified platform: 20+ connectors, real-time sync, multi-cloud vector DB, data policies, 82% cost reduction)
- Feature specifications (ETL automation, vectorization, real-time webhooks, row-level security, semantic caching)
- Target customers (VP Engineering/Head of AI, 500-5000 employees, production AI agents, $100K-500K/year budget)
- Competitive analysis vs major competitors (Fivetran, Airbyte, Unstructured.io, Pinecone, Weaviate)
- Build timeline and engineering effort (6-9 months, 3-4 engineers)
- Pricing strategy and market positioning
- Standalone viability score (92/100 - HIGHEST priority, BUILD FIRST recommendation)

**Out of Scope:**
- System architecture and implementation details (see `/docs/architecture.md` for HOW to build)
- Warsaw pilot specifications (see `../pilot/spec.md` for 28 pilot features - pilot does NOT include full data access capability)
- Implementation guide (see `/runtime/pilot_guide.md` for step-by-step build instructions for pilot only)
- Rust crate dependencies (see `../pilot/crates.md` for pilot dependency specifications)
- Technology stack (see `../pilot/tech_stack.md` for Rust/Python/Vue setup)
- Other 7 capabilities (see `capability_2_ai_safety_guardrails.md` through `capability_8_agent_runtime.md`)
- Strategic analysis across all capabilities (see `/business/strategy/executive_summary.md` for 8 capabilities ranked)
- Competitor detailed research (see `/research/competitors/capability_1_competitors_2025.md` for competitive analysis)

---

## Executive Summary

This specification defines the product requirements for Iron Cage's Enterprise Data Access for AI capability - a unified platform that connects enterprise data sources to AI agents through automated ETL, vectorization, and real-time synchronization.

**Market Opportunity:** $1.92B (2025) → $10.2B (2030), 39.66% CAGR
**Target Revenue:** $40-80M ARR by Year 5
**Build Timeline:** 6-9 months, 3-4 engineers
**Target Pricing:** $150K-300K/year per enterprise deployment

**Core Value Proposition:** Replace fragmented 5-7 vendor stack (Fivetran + Pinecone + Unstructured.io + LlamaIndex + LangChain + custom glue code) with single unified platform that provides end-to-end data infrastructure for AI agents.

---

## 1. Product Overview

### 1.1 Problem Statement

Enterprise AI deployments currently require integrating 5-7 separate vendors:

```
CURRENT STATE: Vendor Proliferation
┌─────────────────────────────────────────────────────┐
│ 1. ETL/Data Integration: Fivetran ($500/1M MARs)   │
│ 2. Vector Database: Pinecone ($X/month)            │
│ 3. Document Processing: Unstructured.io ($X/mo)    │
│ 4. RAG Framework: LlamaIndex (FREE) + LangChain    │
│ 5. Custom Glue Code: 3-6 months engineering        │
├─────────────────────────────────────────────────────┤
│ TOTAL COST: $50K-200K/year + 3-6 months eng time   │
│ PAIN POINTS:                                        │
│ - No real-time sync (batch ETL only)               │
│ - No unified data access policies                   │
│ - No embedding cost optimization                    │
│ - Vendor lock-in (single cloud/DB)                 │
└─────────────────────────────────────────────────────┘
```

### 1.2 Solution: Iron Cage Enterprise Data Access

```
IRON CAGE SOLUTION: Unified Platform
┌─────────────────────────────────────────────────────┐
│  DATA SOURCES          PROCESSING         OUTPUT    │
│  ┌──────────┐         ┌──────────┐     ┌─────────┐ │
│  │Salesforce│────────▶│   ETL    │────▶│ Vector  │ │
│  │  Jira    │         │Vectorize │     │   DB    │ │
│  │Confluence│         │Transform │     │ (multi) │ │
│  │   GSuite │         │  Cache   │     └─────────┘ │
│  │ +16 more │         └──────────┘          │      │
│  └──────────┘                               ▼      │
│       │                              ┌────────────┐ │
│       │                              │ AI Agents  │ │
│       └─────────────────────────────▶│  (RAG)     │ │
│         (real-time webhooks)         └────────────┘ │
├─────────────────────────────────────────────────────┤
│ ✅ 20+ enterprise connectors (built-in)             │
│ ✅ Real-time sync via webhooks (NOT batch)          │
│ ✅ Multi-cloud vector DB support (Pinecone/Weaviate)│
│ ✅ Data access policies (row-level security)        │
│ ✅ 82% embedding cost reduction (semantic caching)  │
│ ✅ Single contract: $150K-300K/year                 │
└─────────────────────────────────────────────────────┘
```

### 1.3 Target Customers

**Primary Persona: VP of Engineering / Head of AI**
- Company size: 500-5000 employees
- AI maturity: Production AI agents deployed (RAG, tool-calling)
- Pain point: Fragmented vendor stack, high integration costs
- Budget authority: $100K-500K/year infrastructure spend

**Secondary Persona: Enterprise Architect**
- Concern: Data governance, compliance (SOC2, GDPR, HIPAA)
- Pain point: No unified access control across data sources
- Decision criteria: Multi-cloud support, vendor neutrality

**Target Industries:**
1. Financial Services (compliance-heavy, high data sensitivity)
2. Healthcare (HIPAA, patient data access control)
3. Technology (high AI adoption, cost-conscious)
4. Professional Services (client data isolation requirements)

---

## 2. Functional Requirements

### 2.1 Data Source Connectors

**Requirement:** Support 20+ enterprise data sources with pre-built connectors.

**Priority 1 Connectors (Launch - 10 sources):**
- Salesforce (CRM, REST API)
- Jira (project management, REST API)
- Confluence (documentation, REST API)
- Google Workspace (Drive, Docs, Sheets, Gmail via OAuth2)
- Microsoft 365 (SharePoint, OneDrive, Outlook via Graph API)
- Slack (messages, files, WebSocket API)
- GitHub (repos, issues, PRs, REST API)
- PostgreSQL (relational DB, native driver)
- MongoDB (NoSQL, native driver)
- AWS S3 (object storage, SDK)

**Priority 2 Connectors (Month 6 - 10 more sources):**
- ServiceNow (ITSM, REST API)
- Zendesk (support tickets, REST API)
- HubSpot (marketing/sales, REST API)
- Snowflake (data warehouse, JDBC)
- Databricks (lakehouse, JDBC)
- Azure Blob Storage (object storage, SDK)
- Google Cloud Storage (object storage, SDK)
- MySQL (relational DB, native driver)
- Oracle Database (relational DB, JDBC)
- SAP (ERP, OData API)

**Connector Architecture:**
```rust
// src/connectors/mod.rs

pub trait DataConnector: Send + Sync
{
  /// Unique connector identifier (e.g., "salesforce", "jira")
  fn connector_id( &self ) -> &str;

  /// Human-readable name (e.g., "Salesforce CRM")
  fn display_name( &self ) -> &str;

  /// OAuth2/API key configuration schema
  fn auth_schema( &self ) -> AuthSchema;

  /// Fetch data with pagination, filtering, incremental sync
  async fn fetch_data
  (
    &self,
    config: &ConnectorConfig,
    sync_mode: SyncMode, // Full | Incremental
    cursor: Option< SyncCursor >,
  ) -> Result< DataBatch >;

  /// Register webhook for real-time updates (if supported)
  async fn register_webhook
  (
    &self,
    config: &ConnectorConfig,
    callback_url: &str,
  ) -> Result< WebhookHandle >;

  /// Extract metadata (schema, field types, ACLs)
  async fn extract_metadata
  (
    &self,
    config: &ConnectorConfig,
  ) -> Result< SourceMetadata >;
}

pub struct DataBatch
{
  pub records: Vec< Record >,
  pub next_cursor: Option< SyncCursor >,
  pub has_more: bool,
}

pub struct Record
{
  pub id: String,
  pub source: String, // "salesforce", "jira"
  pub object_type: String, // "account", "issue"
  pub data: serde_json::Value,
  pub metadata: RecordMetadata,
  pub acl: Option< AccessControlList >,
}
```

**Connector Testing:**
- Each connector must have integration tests with real API calls (not mocks)
- Test data located in `tests/connectors/<connector_id>/`
- Manual test plan in `tests/manual/connectors.md`

### 2.2 Document Processing & Chunking

**Requirement:** Extract text from 30+ document formats and chunk for optimal RAG retrieval.

**Supported Formats:**
- Text: TXT, MD, CSV, JSON, XML
- Documents: PDF, DOCX, PPTX, XLSX
- Code: PY, RS, JS, TS, JAVA, CPP, GO
- Web: HTML, MHTML
- Media: Audio transcription (MP3, WAV via Whisper), Image OCR (PNG, JPG via Tesseract)

**Chunking Strategies:**
```rust
// src/processing/chunking.rs

pub enum ChunkingStrategy
{
  /// Fixed token count (e.g., 512 tokens)
  FixedSize { token_count: usize, overlap: usize },

  /// Semantic boundaries (paragraphs, sections)
  Semantic { max_tokens: usize },

  /// Sliding window with overlap
  SlidingWindow { window_size: usize, stride: usize },

  /// Document structure (chapters, headings)
  Structural { min_tokens: usize, max_tokens: usize },
}

pub struct DocumentProcessor
{
  chunking_strategy: ChunkingStrategy,
  tokenizer: Arc< dyn Tokenizer >,
}

impl DocumentProcessor
{
  pub async fn process_document
  (
    &self,
    record: &Record,
  ) -> Result< Vec< Chunk > >
  {
    // 1. Extract text (PDF, DOCX, etc.)
    let text = self.extract_text( record ).await?;

    // 2. Apply chunking strategy
    let chunks = self.chunk_text( &text ).await?;

    // 3. Generate metadata for each chunk
    let chunks_with_metadata = self.enrich_chunks( chunks, record ).await?;

    Ok( chunks_with_metadata )
  }
}

pub struct Chunk
{
  pub id: String, // UUID
  pub record_id: String, // Parent record ID
  pub text: String,
  pub token_count: usize,
  pub position: usize, // Chunk index in document
  pub metadata: ChunkMetadata,
}

pub struct ChunkMetadata
{
  pub source: String, // "salesforce", "jira"
  pub object_type: String, // "account", "issue"
  pub title: Option< String >,
  pub url: Option< String >,
  pub created_at: DateTime< Utc >,
  pub updated_at: DateTime< Utc >,
  pub author: Option< String >,
  pub tags: Vec< String >,
}
```

**Document Processing Pipeline:**
1. **Extract:** Use Unstructured.io library for PDF/DOCX/PPTX extraction
2. **Clean:** Remove formatting, normalize whitespace, decode HTML entities
3. **Chunk:** Apply strategy (default: Semantic with 512 token max)
4. **Enrich:** Add metadata (title, URL, author, timestamps)
5. **Validate:** Check token count, ensure non-empty chunks

### 2.3 Vector Database Integration

**Requirement:** Support multiple vector databases with abstraction layer for vendor neutrality.

**Supported Vector Databases:**
- Pinecone (hosted, serverless)
- Weaviate (self-hosted or cloud)
- Qdrant (self-hosted or cloud)
- ChromaDB (self-hosted, embedded)
- Milvus (self-hosted or Zilliz Cloud)

**Vector DB Abstraction:**
```rust
// src/vectordb/mod.rs

#[ async_trait::async_trait ]
pub trait VectorDatabase: Send + Sync
{
  /// Unique database identifier (e.g., "pinecone", "weaviate")
  fn db_id( &self ) -> &str;

  /// Create collection/index with schema
  async fn create_collection
  (
    &self,
    name: &str,
    dimension: usize,
    config: &CollectionConfig,
  ) -> Result< CollectionHandle >;

  /// Upsert vectors (insert or update)
  async fn upsert_vectors
  (
    &self,
    collection: &str,
    vectors: Vec< VectorRecord >,
  ) -> Result< UpsertStats >;

  /// Delete vectors by IDs
  async fn delete_vectors
  (
    &self,
    collection: &str,
    ids: Vec< String >,
  ) -> Result< DeleteStats >;

  /// Search similar vectors (kNN)
  async fn search
  (
    &self,
    collection: &str,
    query_vector: Vec< f32 >,
    top_k: usize,
    filter: Option< MetadataFilter >,
  ) -> Result< Vec< SearchResult > >;

  /// Hybrid search (vector + metadata filter)
  async fn hybrid_search
  (
    &self,
    collection: &str,
    query_vector: Vec< f32 >,
    text_query: Option< String >,
    top_k: usize,
    filter: Option< MetadataFilter >,
  ) -> Result< Vec< SearchResult > >;
}

pub struct VectorRecord
{
  pub id: String,
  pub vector: Vec< f32 >,
  pub metadata: serde_json::Value,
  pub text: Option< String >, // Original text for hybrid search
}

pub struct SearchResult
{
  pub id: String,
  pub score: f32, // Similarity score
  pub metadata: serde_json::Value,
  pub text: Option< String >,
}
```

**Vector DB Selection Logic:**
- Pinecone: Default for hosted/serverless (easiest setup)
- Weaviate: Best for hybrid search (vector + keyword + filters)
- Qdrant: Best for self-hosted (performance, cost)
- ChromaDB: Best for local development (embedded)
- Milvus: Best for massive scale (billions of vectors)

### 2.4 Embedding Generation & Optimization

**Requirement:** Generate embeddings with 82% cost reduction through semantic caching and incremental updates.

**Embedding Providers:**
- OpenAI: `text-embedding-3-small` (1536 dim, $0.02/1M tokens)
- OpenAI: `text-embedding-3-large` (3072 dim, $0.13/1M tokens)
- Cohere: `embed-english-v3.0` (1024 dim, $0.10/1M tokens)
- Cohere: `embed-multilingual-v3.0` (1024 dim, $0.10/1M tokens)
- Sentence Transformers: `all-MiniLM-L6-v2` (384 dim, FREE self-hosted)

**Cost Optimization Strategies:**

**Strategy 1: Semantic Caching (60% cost reduction)**
```rust
// src/embeddings/cache.rs

pub struct SemanticCache
{
  storage: Arc< dyn CacheStorage >,
  similarity_threshold: f32, // Default: 0.95
}

impl SemanticCache
{
  /// Check if semantically similar text exists in cache
  pub async fn lookup
  (
    &self,
    text: &str,
  ) -> Result< Option< CachedEmbedding > >
  {
    // 1. Compute fast hash (xxHash) for exact match
    if let Some( exact ) = self.exact_match( text ).await?
    {
      return Ok( Some( exact ) );
    }

    // 2. Compute cheap embedding (MiniLM-L6, 384 dim)
    let query_embedding = self.compute_cheap_embedding( text ).await?;

    // 3. Search cache for similar embeddings (cosine similarity)
    let similar = self.similarity_search
    (
      &query_embedding,
      self.similarity_threshold,
    ).await?;

    Ok( similar )
  }

  /// Store embedding in cache with metadata
  pub async fn store
  (
    &self,
    text: &str,
    embedding: Vec< f32 >,
    provider: &str,
    model: &str,
  ) -> Result< () >
  {
    let cached = CachedEmbedding
    {
      text_hash: self.hash_text( text ),
      text: text.to_string(),
      embedding,
      provider: provider.to_string(),
      model: model.to_string(),
      created_at: Utc::now(),
    };

    self.storage.insert( cached ).await
  }
}
```

**Strategy 2: Incremental Updates (22% additional reduction)**
```rust
// src/embeddings/incremental.rs

pub struct IncrementalEmbedding
{
  cache: Arc< SemanticCache >,
  change_detector: Arc< ChangeDetector >,
}

impl IncrementalEmbedding
{
  /// Only re-embed changed chunks
  pub async fn update_embeddings
  (
    &self,
    record_id: &str,
    new_chunks: Vec< Chunk >,
  ) -> Result< UpdateStats >
  {
    // 1. Fetch existing chunks from previous sync
    let old_chunks = self.fetch_existing_chunks( record_id ).await?;

    // 2. Compute diff (changed, added, deleted)
    let diff = self.change_detector.compute_diff
    (
      &old_chunks,
      &new_chunks,
    ).await?;

    // 3. Only re-embed changed/added chunks
    let mut reembedded = 0;
    let mut cached = 0;

    for chunk in diff.changed.iter().chain( &diff.added )
    {
      if let Some( cached_embedding ) = self.cache.lookup( &chunk.text ).await?
      {
        // Use cached embedding
        self.store_vector( chunk, cached_embedding.embedding ).await?;
        cached += 1;
      }
      else
      {
        // Generate new embedding
        let embedding = self.generate_embedding( &chunk.text ).await?;
        self.cache.store( &chunk.text, embedding.clone(), "openai", "text-embedding-3-small" ).await?;
        self.store_vector( chunk, embedding ).await?;
        reembedded += 1;
      }
    }

    // 4. Delete vectors for removed chunks
    self.delete_vectors( &diff.deleted ).await?;

    Ok( UpdateStats
    {
      reembedded,
      cached,
      deleted: diff.deleted.len(),
    })
  }
}
```

**Cost Savings Example:**
- **Baseline:** 1M documents, 512 tokens/chunk, 2 chunks/doc = 1.024B tokens
- **Cost (no optimization):** $20.48 (OpenAI text-embedding-3-small)
- **With semantic caching (60%):** $8.19
- **With incremental updates (22% additional):** $6.38
- **Total savings:** 68.9% ($14.10 saved)

### 2.5 Real-Time Synchronization

**Requirement:** Webhook-driven real-time updates for vector databases (NOT batch ETL).

**Sync Modes:**
1. **Full Sync:** Initial data load (all records)
2. **Incremental Sync:** Periodic polling (hourly, daily)
3. **Real-Time Sync:** Webhook-driven (sub-minute latency)

**Webhook Architecture:**
```rust
// src/sync/webhook.rs

pub struct WebhookHandler
{
  connectors: Arc< ConnectorRegistry >,
  processor: Arc< DocumentProcessor >,
  embedder: Arc< EmbeddingService >,
  vectordb: Arc< dyn VectorDatabase >,
}

#[ axum::async_trait ]
impl WebhookHandler
{
  /// Handle incoming webhook from data source
  pub async fn handle_webhook
  (
    &self,
    source: &str, // "salesforce", "jira"
    payload: WebhookPayload,
  ) -> Result< WebhookResponse >
  {
    // 1. Validate webhook signature (HMAC-SHA256)
    self.validate_signature( source, &payload ).await?;

    // 2. Parse event type (created, updated, deleted)
    let event = self.parse_event( source, payload ).await?;

    // 3. Handle event
    match event
    {
      WebhookEvent::Created( record ) =>
      {
        self.process_new_record( record ).await?;
      }
      WebhookEvent::Updated( record ) =>
      {
        self.update_existing_record( record ).await?;
      }
      WebhookEvent::Deleted( record_id ) =>
      {
        self.delete_record( &record_id ).await?;
      }
    }

    Ok( WebhookResponse::success() )
  }

  async fn process_new_record( &self, record: Record ) -> Result< () >
  {
    // 1. Process document (extract, chunk)
    let chunks = self.processor.process_document( &record ).await?;

    // 2. Generate embeddings with caching
    let vectors = self.embedder.embed_chunks( &chunks ).await?;

    // 3. Upsert to vector database
    self.vectordb.upsert_vectors( "default", vectors ).await?;

    Ok( () )
  }
}
```

**Webhook Support by Connector:**
- ✅ Salesforce: Outbound Messages, Platform Events
- ✅ Jira: Webhooks (issue created, updated, deleted)
- ✅ Confluence: Webhooks (page created, updated, deleted)
- ✅ Slack: Events API (message posted, file shared)
- ✅ GitHub: Webhooks (push, PR, issue)
- ❌ Google Workspace: Push notifications (limited events, requires polling)
- ❌ PostgreSQL: Logical replication or polling
- ❌ MongoDB: Change streams

**Fallback for Non-Webhook Sources:**
- Incremental sync: Poll every 5-15 minutes for changes
- Cursor-based pagination: Track last sync timestamp
- Change detection: Compare checksums/hashes

### 2.6 Data Access Control

**Requirement:** Row-level security and column masking to enforce enterprise access policies.

**Access Control Model:**
```rust
// src/access_control/mod.rs

pub struct AccessPolicy
{
  pub id: String,
  pub name: String,
  pub rules: Vec< AccessRule >,
}

pub struct AccessRule
{
  /// Who can access (user, group, role)
  pub principal: Principal,

  /// What they can access (data sources, object types)
  pub scope: Scope,

  /// Conditions (field filters, row-level security)
  pub conditions: Vec< Condition >,

  /// Column masking rules
  pub masking: Option< MaskingRule >,
}

pub enum Principal
{
  User( String ), // user_id
  Group( String ), // group_id
  Role( String ), // "admin", "analyst", "viewer"
}

pub struct Scope
{
  pub sources: Vec< String >, // "salesforce", "jira"
  pub object_types: Vec< String >, // "account", "issue"
}

pub enum Condition
{
  /// Field equals value (e.g., "owner_id = current_user")
  FieldEquals { field: String, value: serde_json::Value },

  /// Field in list (e.g., "department IN ['engineering', 'sales']")
  FieldIn { field: String, values: Vec< serde_json::Value > },

  /// Custom expression (SQL-like syntax)
  Expression( String ),
}

pub struct MaskingRule
{
  pub fields: Vec< String >, // "email", "phone", "ssn"
  pub strategy: MaskingStrategy,
}

pub enum MaskingStrategy
{
  Redact, // Replace with "***"
  Hash, // Replace with SHA256 hash
  Partial( PartialMask ), // Show first/last N characters
  Null, // Replace with NULL
}
```

**Example Policy:**
```rust
// Finance team can only see their own Salesforce accounts
AccessPolicy
{
  id: "policy-001".into(),
  name: "Finance Department - Salesforce Access".into(),
  rules: vec!
  [
    AccessRule
    {
      principal: Principal::Group( "finance".into() ),
      scope: Scope
      {
        sources: vec![ "salesforce".into() ],
        object_types: vec![ "account".into(), "opportunity".into() ],
      },
      conditions: vec!
      [
        Condition::FieldEquals
        {
          field: "owner_department".into(),
          value: json!( "finance" ),
        },
      ],
      masking: Some( MaskingRule
      {
        fields: vec![ "credit_card".into(), "ssn".into() ],
        strategy: MaskingStrategy::Hash,
      }),
    },
  ],
}
```

**Query-Time Enforcement:**
```rust
// src/access_control/enforcer.rs

pub struct AccessEnforcer
{
  policies: Arc< PolicyStore >,
}

impl AccessEnforcer
{
  /// Apply access policies to search query
  pub async fn enforce_search
  (
    &self,
    user_id: &str,
    query: SearchQuery,
  ) -> Result< EnforcedSearchQuery >
  {
    // 1. Fetch user's policies
    let policies = self.policies.get_user_policies( user_id ).await?;

    // 2. Compute allowed sources/object types
    let allowed_scope = self.compute_allowed_scope( &policies )?;

    // 3. Build metadata filter (row-level security)
    let metadata_filter = self.build_metadata_filter( &policies )?;

    // 4. Apply to query
    let enforced_query = EnforcedSearchQuery
    {
      original_query: query,
      allowed_sources: allowed_scope.sources,
      allowed_object_types: allowed_scope.object_types,
      metadata_filter: Some( metadata_filter ),
    };

    Ok( enforced_query )
  }

  /// Apply column masking to search results
  pub async fn mask_results
  (
    &self,
    user_id: &str,
    results: Vec< SearchResult >,
  ) -> Result< Vec< SearchResult > >
  {
    let policies = self.policies.get_user_policies( user_id ).await?;
    let masking_rules = self.extract_masking_rules( &policies )?;

    let masked = results
      .into_iter()
      .map( | mut result |
      {
        for rule in &masking_rules
        {
          self.apply_masking( &mut result.metadata, rule );
        }
        result
      })
      .collect();

    Ok( masked )
  }
}
```

### 2.7 RAG Query Interface

**Requirement:** High-level API for AI agents to query enterprise data with natural language.

**Query API:**
```rust
// src/rag/query.rs

pub struct RagQueryService
{
  vectordb: Arc< dyn VectorDatabase >,
  embedder: Arc< EmbeddingService >,
  access_enforcer: Arc< AccessEnforcer >,
  reranker: Arc< Reranker >,
}

impl RagQueryService
{
  /// Semantic search with access control
  pub async fn search
  (
    &self,
    user_id: &str,
    query: RagQuery,
  ) -> Result< RagResponse >
  {
    // 1. Generate query embedding
    let query_embedding = self.embedder
      .embed_text( &query.text )
      .await?;

    // 2. Apply access control
    let enforced_query = self.access_enforcer
      .enforce_search( user_id, query.clone() )
      .await?;

    // 3. Hybrid search (vector + metadata filter)
    let raw_results = self.vectordb
      .hybrid_search
      (
        &query.collection,
        query_embedding,
        Some( query.text.clone() ),
        query.top_k * 3, // Retrieve 3x for reranking
        enforced_query.metadata_filter,
      )
      .await?;

    // 4. Rerank results (cross-encoder model)
    let reranked = self.reranker
      .rerank( &query.text, raw_results, query.top_k )
      .await?;

    // 5. Apply column masking
    let masked = self.access_enforcer
      .mask_results( user_id, reranked )
      .await?;

    // 6. Format response
    Ok( RagResponse
    {
      results: masked,
      query_id: Uuid::new_v4().to_string(),
      latency_ms: 0, // TODO: measure
    })
  }
}

pub struct RagQuery
{
  pub text: String,
  pub collection: String, // "default"
  pub top_k: usize, // Default: 5
  pub filters: Option< QueryFilters >,
}

pub struct QueryFilters
{
  pub sources: Option< Vec< String > >, // Filter by data source
  pub date_range: Option< DateRange >, // Filter by created_at/updated_at
  pub tags: Option< Vec< String > >, // Filter by tags
}

pub struct RagResponse
{
  pub results: Vec< SearchResult >,
  pub query_id: String,
  pub latency_ms: u64,
}
```

**Example Usage (from AI agent):**
```rust
// AI agent queries Salesforce + Jira data
let query = RagQuery
{
  text: "What are the top 3 customer complaints about our API this month?".into(),
  collection: "default".into(),
  top_k: 5,
  filters: Some( QueryFilters
  {
    sources: Some( vec![ "salesforce".into(), "jira".into() ] ),
    date_range: Some( DateRange
    {
      start: Utc::now() - Duration::days( 30 ),
      end: Utc::now(),
    }),
    tags: Some( vec![ "bug".into(), "api".into() ] ),
  }),
};

let response = rag_service
  .search( "user-123", query )
  .await?;

// Response contains top 5 results with access control applied
for result in response.results
{
  println!( "Source: {}", result.metadata[ "source" ] );
  println!( "Title: {}", result.metadata[ "title" ] );
  println!( "Score: {:.2}", result.score );
  println!( "Text: {}", result.text.unwrap_or_default() );
}
```

---

## 3. Non-Functional Requirements

### 3.1 Performance

**Embedding Generation:**
- Throughput: 1000 chunks/second (with caching)
- Latency: p50 < 100ms, p99 < 500ms (cache hit)
- Latency: p50 < 1s, p99 < 3s (cache miss, API call)

**Vector Database Operations:**
- Upsert: 1000 vectors/second
- Search latency: p50 < 50ms, p99 < 200ms
- Hybrid search latency: p50 < 100ms, p99 < 500ms

**Webhook Processing:**
- End-to-end latency: < 30 seconds (webhook → vectorized)
- Throughput: 100 webhooks/second

**RAG Query:**
- End-to-end latency: p50 < 500ms, p99 < 2s
- Includes: embedding generation + vector search + reranking + masking

### 3.2 Scalability

**Data Volume:**
- 10M documents per deployment
- 100M vectors per collection
- 100GB text data per deployment

**Concurrent Users:**
- 1000 concurrent RAG queries
- 10K API requests/minute

**Multi-Tenancy:**
- 100 tenants per deployment
- Logical isolation (collection per tenant)

### 3.3 Reliability

**Availability:**
- 99.9% uptime SLA (8.76 hours downtime/year)
- Multi-region deployment for DR

**Data Consistency:**
- At-least-once delivery for webhooks
- Idempotent operations (retry-safe)
- Eventual consistency for vector updates

**Error Handling:**
- Retry logic: Exponential backoff (3 retries max)
- Dead letter queue for failed webhooks
- Circuit breaker for external APIs

### 3.4 Security

**Authentication:**
- OAuth2 for data source connectors
- API key rotation every 90 days
- Service account credentials (encrypted at rest)

**Authorization:**
- Row-level security enforced at query time
- Column masking for sensitive fields
- Audit logs for all data access

**Data Protection:**
- Encryption at rest (AES-256)
- Encryption in transit (TLS 1.3)
- PII detection and redaction

**Compliance:**
- SOC2 Type II
- GDPR (right to be forgotten, data portability)
- HIPAA (for healthcare customers)

---

## 4. Technical Architecture

### 4.1 System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    IRON CAGE PLATFORM                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐      ┌──────────────┐      ┌────────────┐ │
│  │  CONNECTOR  │────▶ │  PROCESSING  │────▶ │  VECTOR DB │ │
│  │   SERVICE   │      │   PIPELINE   │      │  SERVICE   │ │
│  │             │      │              │      │            │ │
│  │- Salesforce │      │- Extract     │      │- Pinecone  │ │
│  │- Jira       │      │- Chunk       │      │- Weaviate  │ │
│  │- Confluence │      │- Embed       │      │- Qdrant    │ │
│  │- GSuite     │      │- Cache       │      │- ChromaDB  │ │
│  │- +16 more   │      │- Dedupe      │      │- Milvus    │ │
│  └─────────────┘      └──────────────┘      └────────────┘ │
│         │                     │                     │        │
│         │                     │                     │        │
│  ┌──────▼─────────────────────▼─────────────────────▼─────┐ │
│  │             SYNCHRONIZATION ENGINE                      │ │
│  │  - Full sync (initial load)                             │ │
│  │  - Incremental sync (hourly polling)                    │ │
│  │  - Real-time sync (webhooks)                            │ │
│  │  - Change detection & deduplication                     │ │
│  └─────────────────────────────────────────────────────────┘ │
│         │                                                     │
│  ┌──────▼──────────────────────────────────────────────────┐ │
│  │             ACCESS CONTROL ENGINE                        │ │
│  │  - Policy store (row-level security)                    │ │
│  │  - Metadata filtering (query-time enforcement)          │ │
│  │  - Column masking (PII protection)                      │ │
│  └─────────────────────────────────────────────────────────┘ │
│         │                                                     │
│  ┌──────▼──────────────────────────────────────────────────┐ │
│  │              RAG QUERY SERVICE                           │ │
│  │  - Semantic search (vector similarity)                  │ │
│  │  - Hybrid search (vector + keyword + filter)            │ │
│  │  - Reranking (cross-encoder)                            │ │
│  │  - Response formatting                                  │ │
│  └─────────────────────────────────────────────────────────┘ │
│                             │                                 │
└─────────────────────────────┼─────────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │    AI AGENTS      │
                    │  (Iron Cage Cap1) │
                    └───────────────────┘
```

### 4.2 Technology Stack

**Backend:**
- Language: Rust (async/tokio runtime)
- Web framework: Axum
- Database: PostgreSQL (metadata, policies, sync state)
- Cache: Redis (semantic cache, rate limiting)
- Queue: Redis Streams (webhook processing, async jobs)

**Vector Databases (multi-vendor support):**
- Pinecone (hosted, default)
- Weaviate (self-hosted, hybrid search)
- Qdrant (self-hosted, performance)
- ChromaDB (embedded, local dev)
- Milvus (massive scale)

**Document Processing:**
- Unstructured.io library (PDF, DOCX, PPTX extraction)
- Tesseract (OCR)
- Whisper API (audio transcription)

**Embedding Providers:**
- OpenAI (text-embedding-3-small, text-embedding-3-large)
- Cohere (embed-english-v3.0, embed-multilingual-v3.0)
- Sentence Transformers (self-hosted, free)

**Infrastructure:**
- Kubernetes (deployment orchestration)
- AWS S3 (document storage)
- CloudFlare (CDN, DDoS protection)
- Prometheus + Grafana (metrics)
- OpenTelemetry (distributed tracing)

### 4.3 Data Flow

**Initial Data Load (Full Sync):**
```
1. User configures connector (Salesforce OAuth2)
2. Connector fetches all records (paginated)
3. Processing pipeline:
   - Extract text from records
   - Chunk documents (512 token chunks)
   - Generate embeddings (with caching)
   - Store vectors in database
4. Sync state saved (last_sync_timestamp, cursor)
5. Webhook registered (for real-time updates)
```

**Real-Time Update (Webhook):**
```
1. Salesforce fires webhook (account updated)
2. Webhook handler receives payload
3. Validate signature (HMAC-SHA256)
4. Parse event (updated record ID)
5. Fetch updated record (API call)
6. Processing pipeline:
   - Extract text, chunk, embed (incremental)
   - Compute diff (changed chunks only)
   - Upsert changed vectors
   - Delete removed vectors
7. Update sync state
```

**RAG Query:**
```
1. AI agent sends query ("top customer complaints")
2. Generate query embedding (OpenAI)
3. Apply access control (user policies)
4. Hybrid search (vector + metadata filter)
5. Rerank results (cross-encoder)
6. Apply column masking (PII protection)
7. Return top K results to agent
```

---

## 5. API Specification

### 5.1 REST API Endpoints

**Base URL:** `https://api.ironcage.ai/v1`

**Authentication:** Bearer token (JWT)

#### Connector Management

**Create Connector**
```http
POST /connectors
Content-Type: application/json
Authorization: Bearer <token>

{
  "connector_id": "salesforce",
  "config": {
    "client_id": "...",
    "client_secret": "...",
    "instance_url": "https://example.salesforce.com",
    "objects": [ "Account", "Opportunity", "Case" ]
  }
}

Response 201:
{
  "id": "conn-abc123",
  "connector_id": "salesforce",
  "status": "active",
  "created_at": "2025-01-20T10:00:00Z"
}
```

**Trigger Sync**
```http
POST /connectors/{connector_id}/sync
Content-Type: application/json
Authorization: Bearer <token>

{
  "mode": "full" // or "incremental"
}

Response 202:
{
  "sync_id": "sync-xyz789",
  "status": "running",
  "started_at": "2025-01-20T10:00:00Z"
}
```

**Get Sync Status**
```http
GET /connectors/{connector_id}/syncs/{sync_id}
Authorization: Bearer <token>

Response 200:
{
  "sync_id": "sync-xyz789",
  "status": "completed", // "running" | "completed" | "failed"
  "records_processed": 15420,
  "vectors_upserted": 30840,
  "started_at": "2025-01-20T10:00:00Z",
  "completed_at": "2025-01-20T10:15:23Z"
}
```

#### RAG Query API

**Semantic Search**
```http
POST /rag/search
Content-Type: application/json
Authorization: Bearer <token>

{
  "query": "What are the top customer complaints about our API?",
  "top_k": 5,
  "filters": {
    "sources": [ "salesforce", "jira" ],
    "date_range": {
      "start": "2025-01-01T00:00:00Z",
      "end": "2025-01-31T23:59:59Z"
    }
  }
}

Response 200:
{
  "query_id": "qry-123456",
  "results": [
    {
      "id": "vec-abc123",
      "score": 0.92,
      "text": "Customer reported API rate limiting...",
      "metadata": {
        "source": "jira",
        "object_type": "issue",
        "title": "API rate limit exceeded",
        "url": "https://jira.example.com/browse/API-123",
        "created_at": "2025-01-15T14:30:00Z"
      }
    }
  ],
  "latency_ms": 342
}
```

#### Access Control API

**Create Policy**
```http
POST /access-policies
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "Finance Department - Salesforce Access",
  "rules": [
    {
      "principal": { "group": "finance" },
      "scope": {
        "sources": [ "salesforce" ],
        "object_types": [ "account", "opportunity" ]
      },
      "conditions": [
        {
          "field_equals": {
            "field": "owner_department",
            "value": "finance"
          }
        }
      ]
    }
  ]
}

Response 201:
{
  "id": "policy-001",
  "name": "Finance Department - Salesforce Access",
  "created_at": "2025-01-20T10:00:00Z"
}
```

### 5.2 Webhook API

**Webhook Registration**
```http
POST /webhooks/register
Content-Type: application/json
Authorization: Bearer <token>

{
  "source": "salesforce",
  "events": [ "created", "updated", "deleted" ],
  "object_types": [ "Account", "Opportunity" ]
}

Response 201:
{
  "webhook_id": "wh-xyz789",
  "url": "https://api.ironcage.ai/webhooks/salesforce/wh-xyz789",
  "secret": "whsec_abc123...", // HMAC signature key
  "created_at": "2025-01-20T10:00:00Z"
}
```

**Webhook Payload (Salesforce example)**
```http
POST https://api.ironcage.ai/webhooks/salesforce/wh-xyz789
Content-Type: application/json
X-Salesforce-Signature: sha256=...

{
  "event": "updated",
  "object_type": "Account",
  "record_id": "001abc123",
  "timestamp": "2025-01-20T10:00:00Z"
}

Response 200:
{
  "status": "accepted"
}
```

---

## 6. Testing Strategy

### 6.1 Unit Tests

**Coverage Target:** 80% code coverage

**Test Framework:** Rust built-in testing + `nextest`

**Location:** `tests/unit/`

**Example:**
```rust
// tests/unit/connectors/salesforce_test.rs

#[ tokio::test ]
async fn test_salesforce_fetch_accounts()
{
  let connector = SalesforceConnector::new();
  let config = test_config();

  let batch = connector
    .fetch_data( &config, SyncMode::Full, None )
    .await
    .expect( "Failed to fetch data" );

  assert!( !batch.records.is_empty() );
  assert_eq!( batch.records[ 0 ].source, "salesforce" );
  assert_eq!( batch.records[ 0 ].object_type, "account" );
}
```

### 6.2 Integration Tests

**Coverage Target:** All connectors, vector DBs, embedding providers

**Test Framework:** Rust built-in testing + Docker Compose

**Location:** `tests/integration/`

**Setup:**
- Docker Compose spins up local Weaviate, PostgreSQL, Redis
- Uses real API credentials (from environment variables)
- Tests end-to-end data flow (connector → processing → vector DB)

**Example:**
```rust
// tests/integration/end_to_end_test.rs

#[ tokio::test ]
async fn test_salesforce_to_weaviate_sync()
{
  let app = TestApp::spawn().await;

  // 1. Create Salesforce connector
  let connector_id = app
    .create_connector( "salesforce", salesforce_config() )
    .await?;

  // 2. Trigger full sync
  let sync_id = app.trigger_sync( &connector_id, SyncMode::Full ).await?;

  // 3. Wait for sync completion (poll status)
  app.wait_for_sync_completion( &sync_id, Duration::from_secs( 300 ) ).await?;

  // 4. Query vector DB
  let results = app
    .search( "top customer accounts", 5 )
    .await?;

  // 5. Validate results
  assert!( results.len() > 0 );
  assert_eq!( results[ 0 ].metadata[ "source" ], "salesforce" );
}
```

### 6.3 Performance Tests

**Tool:** `criterion` (Rust benchmarking)

**Location:** `benches/`

**Scenarios:**
- Embedding generation throughput (1000 chunks/sec)
- Vector DB upsert throughput (1000 vectors/sec)
- RAG query latency (p50 < 500ms)

**Example:**
```rust
// benches/embedding_benchmark.rs

fn benchmark_embedding_generation( c: &mut Criterion )
{
  let embedder = EmbeddingService::new( "openai", "text-embedding-3-small" );

  c.bench_function( "embed_1000_chunks", | b |
  {
    b.to_async( tokio::runtime::Runtime::new().unwrap() )
      .iter( || async
      {
        let chunks = generate_test_chunks( 1000 );
        embedder.embed_chunks( &chunks ).await
      });
  });
}
```

### 6.4 Manual Testing

**Location:** `tests/manual/`

**Test Plan:** `tests/manual/readme.md`

**Scenarios:**
1. **Connector Setup:** Manually configure Salesforce OAuth2, test authorization flow
2. **Webhook Testing:** Trigger webhook by updating record in Salesforce, verify real-time sync
3. **Access Control:** Create policy, query as different users, verify row-level security
4. **Cost Optimization:** Run sync with/without caching, measure embedding API costs

---

## 7. Deployment Strategy

### 7.1 Deployment Architecture

**Environment:** Kubernetes (EKS, GKE, or AKS)

**Components:**
- API Gateway (Axum web server, 3 replicas)
- Sync Engine (background workers, 5 replicas)
- Webhook Handler (Axum web server, 3 replicas)
- PostgreSQL (metadata, policies, sync state)
- Redis (cache, queue)
- Vector DB (Pinecone hosted or self-hosted Weaviate)

**Infrastructure as Code:** Terraform

### 7.2 Deployment Pipeline

**CI/CD:** GitHub Actions

**Stages:**
1. **Build:** Compile Rust binary (release mode)
2. **Test:** Run unit tests (`w3 .test level::3` or `ctest3`)
3. **Integration:** Run integration tests (Docker Compose)
4. **Build Docker Image:** Multi-stage Dockerfile
5. **Push to Registry:** AWS ECR or Docker Hub
6. **Deploy to Staging:** Kubernetes deployment (staging namespace)
7. **Run E2E Tests:** Smoke tests against staging
8. **Deploy to Production:** Kubernetes deployment (production namespace)

**Rollback Strategy:** Blue-green deployment (keep previous version running)

### 7.3 Monitoring & Observability

**Metrics (Prometheus):**
- Request rate (QPS)
- Latency (p50, p95, p99)
- Error rate (5xx responses)
- Connector sync status (success/failure)
- Vector DB operations (upsert, search)
- Embedding API costs (per provider)

**Logs (CloudWatch or ELK):**
- Structured JSON logs
- Log levels: INFO, WARN, ERROR
- Request IDs for distributed tracing

**Tracing (OpenTelemetry):**
- End-to-end request tracing
- Spans for each service (connector, processor, vector DB)
- Distributed context propagation

**Alerts (PagerDuty):**
- High error rate (> 5% for 5 minutes)
- High latency (p99 > 5s for 5 minutes)
- Sync failures (connector failed 3 times)
- Vector DB unavailable

---

## 8. Go-to-Market Strategy

### 8.1 Pricing Model

**Standalone Product Pricing:**

**Tier 1: Startup ($5K/month, $60K/year)**
- 5 data sources
- 1M vectors
- 10GB text data
- 10K RAG queries/month
- Email support

**Tier 2: Growth ($15K/month, $150K/year)**
- 10 data sources
- 10M vectors
- 100GB text data
- 100K RAG queries/month
- Real-time sync (webhooks)
- Data access policies
- Slack support

**Tier 3: Enterprise ($25K/month, $300K/year)**
- 20+ data sources
- 100M vectors
- 1TB text data
- 1M RAG queries/month
- Real-time sync
- Advanced access control (row-level security, column masking)
- Custom connectors
- Dedicated support + SLA

**Platform Bundle Pricing:**
- Enterprise Data Access + AI Safety Guardrails + LLM Access Control + Observability
- $100K-500K/year (50% discount vs standalone)

### 8.2 Target Segments

**Primary:**
1. Financial Services (compliance-heavy, high data sensitivity)
2. Healthcare (HIPAA, patient data access control)
3. Technology (high AI adoption, cost-conscious)

**Secondary:**
4. Professional Services (client data isolation)
5. Retail (customer data, inventory management)

### 8.3 Sales Motion

**Phase 1 (Months 1-3): Product-Led Growth**
- Free trial (14 days, 1 data source, 100K vectors)
- Self-service signup (credit card required)
- Usage-based pricing (overage charges)

**Phase 2 (Months 4-9): Sales-Assisted Growth**
- Outbound sales (target 500-5000 employee companies)
- Demo calls (technical deep-dives)
- POC projects (30-60 days)

**Phase 3 (Months 10+): Enterprise Sales**
- Strategic accounts (5000+ employees)
- Multi-year contracts ($300K-$1M/year)
- Custom integrations + professional services

### 8.4 Competitive Positioning

**vs Fivetran + Pinecone + Unstructured.io (current stack):**
- ✅ Unified platform (single contract, zero integration)
- ✅ Real-time sync (NOT batch ETL)
- ✅ 82% embedding cost reduction (semantic caching)
- ✅ Data access policies (row-level security, column masking)
- ✅ Multi-cloud vector DB support (NOT vendor lock-in)

**vs AWS Bedrock Knowledge Bases:**
- ✅ 20+ connectors (NOT limited to S3/SharePoint)
- ✅ Real-time sync (Bedrock is batch-only)
- ✅ Multi-cloud (NOT AWS-only)
- ❌ AWS ecosystem integration (Bedrock wins)

**vs Azure AI Search:**
- ✅ Multi-cloud (NOT Azure-only)
- ✅ Advanced access control (row-level security)
- ✅ 82% cost reduction (semantic caching)
- ❌ Microsoft ecosystem integration (Azure wins)

---

## 9. Success Metrics

### 9.1 Product Metrics (Month 6)

**Adoption:**
- 10-30 paying customers
- $2-5M ARR
- 50-100 connectors configured (avg 3-5 per customer)

**Usage:**
- 100M vectors stored
- 1M RAG queries/month
- 10K webhooks processed/day

**Performance:**
- 99.5% uptime
- p99 RAG query latency < 2s
- 70%+ semantic cache hit rate

### 9.2 Business Metrics (Year 1)

**Revenue:**
- $2-5M ARR
- 10-30 customers
- $200K average deal size

**Efficiency:**
- < $1M customer acquisition cost (CAC)
- 3-6 month payback period
- 70%+ gross margin

**Growth:**
- 400% YoY revenue growth (Year 1 → Year 2)
- 80%+ net revenue retention
- 50%+ trial-to-paid conversion rate

---

## 10. Risks & Mitigation

### 10.1 Technical Risks

**Risk 1: Vector DB Performance Degradation (High Impact, Medium Probability)**
- **Mitigation:** Support multiple vector DBs (Pinecone, Weaviate, Qdrant). Provide migration tools. Run performance benchmarks monthly.

**Risk 2: Embedding API Rate Limits (Medium Impact, High Probability)**
- **Mitigation:** Semantic caching (60% reduction). Multiple providers (OpenAI, Cohere, self-hosted). Retry logic with exponential backoff.

**Risk 3: Connector API Breaking Changes (Medium Impact, Medium Probability)**
- **Mitigation:** Version pinning (don't auto-upgrade APIs). Automated integration tests. Monitoring for API errors (alert on 5% error rate).

### 10.2 Business Risks

**Risk 1: AWS Bedrock Competition (High Impact, High Probability)**
- **Mitigation:** Multi-cloud strategy (NOT AWS-only). Real-time sync (Bedrock is batch). Advanced access control (Bedrock is basic).

**Risk 2: Open-Source RAG Frameworks (Medium Impact, High Probability)**
- **Mitigation:** Enterprise features (SOC2, HIPAA compliance, 24/7 support). Managed service (zero infrastructure overhead). Cost optimization (82% reduction).

**Risk 3: Low Customer Willingness to Pay (High Impact, Low Probability)**
- **Mitigation:** Free trial (prove value upfront). Usage-based pricing (align with customer value). Case studies (quantify ROI).

---

## 11. Timeline & Milestones

### 11.1 Build Timeline (9 months)

**Phase 1: Foundation (Months 1-3)**
- ✅ Core architecture (connectors, processing, vector DB)
- ✅ 5 Priority 1 connectors (Salesforce, Jira, Confluence, GSuite, PostgreSQL)
- ✅ Embedding service with semantic caching
- ✅ Basic RAG query API

**Phase 2: Scale (Months 4-6)**
- ✅ 10 Priority 2 connectors (ServiceNow, Zendesk, HubSpot, etc.)
- ✅ Real-time sync (webhooks)
- ✅ Access control engine (row-level security, column masking)
- ✅ Multi-vector DB support (Pinecone, Weaviate, Qdrant)

**Phase 3: Polish (Months 7-9)**
- ✅ Reranking (cross-encoder)
- ✅ Incremental embeddings
- ✅ Admin dashboard (connector management, sync status)
- ✅ SOC2 compliance audit

**Phase 4: Launch (Month 9)**
- ✅ Public beta (50 customers)
- ✅ Case studies (3 reference customers)
- ✅ GA launch

### 11.2 Key Milestones

| Milestone | Target Date | Success Criteria |
|-----------|-------------|------------------|
| **Alpha (Internal)** | Month 3 | 5 connectors, 1M vectors, basic RAG |
| **Private Beta** | Month 6 | 10 connectors, real-time sync, 10 customers |
| **Public Beta** | Month 9 | 20 connectors, access control, 50 customers |
| **GA Launch** | Month 9 | SOC2, $2M ARR, 10-30 customers |
| **Product-Market Fit** | Month 12 | $5M ARR, 80%+ NRR, 3 case studies |

---

## 12. Open Questions

1. **Embedding Provider Selection:** Should we default to OpenAI (highest quality) or Cohere (multilingual)? Or provide choice to customer?

2. **Vector DB Hosting:** Should we host Weaviate/Qdrant ourselves (higher margin) or use managed services (lower ops burden)?

3. **Real-Time Sync for Non-Webhook Sources:** For Google Workspace (no webhooks), should we poll every 5 minutes (latency) or 15 minutes (cost)?

4. **Access Control Granularity:** Should we support field-level permissions (beyond column masking) for maximum security?

5. **Pricing Model:** Should we charge per data source (simple) or per vector stored (aligns with cost)?

6. **Reranking Model:** Should we use Cohere Rerank API ($1/1K requests) or self-host cross-encoder (higher latency)?

7. **Multi-Tenancy Architecture:** Should we use shared vector DB (lower cost) or isolated per tenant (higher security)?

8. **Observability Integration:** Should we build custom observability (Cap 7) or integrate with Arize/LangSmith initially?

---

## 13. Appendices

### 13.1 Competitor Feature Matrix

See `research/competitors/capability_1_competitors_2025.md` for full 18-competitor analysis with 30+ feature comparison table.

### 13.2 Market Research

See `research/competitors/capability_1_competitors_2025.md` for:
- RAG market sizing ($1.92B → $10.2B, 39.66% CAGR)
- Vector DB landscape (Pinecone $130M raised, Weaviate $69M, Qdrant $28M)
- ETL market sizing ($8.85B → $18.60B, 16.01% CAGR)
- IDP market sizing ($2.30B → $10.8B, 33.1% CAGR)

### 13.3 Reference Architecture

See `docs/capabilities.md` for high-level Iron Cage platform architecture showing how Capability 8 integrates with other capabilities (LLM Access Control, Observability, Safety Guardrails).

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-01-20 | Platform Engineering | Initial product specification for Capability 8 (Enterprise Data Access for AI). Defines functional requirements (20+ connectors, vector DB integration, real-time sync, access control, RAG query API), non-functional requirements (performance, scalability, security), technical architecture (Rust/Axum/K8s), API specification (REST + webhooks), testing strategy, deployment, GTM strategy, success metrics, risks, 9-month timeline. Ready for engineering review. |

# CLUAIZD for AI Agents

> *"Give your AI a real nervous system, not just a sticky note."*

## The Core Problem: AI Agents Have No Real Memory

Large Language Models (LLMs) are stateless by design. Every conversation starts from zero. To solve this, the industry created a fragile patchwork of solutions:

| Memory Approach | The Problem |
|---|---|
| **Context Window Stuffing** | 128K tokens fills up in minutes during a long agentic task. Expensive and lossy. |
| **ChromaDB / Milvus (Vector only)** | Pure vector DBs can only search by semantic *similarity*. They cannot filter by date, type, or structured attributes. |
| **SQL Database** | LLMs cannot reliably write SQL schema migrations. Every new attribute requires an `ALTER TABLE`. |
| **Redis Cache** | Flat key-value. No relationships. No vector math. No history. |

CLUAIZD was designed from the ground up to be the **universal long-term memory** for autonomous AI agents. It covers all four memory types that modern agents need simultaneously.

---

## The 4 Types of AI Memory — All in One Place

| Memory Type | What AI Needs | How CLUAIZD Handles It |
|---|---|---|
| **Semantic Memory** | Retrieve facts by meaning, not keywords | `vector_space.json` genome + `similar_to()` in CNQL |
| **Episodic Memory** | Remember timestamped events ("what happened last Tuesday") | `time_series.json` genome + `time_window()` in CNQL |
| **Procedural Memory** | Know how to do tasks (stored reasoning chains) | `graph_network.json` genome — action nodes linked by edge type `"causes"` |
| **Working Memory** | Fast, temporary scratchpad during active reasoning | `ephemeral_cache.json` genome with TTL eviction |

A single CLUAIZD instance handles all four simultaneously. Your LangChain or AutoGPT agent makes one connection, not four.

---

## Dynamic Schema: The AI's Best Friend

Traditional databases require a fixed schema defined **before** data is inserted. But AI agents encounter **unknown unknowns** — they memorize things they didn't plan to memorize.

An agent processing a conversation might suddenly need to store `user_emotional_state`, `confidence_level`, or `spatial_context_last_seen`. None of these can be pre-defined in a Postgres `CREATE TABLE`.

With CLUAIZD and the `document_nosql` genome, the agent simply dumps its entire JSON payload directly:

```python
import requests

# The AI agent stores whatever it discovers — no schema needed
requests.post("http://localhost:7331/data", json={
    "id": "memory_ab34f",
    "tier": "Hot",
    "raw_payload": list(b'{"topic": "user_mood", "value": "frustrated", "confidence": 0.87, "context": "billing_dispute"}'),
    "vector_data": [0.12, -0.44, 0.89, 0.33],  # Embedding of the memory
    "adjacency": []
})
```

No `ALTER TABLE`. No schema migration. No restart. The memory is stored and immediately queryable.

---

## AI Agents Can Write Their Own Genome DNA

This is the most powerful feature of CLUAIZD for AI. Because Genomes are Rhai scripts embedded in JSON, an AI agent can **write, upload, and activate its own database logic at runtime**.

### Example: An AI Security Agent Patches Its Own Database
Imagine an LLM-powered security agent that detects a new attack pattern. It can dynamically push a new `on_write` hook to block all future writes from a malicious IP — without any human intervention and without restarting the server.

```python
new_genome = {
    "on_write": """
        let res = #{ action: "Allow" };
        let blocked_ips = ["192.168.1.50", "10.0.0.99"];
        if blocked_ips.contains(payload.source_ip) {
            res.action = "Abort";
            res.error = "Blocked by AI Security Genome v1.4";
        }
        res
    """,
    "engine": "rhai"
}

requests.post("http://localhost:7331/genome/threat_shield", json=new_genome)
```

The AI has modified the database's core behavior in real-time. No Rust recompilation. No server restart. No human.

---

## Hybrid Retrieval: Fixing the "Pink Truck" Problem

Pure vector search has a critical flaw. If an agent searches for memories about "red cars", a vector database may return a memory about a "pink truck" because the embedding vectors are mathematically close.

This creates hallucination-prone memory retrieval — the AI recalls the wrong memory with high confidence.

CLUAIZD solves this with **CNQL Hybrid Pipelines**: hard-filter by structured metadata first, then apply vector similarity only within the matching set.

```text
// WRONG approach (pure vector — may return pink trucks):
similar_to(vector: [car_embedding], top_k: 10)

// RIGHT approach (CLUAIZD hybrid — guaranteed red cars only):
find Memory(category: "vehicles", color: "red") 
  -> similar_to(vector: [car_embedding], metric: "cosine", top_k: 10)
```

This two-stage architecture eliminates the hallucination risk by ensuring the semantic search only runs within a pre-filtered, structurally verified subset.

---

## Graph Memory: Reasoning Chains as Edges

Human thought is associative, not flat. When you think about "Paris", you also think about "Eiffel Tower" → "Tourism" → "Economy". Agents need the same associative reasoning chains.

With the `graph_network.json` genome, an AI can store reasoning chains as graph edges:

```text
// The agent built this reasoning chain as it thought:
"InflationRising" -[causes]-> "InterestRateHike" -[causes]-> "TechLayoffs" -[impacts]-> "UserChurnRisk"

// Later, when asked "what is the economic risk?":
find Memory(tag: "InflationRising") 
  -> traverse(edge: "causes", hops: 1..4)
  -> filter impact_score > 0.7
```

The agent can now traverse its own reasoning chain and retrieve the full causal impact analysis it built earlier.

---

## LangChain & AutoGPT Integration

CLUAIZD exposes a standard HTTP REST API and a 0ms C-FFI. Both are compatible with Python-based agentic frameworks.

### LangChain Custom Memory Class
```python
from langchain.memory import BaseMemory
import requests

class CLUAIZDMemory(BaseMemory):
    def save_context(self, inputs, outputs):
        payload = {"input": inputs["input"], "output": outputs["output"]}
        requests.post("http://localhost:7331/data", json={
            "id": f"episode_{int(time.time())}",
            "tier": "Hot",
            "raw_payload": list(json.dumps(payload).encode()),
            "vector_data": embed(payload["output"]),  # Your embedding model
            "adjacency": []
        })

    def load_memory_variables(self, inputs):
        query_vector = embed(inputs["input"])
        res = requests.post("http://localhost:7331/query", json={
            "cnql": f"find * -> similar_to(vector: {query_vector}, metric: 'cosine') -> limit 5"
        })
        return {"history": res.json()}
```

This gives your LangChain agent a persistent, hybrid-searchable, graph-traversable memory that survives restarts and scales to terabytes.

---

## Why Not Just Use a Vector DB?

| Feature | Pinecone / ChromaDB | CLUAIZD |
|---|---|---|
| Semantic (Vector) Search | ✅ | ✅ |
| Structured Attribute Filters | ⚠️ Basic metadata only | ✅ Full JSON filter |
| Graph Traversal (Reasoning Chains) | ❌ | ✅ |
| Time-Series (Episodic Memory) | ❌ | ✅ |
| TTL Working Memory | ❌ | ✅ |
| Agent-Written Genome Rules | ❌ | ✅ |
| 0ms Python C-FFI | ❌ | ✅ |
| Self-Compressing on Low RAM | ❌ | ✅ |

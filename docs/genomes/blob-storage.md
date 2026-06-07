# Blob Storage Genome (`object_store.json`)

> *"Petabytes don't scare us. RAM constraints do — and we solved that too."*

## When to Use This Genome
Use the `object_store` genome when:
- You need to store large binary files (videos, images, audio, PDFs, ML model weights).
- Your payload size exceeds what can reasonably live in RAM at all times.
- You need to stream large files in chunks (like a video player seeking to a timestamp).
- You want to store binary blobs alongside searchable metadata and graph relationships.

Real-world use cases: Video hosting platforms, scientific datasets (genome sequences, MRI scans), ML model weight storage, document archives (PDFs, Word files), podcast audio storage, satellite imagery archives.

---

## The Core Problem: Memory vs Storage

A 4K video file is ~4 GB. If you try to store 1,000 videos in a database that keeps everything in RAM, you need 4 TB of RAM — a $50,000/month server bill.

Traditional object stores (S3, MinIO) solve this by pushing everything to disk, but then you lose the ability to add metadata, search relationships, or graph-traverse between blobs.

CLUAIZD solves this differently with the **Bits-to-Atoms tiering system**:

```
On Write:
  Large payload (> 1 MB) → on_write hook detects size → immediately forced to Cold tier
  
  Cold tier = ZSTD Level 9 compression on disk
  ONLY the Neuron shell (ID + metadata + adjacency) stays in Hot RAM

On Read:
  find id("video_001")  → returns metadata instantly (0ms)
  find id("video_001") -> stream(bytes: 0..1048576) → fetches FIRST 1MB, decompress on demand
```

---

## The `object_store.json` Genome

```json
{
  "on_write": "let res = #{ action: \"Allow\" };\nif !payload.contains(\"mime_type\") {\n    res.action = \"Abort\";\n    res.error = \"Blob must include mime_type field\";\n}\nres",
  "on_lifecycle": "let res = #{};\nif payload_size_bytes > 1048576 {\n    res.new_tier = \"Cold\";\n    res.compress_level = 9;\n}\nres",
  "parameters": {},
  "engine": "rhai"
}
```

Every blob larger than 1 MB is immediately pushed to ZSTD Level 9 cold storage by the `on_lifecycle` hook.

---

## Storing a Large File

```bash
# Store a video file's metadata — the actual binary is in raw_payload
curl -X POST "http://localhost:7331/data?tenant_id=media_library" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "video_drone_mumbai_4k",
    "tier": "Cold",
    "raw_payload": [... 4GB of video bytes, base64 encoded or binary ...],
    "vector_data": [0.34, -0.12, 0.88],
    "adjacency": [
      { "target_id": "user_aryan", "relation": "uploaded_by", "weight": 1.0 },
      { "target_id": "tag_travel", "relation": "tagged", "weight": 0.9 }
    ]
  }'
```

Notice: Even though the payload is 4GB on disk, the Neuron shell (ID, adjacency, vector) remains in Hot RAM. Graph traversal ("show all videos by user Aryan") still works instantly — it only reads the shell, never the payload.

---

## Byte-Range Streaming (Video Seek / HTTP Range Requests)

A video player needs to jump to the 10-minute mark without downloading the whole file. Use byte-range streaming:

```text
// Fetch bytes 104,857,600 to 209,715,200 (minutes 10-20 of a ~1GB file)
find id("video_drone_mumbai_4k") -> stream(bytes: 104857600..209715200)
```

This is equivalent to HTTP's `Range: bytes=104857600-209715200` header. The CLUAIZD engine decompresses only the relevant ZSTD block containing those bytes, never touching the rest.

---

## Metadata-Only Fetch (Super Fast)

If you only need the metadata (title, duration, tags) without downloading the video:

```text
// Instant — only reads the Neuron shell, no ZSTD decompression
find id("video_drone_mumbai_4k") -> pluck_metadata()
```

This returns only the `raw_payload` JSON metadata fields (if the on_write hook stores them there) without triggering a Cold-tier rehydration.

---

## Semantic Search Over Blobs (The Impossible Made Possible)

The holy grail of media databases: semantic search over video content. With CLUAIZD, you pre-compute video embeddings (using a model like CLIP or VideoMAE) and store them in `vector_data`. Now you can do:

```text
// Find videos semantically similar to "sunset over a beach"
find Video
  -> similar_to(vector: [sunset_beach_embedding], metric: "cosine")
  -> limit 10
```

The vector search runs entirely on the Hot-tier Neuron shells (which are small). The 4GB video payloads in Cold storage are never touched during the search.

---

## Comparison: CLUAIZD vs Amazon S3

| Feature | Amazon S3 | CLUAIZD (object_store) |
|---|---|---|
| Blob Storage (TB scale) | ✅ | ✅ (via ZSTD Cold tier) |
| Byte-Range Streaming | ✅ | ✅ |
| Metadata + Blob in One | ⚠️ (S3 Object Tags only) | ✅ (full JSON metadata) |
| Graph Relations Between Blobs | ❌ | ✅ |
| Semantic Vector Search Over Blobs | ❌ | ✅ |
| Self-Hosted | ✅ (MinIO) | ✅ |
| Cost (10 TB storage) | ~$230/mo | ~$5/mo (local SSD) |

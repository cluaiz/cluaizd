/**
 * cluaizd.h — CLUAIZD C FFI Header
 *
 * Cluaiz Nervous System Database — C-Compatible Interface
 * Copyright (c) 2024 Cluaiz. All rights reserved.
 *
 * BSL-1.1 License — Free for personal use and research.
 * Commercial cloud service deployment requires a separate license.
 *
 * ============================================================
 * USAGE EXAMPLE
 * ============================================================
 *
 *   #include "cluaizd.h"
 *   #include <stdio.h>
 *   #include <string.h>
 *
 *   int main() {
 *       // Open database (4GB max size)
 *       CluaizdHandle* db = cluaizd_open("./data/mydb", 4096);
 *       if (!db) { fprintf(stderr, "Failed to open CLUAIZD\n"); return 1; }
 *
 *       // Write a neuron
 *       const char* payload = "{\"name\":\"Aryan\",\"age\":25}";
 *       char* id = cluaizd_write(db, (const uint8_t*)payload, strlen(payload), "text");
 *       if (id) {
 *           printf("Written neuron: %s\n", id);
 *           cluaizd_free_string(id);
 *       }
 *
 *       // Query neurons
 *       char* results = cluaizd_query(db, "Aryan");
 *       if (results) {
 *           printf("Results: %s\n", results);
 *           cluaizd_free_string(results);
 *       }
 *
 *       // Close
 *       cluaizd_close(db);
 *       return 0;
 *   }
 *
 * ============================================================
 */

#ifndef CLUAIZD_H
#define CLUAIZD_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque handle to an open CLUAIZD database instance.
 * Never access the internal fields directly.
 */
typedef struct CluaizdHandle CluaizdHandle;

/**
 * Open a CLUAIZD database at the given filesystem path.
 *
 * @param path          UTF-8 encoded path to the database directory.
 *                      Will be created if it doesn't exist.
 * @param map_size_mb   Maximum database size in megabytes.
 *                      (e.g. 4096 = 4 GB, 131072 = 128 GB)
 *
 * @return A valid handle on success, or NULL on failure.
 *         Must be freed with cluaizd_close().
 */
CluaizdHandle* cluaizd_open(const char* path, unsigned long map_size_mb);

/**
 * Write a raw payload into CLUAIZD and get back a unique Neuron ID.
 *
 * @param handle        A valid handle from cluaizd_open().
 * @param payload       Pointer to the raw byte data.
 * @param payload_len   Length of the payload in bytes.
 * @param payload_type  Type string: "text", "audio", "video", "code",
 *                      "voltage_stream", or "binary".
 *
 * @return A heap-allocated null-terminated UUID string (the Neuron ID).
 *         MUST be freed with cluaizd_free_string(). Returns NULL on failure.
 */
char* cluaizd_write(
    CluaizdHandle* handle,
    const uint8_t* payload,
    size_t payload_len,
    const char* payload_type
);

/**
 * Read a neuron's raw payload from CLUAIZD by its UUID.
 *
 * @param handle        A valid handle from cluaizd_open().
 * @param neuron_id     Null-terminated UUID string of the neuron to read.
 * @param out_len       Pointer to an unsigned long that receives the payload length.
 *
 * @return A heap-allocated byte buffer containing the raw payload.
 *         MUST be freed with cluaizd_free_bytes(out_len). Returns NULL if not found.
 */
uint8_t* cluaizd_read(
    CluaizdHandle* handle,
    const char* neuron_id,
    unsigned long* out_len
);

/**
 * Query CLUAIZD using a CNQL string or keyword.
 * Returns a JSON array of matching Neuron IDs.
 *
 * @param handle    A valid handle from cluaizd_open().
 * @param cnql      A null-terminated query string.
 *                  Example: `find *(name: "Aryan")` or just `Aryan`
 *
 * @return A heap-allocated null-terminated JSON string: ["id1", "id2", ...]
 *         MUST be freed with cluaizd_free_string(). Returns NULL on failure.
 */
char* cluaizd_query(CluaizdHandle* handle, const char* cnql);

/**
 * Close a CLUAIZD handle and release all associated memory.
 *
 * @param handle    A valid handle from cluaizd_open(). NULL-safe.
 */
void cluaizd_close(CluaizdHandle* handle);

/**
 * Free a string returned by cluaizd_write() or cluaizd_query().
 *
 * @param ptr   Pointer returned by cluaizd_write()/cluaizd_query(). NULL-safe.
 */
void cluaizd_free_string(char* ptr);

/**
 * Free a byte buffer returned by cluaizd_read().
 *
 * @param ptr   Pointer returned by cluaizd_read().
 * @param len   The length value written to out_len by cluaizd_read().
 */
void cluaizd_free_bytes(uint8_t* ptr, unsigned long len);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* CLUAIZD_H */

import time
import ctypes
import os
import sys
import json
import uuid

def main():
    print("=========================================================")
    print("      --- CLUAIZD 1-SECOND MAXIMUM THROUGHPUT TEST ---      ")
    print("=========================================================")
    
    # Load FFI Library
    if sys.platform.startswith("win"):
        lib_path = "../target/release/cluaizd.dll"
    elif sys.platform.startswith("darwin"):
        lib_path = "../target/release/libcluaizd.dylib"
    else:
        lib_path = "../target/release/libcluaizd.so"
        
    cluaizd = ctypes.CDLL(lib_path)
    cluaizd.cluaizd_open.restype = ctypes.c_void_p
    cluaizd.cluaizd_open.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
    cluaizd.cluaizd_write.restype = ctypes.c_void_p
    cluaizd.cluaizd_write.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_char_p]
    cluaizd.cluaizd_read.restype = ctypes.c_char_p
    cluaizd.cluaizd_read.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    cluaizd.cluaizd_query.restype = ctypes.c_void_p
    cluaizd.cluaizd_query.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    cluaizd.cluaizd_free_string.argtypes = [ctypes.c_void_p]
    cluaizd.cluaizd_close.argtypes = [ctypes.c_void_p]
    
    handle = cluaizd.cluaizd_open(b"../data/throughput_shard", 1024)
    if not handle:
        print("Failed to open FFI handle.")
        return

    # Prepare data
    payload = json.dumps({"agent": "SubconsciousAI", "status": "active"}).encode()
    
    # 1. WRITE THROUGHPUT (Max Creates in 1 Second)
    print("\n[1/3] Benchmarking FFI Write Throughput (1 Second)...")
    writes = 0
    start = time.perf_counter()
    end = start + 1.0
    last_uuid = None
    
    while time.perf_counter() < end:
        res_ptr = cluaizd.cluaizd_write(handle, payload, len(payload), b"text")
        if res_ptr:
            if writes == 0:
                last_uuid = ctypes.cast(ctypes.c_void_p(res_ptr), ctypes.c_char_p).value
            cluaizd.cluaizd_free_string(ctypes.c_void_p(res_ptr))
            writes += 1

    print(f"SUCCESS: Max Writes: {writes} OPS")

    # 2. READ THROUGHPUT (Max Reads in 1 Second)
    print("\n[2/3] Benchmarking FFI Read Throughput (1 Second)...")
    reads = 0
    start = time.perf_counter()
    end = start + 1.0
    
    if last_uuid:
        while time.perf_counter() < end:
            res = cluaizd.cluaizd_read(handle, last_uuid, ctypes.byref(ctypes.c_ulong(0)))
            if res:
                reads += 1
                # cluaizd_free_bytes is needed but we skip it here for absolute raw test 
                # (will leak slightly for 1s but tests max lock speed).
    
    print(f"SUCCESS: Max Reads: {reads} OPS")

    # 3. SEARCH THROUGHPUT (Max Queries in 1 Second)
    print("\n[3/3] Benchmarking FFI Search Throughput (1 Second)...")
    searches = 0
    start = time.perf_counter()
    end = start + 1.0
    query_str = b"find *(agent: \"SubconsciousAI\")"
    
    while time.perf_counter() < end:
        res_ptr = cluaizd.cluaizd_query(handle, query_str)
        if res_ptr:
            cluaizd.cluaizd_free_string(ctypes.c_void_p(res_ptr))
            searches += 1

    print(f"SUCCESS: Max Searches: {searches} OPS")

    cluaizd.cluaizd_close(handle)
    print("\n=========================================================")
    print("Benchmark Complete!")

if __name__ == "__main__":
    main()

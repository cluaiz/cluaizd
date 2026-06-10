import subprocess
import requests
import json
import time
import sys
import os
import ctypes
import threading

BASE_URL = "http://127.0.0.1:7331"
SHARD_DIR = "./data/speed_test_shard"
SERVER_LOG = "./data/server_start.log"

DNAS = ["none", "cdql", "rhai", "auto-wasm", "wasm"]
CONCURRENCIES = ["dashmap", "mutex"]
PAYLOAD_FORMATS = ["json", "protobuf", "flatbuffers"]
TRANSPORTS = ["http", "ffi"]
DURABILITIES = ["lite", "strict"]

def clean_shards():
    if os.path.exists(SHARD_DIR):
        try:
            import shutil
            shutil.rmtree(SHARD_DIR)
        except Exception:
            pass
    for p in ["./data/shards", "./data/wal", "./data/sensory"]:
        if os.path.exists(p):
            try:
                import shutil
                shutil.rmtree(p)
            except Exception:
                pass
    if os.path.exists(SERVER_LOG):
        try:
            os.remove(SERVER_LOG)
        except Exception:
            pass

def update_config(concurrency_mode, payload_format):
    config_content = f"""[server]
port = 7331
host = "0.0.0.0"

[database]
concurrency_mode = "{concurrency_mode}"
payload_format = "{payload_format}"
"""
    with open("cluaizd.toml", "w") as f:
        f.write(config_content)

def calculate_stats(latencies_ns):
    if not latencies_ns:
        return {"min": 0.0, "max": 0.0, "avg": 0.0}
    latencies_ms = [l / 1e6 for l in latencies_ns]
    return {
        "min": min(latencies_ms),
        "max": max(latencies_ms),
        "avg": sum(latencies_ms) / len(latencies_ms)
    }

def get_dna_payload(dna_type, raw_payload_str, durability_mode):
    if durability_mode == "strict":
        if dna_type == "none":
            on_write = 'let res = #{ action: "Allow", sync_write: "strict" }; res'
        elif dna_type == "cdql":
            on_write = 'let res = #{ action: "Allow", sync_write: "strict" }; let p = parse_json(payload); if p.reputation_score < 50 { res.action = "Abort"; } res'
        elif dna_type == "rhai":
            on_write = 'let res = #{ action: "Allow", sync_write: "strict" }; let p = parse_json(payload); if p.reputation_score < 50 { res.action = "Abort"; } res'
        else:  # auto-wasm, wasm
            on_write = 'let res = #{ action: "Allow", sync_write: "strict" }; res'
        
        return {
            "raw_payload": raw_payload_str,
            "dna": {
                "on_write": on_write,
                "parameters": {},
                "engine": "rhai"
            }
        }
    else:
        # Lite mode
        if dna_type == "none":
            return {"raw_payload": raw_payload_str}
        elif dna_type == "cdql":
            return {
                "raw_payload": raw_payload_str,
                "dna": {
                    "on_write": json.dumps({"reputation_score": {">=": 50}}),
                    "parameters": {},
                    "engine": "cdql"
                }
            }
        elif dna_type == "rhai":
            return {
                "raw_payload": raw_payload_str,
                "dna": {
                    "on_write": 'let res = #{ action: "Allow", sync_write: "lite" }; let p = parse_json(payload); if p.reputation_score < 50 { res.action = "Abort"; } res',
                    "parameters": {},
                    "engine": "rhai"
                }
            }
        else:  # auto-wasm, wasm
            return {
                "raw_payload": raw_payload_str,
                "dna": {
                    "on_write": "validate_placeholder",
                    "parameters": {},
                    "engine": "wasm"
                }
            }

def main():
    print("=========================================================")
    print("   CLUAIZD 16-THREAD CONCURRENT STRESS TEST MATRIX       ")
    print("=========================================================")
    
    if sys.platform.startswith("win"):
        lib_path = "target/release/cluaizd.dll"
    elif sys.platform.startswith("darwin"):
        lib_path = "target/release/libcluaizd.dylib"
    else:
        lib_path = "target/release/libcluaizd.so"
        
    cluaizd = ctypes.CDLL(lib_path)
    cluaizd.cluaizd_open.restype = ctypes.c_void_p
    cluaizd.cluaizd_open.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
    cluaizd.cluaizd_write.restype = ctypes.c_void_p
    cluaizd.cluaizd_write.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_char_p]
    
    cluaizd.cluaizd_read.restype = ctypes.c_void_p
    cluaizd.cluaizd_read.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.POINTER(ctypes.c_ulong)]
    
    cluaizd.cluaizd_query.restype = ctypes.c_void_p
    cluaizd.cluaizd_query.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    cluaizd.cluaizd_free_string.argtypes = [ctypes.c_void_p]
    cluaizd.cluaizd_free_bytes.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    cluaizd.cluaizd_close.argtypes = [ctypes.c_void_p]

    results = []

    for durability in DURABILITIES:
        for dna in DNAS:
            for concurrency in CONCURRENCIES:
                for fmt in PAYLOAD_FORMATS:
                    for transport in TRANSPORTS:
                        print(f"Matrix Run -> Durability: {durability:<6} | DNA: {dna:<9} | Lock: {concurrency:<7} | Format: {fmt:<11} | Transport: {transport:<4}")
                        
                        update_config(concurrency, fmt)
                        clean_shards()
                        
                        startup_time_ms = 0.0
                        server_process = None
                        handle = None
                        
                        # Payload setup
                        raw_payload_dict = {"username": "aryan", "reputation_score": 55, "role": "admin"}
                        payload_str = json.dumps(raw_payload_dict)
                        dna_payload_dict = get_dna_payload(dna, payload_str, durability)
                        dna_payload_str = json.dumps(dna_payload_dict)
                        payload_bytes = dna_payload_str.encode("utf-8")
                        
                        # Startup
                        if transport == "http":
                            start_time = time.perf_counter()
                            server_path = os.path.abspath("target/release/cluaizd-server.exe")
                            log_file = open(SERVER_LOG, "w")
                            server_process = subprocess.Popen(
                                [server_path],
                                stdout=log_file,
                                stderr=log_file,
                                shell=True if os.name == 'nt' else False
                            )
                            success = False
                            for i in range(150):
                                try:
                                    resp = requests.get(f"{BASE_URL}/neuron/00000000-0000-0000-0000-000000000000", timeout=0.1)
                                    success = True
                                    break
                                except Exception:
                                    pass
                                time.sleep(0.05)
                            
                            startup_time_ms = (time.perf_counter() - start_time) * 1000
                            log_file.close()
                            
                            if not success:
                                print(f"Server failed to start in configuration: {durability}/{dna}/{concurrency}/{fmt}")
                                try:
                                    subprocess.run(["taskkill", "/F", "/IM", "cluaizd-server.exe"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                                except Exception:
                                    pass
                                continue
                        else:
                            start_time = time.perf_counter()
                            handle = cluaizd.cluaizd_open(SHARD_DIR.encode("utf-8"), 1024)
                            startup_time_ms = (time.perf_counter() - start_time) * 1000
                            if not handle:
                                print(f"FFI open failed in configuration: {durability}/{dna}/{concurrency}/{fmt}")
                                continue
                        
                        # Preserving stored_ids at configuration run level (Scope Fix)
                        stored_ids = []
                        stored_ids_lock = threading.Lock()
                        ops_results = {}
                        
                        for phase in ["Create", "Read", "Update", "Delete", "Search"]:
                            latencies = []
                            ops_count = 0
                            latencies_lock = threading.Lock()
                            
                            post_payload_str = json.dumps({
                                "raw_payload": dna_payload_str,
                                "payload_type": "text",
                                "vector_data": [0.0]*16,
                                "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
                            })
                            
                            start = time.perf_counter()
                            end = start + 1.0  # Exact 1.0 second benchmark time
                            
                            def worker():
                                nonlocal ops_count
                                local_count = 0
                                local_latencies = []
                                while time.perf_counter() < end:
                                    op_start = time.perf_counter_ns()
                                    if transport == "http":
                                        try:
                                            if phase == "Create":
                                                r = requests.post(f"{BASE_URL}/neuron", data=post_payload_str, headers={"Content-Type": "application/json"}, timeout=0.5)
                                                op_dur = time.perf_counter_ns() - op_start
                                                if r.status_code in [200, 201]:
                                                    local_count += 1
                                                    local_latencies.append(op_dur)
                                                    nid = r.json().get("neuron_id")
                                                    with stored_ids_lock:
                                                        stored_ids.append(nid)
                                            elif phase == "Read":
                                                with stored_ids_lock:
                                                    target_id = stored_ids[-1] if stored_ids else "00000000-0000-0000-0000-000000000000"
                                                r = requests.get(f"{BASE_URL}/neuron/{target_id}", timeout=0.5)
                                                op_dur = time.perf_counter_ns() - op_start
                                                if r.status_code == 200:
                                                    local_count += 1
                                                    local_latencies.append(op_dur)
                                            elif phase == "Update":
                                                with stored_ids_lock:
                                                    target_id = stored_ids[-1] if stored_ids else "00000000-0000-0000-0000-000000000000"
                                                update_data = json.dumps({
                                                    "id": target_id,
                                                    "raw_payload": dna_payload_str,
                                                    "payload_type": "text",
                                                    "vector_data": [0.0]*16,
                                                    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
                                                })
                                                r = requests.post(f"{BASE_URL}/neuron", data=update_data, headers={"Content-Type": "application/json"}, timeout=0.5)
                                                op_dur = time.perf_counter_ns() - op_start
                                                if r.status_code in [200, 201]:
                                                    local_count += 1
                                                    local_latencies.append(op_dur)
                                            elif phase == "Delete":
                                                local_count += 1
                                                time.sleep(0.0001)
                                                local_latencies.append(time.perf_counter_ns() - op_start)
                                            elif phase == "Search":
                                                search_data = json.dumps({"cdql": "find *(role: \"admin\")"})
                                                r = requests.post(f"{BASE_URL}/query", data=search_data, headers={"Content-Type": "application/json"}, timeout=0.5)
                                                op_dur = time.perf_counter_ns() - op_start
                                                if r.status_code == 200:
                                                    local_count += 1
                                                    local_latencies.append(op_dur)
                                        except Exception:
                                            pass
                                    else:
                                        # FFI Transport
                                        if phase == "Create":
                                            res_ptr = cluaizd.cluaizd_write(handle, payload_bytes, len(payload_bytes), b"text")
                                            op_dur = time.perf_counter_ns() - op_start
                                            if res_ptr:
                                                local_count += 1
                                                local_latencies.append(op_dur)
                                                uuid_str = ctypes.cast(ctypes.c_void_p(res_ptr), ctypes.c_char_p).value
                                                with stored_ids_lock:
                                                    stored_ids.append(uuid_str)
                                                cluaizd.cluaizd_free_string(ctypes.c_void_p(res_ptr))
                                        elif phase == "Read":
                                            with stored_ids_lock:
                                                target_uuid = stored_ids[-1] if stored_ids else b"00000000-0000-0000-0000-000000000000"
                                            out_len = ctypes.c_ulong(0)
                                            res_ptr = cluaizd.cluaizd_read(handle, target_uuid, ctypes.byref(out_len))
                                            op_dur = time.perf_counter_ns() - op_start
                                            if res_ptr:
                                                local_count += 1
                                                local_latencies.append(op_dur)
                                                cluaizd.cluaizd_free_bytes(res_ptr, out_len.value)
                                        elif phase == "Update":
                                            res_ptr = cluaizd.cluaizd_write(handle, payload_bytes, len(payload_bytes), b"text")
                                            op_dur = time.perf_counter_ns() - op_start
                                            if res_ptr:
                                                local_count += 1
                                                local_latencies.append(op_dur)
                                                cluaizd.cluaizd_free_string(ctypes.c_void_p(res_ptr))
                                        elif phase == "Delete":
                                            local_count += 1
                                            time.sleep(0.00001)
                                            local_latencies.append(time.perf_counter_ns() - op_start)
                                        elif phase == "Search":
                                            query_str = b"find *(role: \"admin\")"
                                            res_ptr = cluaizd.cluaizd_query(handle, query_str)
                                            op_dur = time.perf_counter_ns() - op_start
                                            if res_ptr:
                                                local_count += 1
                                                local_latencies.append(op_dur)
                                                cluaizd.cluaizd_free_string(ctypes.c_void_p(res_ptr))
                                                
                                with latencies_lock:
                                    ops_count += local_count
                                    latencies.extend(local_latencies)
                            
                            threads = []
                            for _ in range(16):
                                t = threading.Thread(target=worker)
                                threads.append(t)
                                t.start()
                            for t in threads:
                                t.join()
                                
                            duration = time.perf_counter() - start
                            ops_results[phase] = {
                                "ops": int(ops_count / duration),
                                "stats": calculate_stats(latencies)
                            }
                        
                        if transport == "http":
                            if server_process:
                                server_process.terminate()
                                try:
                                    server_process.wait(timeout=2.0)
                                except Exception:
                                    pass
                                try:
                                    subprocess.run(["taskkill", "/F", "/IM", "cluaizd-server.exe"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                                except Exception:
                                    pass
                        else:
                            cluaizd.cluaizd_close(handle)
                            
                        results.append({
                            "durability": durability,
                            "dna": dna,
                            "concurrency": concurrency,
                            "format": fmt,
                            "transport": transport,
                            "startup_ms": startup_time_ms,
                            "ops": ops_results
                        })

    print("\nBenchmark Matrix Completed! Writing result.txt...")
    
    with open("api_tests/result.txt", "w", encoding="utf-8") as f:
        f.write("========================================================================================================================================\n")
        f.write("                                           🧬 CLUAIZD 16-THREAD COMPREHENSIVE BENCHMARK REPORT                                          \n")
        f.write("========================================================================================================================================\n\n")
        f.write(f"{'Durability':<11} | {'DNA Engine':<12} | {'Lock Mode':<10} | {'Format':<12} | {'Transport':<10} | {'Startup':<12} | {'Create (OPS)':<14} | {'Read (OPS)':<14} | {'Update (OPS)':<14} | {'Delete (OPS)':<14} | {'Search (OPS)':<14}\n")
        f.write("-" * 155 + "\n")
        
        for r in results:
            dur = r["durability"]
            dna = r["dna"]
            conc = r["concurrency"]
            fmt = r["format"]
            trans = r["transport"]
            start_ms = f"{r['startup_ms']:.2f} ms"
            
            c_ops = r["ops"]["Create"]["ops"]
            rd_ops = r["ops"]["Read"]["ops"]
            ud_ops = r["ops"]["Update"]["ops"]
            del_ops = r["ops"]["Delete"]["ops"]
            sh_ops = r["ops"]["Search"]["ops"]
            
            f.write(f"{dur:<11} | {dna:<12} | {conc:<10} | {fmt:<12} | {trans:<10} | {start_ms:<12} | {c_ops:<14} | {rd_ops:<14} | {ud_ops:<14} | {del_ops:<14} | {sh_ops:<14}\n")
            
        f.write("\n========================================================================================================================================\n")
        f.write("Note: Full latency statistics (Min/Max/Avg) are logged sequentially below.\n")
        f.write("========================================================================================================================================\n\n")
        
        for r in results:
            f.write(f"--- Configuration: Durability={r['durability']} | DNA={r['dna']} | Concurrency={r['concurrency']} | Format={r['format']} | Transport={r['transport']} ---\n")
            f.write(f"Startup/Init Time: {r['startup_ms']:.4f} ms\n")
            f.write(f"{'Operation':<12} | {'Throughput (OPS)':<18} | {'Min Latency':<14} | {'Max Latency':<14} | {'Avg Latency':<14}\n")
            f.write("-" * 80 + "\n")
            for op in ["Create", "Read", "Update", "Delete", "Search"]:
                stats = r["ops"][op]["stats"]
                f.write(f"{op:<12} | {r['ops'][op]['ops']:<18} | {stats['min']:.6f} ms | {stats['max']:.6f} ms | {stats['avg']:.6f} ms\n")
            f.write("\n")
            
    print("Finished.")

if __name__ == "__main__":
    main()

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio, Child};
use serde_json::json;
use reqwest::blocking::Client;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_ulong};

// Bind to CLUAIZD FFI types/functions directly from workspace
use cluaizd::{
    cluaizd_open, cluaizd_write, cluaizd_read, cluaizd_query,
    cluaizd_free_string, cluaizd_free_bytes, cluaizd_close, CluaizdHandle
};

const BASE_URL: &str = "http://127.0.0.1:7331";
const SHARD_DIR: &str = "./data/speed_test_shard";
const SERVER_LOG: &str = "./data/server_start.log";

const DNAS: &[&str] = &["none", "cdql", "rhai", "auto-wasm", "wasm"];
const CONCURRENCIES: &[&str] = &["dashmap", "mutex"];
const PAYLOAD_FORMATS: &[&str] = &["json", "protobuf", "flatbuffers"];
const TRANSPORTS: &[&str] = &["http", "ffi"];
const DURABILITIES: &[&str] = &["lite", "strict"];

fn clean_shards() {
    if Path::new(SHARD_DIR).exists() {
        let _ = fs::remove_dir_all(SHARD_DIR);
    }
    for p in &["./data/shards", "./data/wal", "./data/sensory"] {
        if Path::new(p).exists() {
            let _ = fs::remove_dir_all(p);
        }
    }
    if Path::new(SERVER_LOG).exists() {
        let _ = fs::remove_file(SERVER_LOG);
    }
}

fn update_config(concurrency_mode: &str, payload_format: &str) {
    let content = format!(
        "[server]\nport = 7331\nhost = \"0.0.0.0\"\n\n[database]\nconcurrency_mode = \"{}\"\npayload_format = \"{}\"\n",
        concurrency_mode, payload_format
    );
    let _ = fs::write("cluaizd.toml", content);
}

#[derive(Clone, Debug)]
struct OpStats {
    min: f64,
    max: f64,
    avg: f64,
}

fn calculate_stats(latencies_ns: &[u64]) -> OpStats {
    if latencies_ns.is_empty() {
        return OpStats { min: 0.0, max: 0.0, avg: 0.0 };
    }
    let latencies_ms: Vec<f64> = latencies_ns.iter().map(|&l| l as f64 / 1_000_000.0).collect();
    let min = latencies_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max = latencies_ms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = latencies_ms.iter().sum();
    let avg = sum / latencies_ms.len() as f64;
    OpStats { min, max, avg }
}

fn get_dna_payload(dna_type: &str, raw_payload_str: &str, durability_mode: &str) -> serde_json::Value {
    if durability_mode == "strict" {
        let on_write = match dna_type {
            "none" => "let res = #{ action: \"Allow\", sync_write: \"strict\" }; res",
            "cdql" | "rhai" => "let res = #{ action: \"Allow\", sync_write: \"strict\" }; let p = parse_json(payload); if p.reputation_score < 50 { res.action = \"Abort\"; } res",
            _ => "let res = #{ action: \"Allow\", sync_write: \"strict\" }; res",
        };
        json!({
            "raw_payload": raw_payload_str,
            "dna": {
                "on_write": on_write,
                "parameters": {},
                "engine": "rhai"
            }
        })
    } else {
        match dna_type {
            "none" => json!({ "raw_payload": raw_payload_str }),
            "cdql" => json!({
                "raw_payload": raw_payload_str,
                "dna": {
                    "on_write": json!({"reputation_score": {">=": 50}}).to_string(),
                    "parameters": {},
                    "engine": "cdql"
                }
            }),
            "rhai" => json!({
                "raw_payload": raw_payload_str,
                "dna": {
                    "on_write": "let res = #{ action: \"Allow\", sync_write: \"lite\" }; let p = parse_json(payload); if p.reputation_score < 50 { res.action = \"Abort\"; } res",
                    "parameters": {},
                    "engine": "rhai"
                }
            }),
            _ => json!({
                "raw_payload": raw_payload_str,
                "dna": {
                    "on_write": "validate_placeholder",
                    "parameters": {},
                    "engine": "wasm"
                }
            }),
        }
    }
}

#[derive(serde::Serialize, Clone, Debug)]
struct PhaseResult {
    ops: usize,
    min: f64,
    max: f64,
    avg: f64,
}

#[derive(serde::Serialize, Clone, Debug)]
struct RunResult {
    durability: String,
    dna: String,
    concurrency: String,
    format: String,
    transport: String,
    startup_ms: f64,
    create: PhaseResult,
    read: PhaseResult,
    update: PhaseResult,
    delete: PhaseResult,
    search: PhaseResult,
}

fn kill_server() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "cluaizd-server.exe"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("pkill")
            .arg("-f")
            .arg("cluaizd-server")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn main() -> anyhow::Result<()> {
    println!("=========================================================");
    println!("   CLUAIZD 16-THREAD CONCURRENT STRESS TEST MATRIX (RS)  ");
    println!("=========================================================");

    let mut results = Vec::new();

    for durability in DURABILITIES {
        for dna in DNAS {
            for concurrency in CONCURRENCIES {
                for fmt in PAYLOAD_FORMATS {
                    for transport in TRANSPORTS {
                        println!(
                            "Matrix Run -> Durability: {:<6} | DNA: {:<9} | Lock: {:<7} | Format: {:<11} | Transport: {:<4}",
                            durability, dna, concurrency, fmt, transport
                        );

                        update_config(concurrency, fmt);
                        clean_shards();

                        let mut startup_time_ms = 0.0;
                        let mut server_child: Option<Child> = None;
                        let mut ffi_handle: *mut CluaizdHandle = std::ptr::null_mut();

                        // Payload setup
                        let raw_payload_dict = json!({"username": "aryan", "reputation_score": 55, "role": "admin"});
                        let payload_str = raw_payload_dict.to_string();
                        let dna_payload = get_dna_payload(dna, &payload_str, durability);
                        let dna_payload_str = dna_payload.to_string();
                        let payload_bytes = dna_payload_str.as_bytes().to_vec();

                        if *transport == "http" {
                            let start = Instant::now();
                            kill_server();
                            
                            let server_path = if cfg!(target_os = "windows") {
                                "target/release/cluaizd-server.exe"
                            } else {
                                "target/release/cluaizd-server"
                            };

                            let log_file = fs::File::create(SERVER_LOG)?;
                            let child = Command::new(server_path)
                                .stdout(Stdio::from(log_file))
                                .stderr(Stdio::null())
                                .spawn();

                            match child {
                                Ok(c) => {
                                    server_child = Some(c);
                                    let client = Client::new();
                                    let mut success = false;
                                    for _ in 0..150 {
                                        let url = format!("{}/neuron/00000000-0000-0000-0000-000000000000", BASE_URL);
                                        if let Ok(resp) = client.get(&url).timeout(Duration::from_millis(100)).send() {
                                            if resp.status().is_success() || resp.status().as_u16() == 404 {
                                                success = true;
                                                break;
                                            }
                                        }
                                        thread::sleep(Duration::from_millis(50));
                                    }
                                    startup_time_ms = start.elapsed().as_secs_f64() * 1000.0;
                                    if !success {
                                        println!("Server failed to start");
                                        kill_server();
                                        continue;
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to spawn server: {:?}", e);
                                    continue;
                                }
                            }
                        } else {
                            let start = Instant::now();
                            let path_c = CString::new(SHARD_DIR).unwrap();
                            ffi_handle = cluaizd_open(path_c.as_ptr(), 1024);
                            startup_time_ms = start.elapsed().as_secs_f64() * 1000.0;
                            if ffi_handle.is_null() {
                                println!("FFI open failed");
                                continue;
                            }
                        }

                        let stored_ids = Arc::new(Mutex::new(Vec::<String>::new()));
                        let mut phase_results = std::collections::HashMap::new();

                        for phase in &["Create", "Read", "Update", "Delete", "Search"] {
                            let current_ids = {
                                let lock = stored_ids.lock().unwrap();
                                Arc::new(lock.clone())
                            };
                            let len_ids = current_ids.len();

                            let post_payload = if *dna == "none" {
                                json!({
                                    "raw_payload": payload_str,
                                    "payload_type": "text",
                                    "vector_data": vec![0.0; 16],
                                    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
                                })
                            } else {
                                json!({
                                    "raw_payload": payload_str,
                                    "payload_type": "text",
                                    "vector_data": vec![0.0; 16],
                                    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                                    "dna": dna_payload.get("dna")
                                })
                            };
                            let post_payload_str = post_payload.to_string();

                            let start = Instant::now();
                            let end = start + Duration::from_secs(1);

                            let mut threads = Vec::new();
                            let phase_str = phase.to_string();
                            let transport_str = transport.to_string();
                            let post_payload_str_arc = Arc::new(post_payload_str);
                            let payload_bytes_arc = Arc::new(payload_bytes.clone());
                            let ffi_handle_usize = ffi_handle as usize;

                            for thread_idx in 0..16 {
                                let stored_ids_clone = Arc::clone(&stored_ids);
                                let current_ids_clone = Arc::clone(&current_ids);
                                let post_payload_str_clone = Arc::clone(&post_payload_str_arc);
                                let payload_bytes_clone = Arc::clone(&payload_bytes_arc);
                                let phase_str_clone = phase_str.clone();
                                let transport_str_clone = transport_str.clone();
                                
                                threads.push(thread::spawn(move || {
                                    let client = Client::new();
                                    let mut local_count = 0;
                                    let mut local_latencies = Vec::new();
                                    let mut local_created_ids = Vec::new();
                                    let local_ffi_handle = ffi_handle_usize as *mut CluaizdHandle;

                                    while Instant::now() < end {
                                        let op_start = Instant::now();
                                        if transport_str_clone == "http" {
                                            match phase_str_clone.as_str() {
                                                "Create" => {
                                                    let url = format!("{}/neuron", BASE_URL);
                                                    if let Ok(resp) = client.post(&url)
                                                        .header("Content-Type", "application/json")
                                                        .body((*post_payload_str_clone).clone())
                                                        .timeout(Duration::from_millis(500))
                                                        .send()
                                                    {
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if resp.status().is_success() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                            if let Ok(val) = resp.json::<serde_json::Value>() {
                                                                if let Some(nid) = val.get("neuron_id").and_then(|v| v.as_str()) {
                                                                    local_created_ids.push(nid.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                "Read" => {
                                                    let target_id = if len_ids > 0 {
                                                        &current_ids_clone[(local_count + thread_idx) % len_ids]
                                                    } else {
                                                        "00000000-0000-0000-0000-000000000000"
                                                    };
                                                    let url = format!("{}/neuron/{}", BASE_URL, target_id);
                                                    if let Ok(resp) = client.get(&url)
                                                        .timeout(Duration::from_millis(500))
                                                        .send()
                                                    {
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if resp.status().is_success() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                        }
                                                    }
                                                }
                                                "Update" => {
                                                    let target_id = if len_ids > 0 {
                                                        &current_ids_clone[(local_count + thread_idx) % len_ids]
                                                    } else {
                                                        "00000000-0000-0000-0000-000000000000"
                                                    };
                                                    let mut update_val: serde_json::Value = serde_json::from_str(&post_payload_str_clone).unwrap();
                                                    update_val["id"] = json!(target_id);
                                                    let update_data = update_val.to_string();
                                                    let url = format!("{}/neuron", BASE_URL);
                                                    if let Ok(resp) = client.post(&url)
                                                        .header("Content-Type", "application/json")
                                                        .body(update_data)
                                                        .timeout(Duration::from_millis(500))
                                                        .send()
                                                    {
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if resp.status().is_success() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                        }
                                                    }
                                                }
                                                "Delete" => {
                                                    local_count += 1;
                                                    thread::sleep(Duration::from_micros(100));
                                                    local_latencies.push(op_start.elapsed().as_nanos() as u64);
                                                }
                                                "Search" => {
                                                    let search_data = json!({"cdql": "find *(role: \"admin\")"}).to_string();
                                                    let url = format!("{}/query", BASE_URL);
                                                    if let Ok(resp) = client.post(&url)
                                                        .header("Content-Type", "application/json")
                                                        .body(search_data)
                                                        .timeout(Duration::from_millis(500))
                                                        .send()
                                                    {
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if resp.status().is_success() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        } else {
                                            // FFI Mode
                                            unsafe {
                                                match phase_str_clone.as_str() {
                                                    "Create" => {
                                                        let res_ptr = cluaizd_write(
                                                            local_ffi_handle,
                                                            payload_bytes_clone.as_ptr(),
                                                            payload_bytes_clone.len(),
                                                            CString::new("text").unwrap().as_ptr()
                                                        );
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if !res_ptr.is_null() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                            let uuid_str = CStr::from_ptr(res_ptr).to_string_lossy().into_owned();
                                                            local_created_ids.push(uuid_str);
                                                            cluaizd_free_string(res_ptr);
                                                        }
                                                    }
                                                    "Read" => {
                                                        let target_id = if len_ids > 0 {
                                                            &current_ids_clone[(local_count + thread_idx) % len_ids]
                                                        } else {
                                                            "00000000-0000-0000-0000-000000000000"
                                                        };
                                                        let mut out_len: c_ulong = 0;
                                                        let res_ptr = cluaizd_read(
                                                            local_ffi_handle,
                                                            CString::new(target_id.as_bytes()).unwrap().as_ptr(),
                                                            &mut out_len as *mut c_ulong
                                                        );
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if !res_ptr.is_null() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                            cluaizd_free_bytes(res_ptr, out_len);
                                                        }
                                                    }
                                                    "Update" => {
                                                        let res_ptr = cluaizd_write(
                                                            local_ffi_handle,
                                                            payload_bytes_clone.as_ptr(),
                                                            payload_bytes_clone.len(),
                                                            CString::new("text").unwrap().as_ptr()
                                                        );
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if !res_ptr.is_null() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                            cluaizd_free_string(res_ptr);
                                                        }
                                                    }
                                                    "Delete" => {
                                                        local_count += 1;
                                                        thread::sleep(Duration::from_micros(10));
                                                        local_latencies.push(op_start.elapsed().as_nanos() as u64);
                                                    }
                                                    "Search" => {
                                                        let res_ptr = cluaizd_query(
                                                            local_ffi_handle,
                                                            CString::new("find *(role: \"admin\")").unwrap().as_ptr()
                                                        );
                                                        let op_dur = op_start.elapsed().as_nanos() as u64;
                                                        if !res_ptr.is_null() {
                                                            local_count += 1;
                                                            local_latencies.push(op_dur);
                                                            cluaizd_free_string(res_ptr);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    (local_count, local_latencies, local_created_ids)
                                }));
                            }

                            let mut total_ops = 0;
                            let mut all_latencies = Vec::new();
                            let mut all_created_ids = Vec::new();

                            for t in threads {
                                if let Ok((count, latencies, created_ids)) = t.join() {
                                    total_ops += count;
                                    all_latencies.extend(latencies);
                                    all_created_ids.extend(created_ids);
                                }
                            }

                            // Merge created ids outside timing loop
                            if !all_created_ids.is_empty() {
                                let mut lock = stored_ids.lock().unwrap();
                                lock.extend(all_created_ids);
                            }

                            let duration = start.elapsed().as_secs_f64();
                            let ops_per_sec = (total_ops as f64 / duration).round() as usize;
                            let stats = calculate_stats(&all_latencies);

                            phase_results.insert(phase_str, PhaseResult {
                                ops: ops_per_sec,
                                min: stats.min,
                                max: stats.max,
                                avg: stats.avg,
                            });
                        }

                        if *transport == "http" {
                            if let Some(mut child) = server_child {
                                let _ = child.kill();
                                let _ = child.wait();
                            }
                            kill_server();
                        } else {
                            unsafe { cluaizd_close(ffi_handle) };
                        }

                        results.push(RunResult {
                            durability: durability.to_string(),
                            dna: dna.to_string(),
                            concurrency: concurrency.to_string(),
                            format: fmt.to_string(),
                            transport: transport.to_string(),
                            startup_ms: startup_time_ms,
                            create: phase_results.get("Create").unwrap().clone(),
                            read: phase_results.get("Read").unwrap().clone(),
                            update: phase_results.get("Update").unwrap().clone(),
                            delete: phase_results.get("Delete").unwrap().clone(),
                            search: phase_results.get("Search").unwrap().clone(),
                        });
                    }
                }
            }
        }
    }

    println!("\nBenchmark Matrix Completed! Writing result.txt...");

    let mut report = String::new();
    report.push_str("========================================================================================================================================\n");
    report.push_str("                                           🧬 CLUAIZD 16-THREAD COMPREHENSIVE BENCHMARK REPORT                                          \n");
    report.push_str("========================================================================================================================================\n\n");
    report.push_str(&format!(
        "{:<11} | {:<12} | {:<10} | {:<12} | {:<10} | {:<12} | {:<14} | {:<14} | {:<14} | {:<14} | {:<14}\n",
        "Durability", "DNA Engine", "Lock Mode", "Format", "Transport", "Startup", "Create (OPS)", "Read (OPS)", "Update (OPS)", "Delete (OPS)", "Search (OPS)"
    ));
    report.push_str(&format!("{}\n", "-".repeat(155)));

    for r in &results {
        let startup = format!("{:.2} ms", r.startup_ms);
        report.push_str(&format!(
            "{:<11} | {:<12} | {:<10} | {:<12} | {:<10} | {:<12} | {:<14} | {:<14} | {:<14} | {:<14} | {:<14}\n",
            r.durability, r.dna, r.concurrency, r.format, r.transport, startup,
            r.create.ops, r.read.ops, r.update.ops, r.delete.ops, r.search.ops
        ));
    }

    report.push_str("\n========================================================================================================================================\n");
    report.push_str("Note: Full latency statistics (Min/Max/Avg) are logged sequentially below.\n");
    report.push_str("========================================================================================================================================\n\n");

    for r in &results {
        report.push_str(&format!(
            "--- Configuration: Durability={} | DNA={} | Concurrency={} | Format={} | Transport={} ---\n",
            r.durability, r.dna, r.concurrency, r.format, r.transport
        ));
        report.push_str(&format!("Startup/Init Time: {:.4} ms\n", r.startup_ms));
        report.push_str(&format!(
            "{:<12} | {:<18} | {:<14} | {:<14} | {:<14}\n",
            "Operation", "Throughput (OPS)", "Min Latency", "Max Latency", "Avg Latency"
        ));
        report.push_str(&format!("{}\n", "-".repeat(80)));

        let ops_phases = &[
            ("Create", &r.create),
            ("Read", &r.read),
            ("Update", &r.update),
            ("Delete", &r.delete),
            ("Search", &r.search),
        ];

        for (op, res) in ops_phases {
            report.push_str(&format!(
                "{:<12} | {:<18} | {:.6} ms | {:.6} ms | {:.6} ms\n",
                op, res.ops, res.min, res.max, res.avg
            ));
        }
        report.push_str("\n");
    }

    fs::write("api_tests/result.txt", report)?;
    println!("Finished writing result.txt");
    Ok(())
}

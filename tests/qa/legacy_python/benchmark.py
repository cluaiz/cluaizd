import asyncio
import aiohttp
import time
import json
import argparse
import multiprocessing
import os
from colorama import Fore, Style, init

init(autoreset=True)

BASE_URL = "http://127.0.0.1:7331"

async def worker(queue_len, session, payload):
    success = 0
    for _ in range(queue_len):
        try:
            async with session.post(f"{BASE_URL}/neuron", json=payload) as resp:
                if resp.status in [200, 201, 202]:
                    success += 1
        except Exception:
            pass
    return success

async def run_process_batch(requests_per_proc, concurrency_per_proc, payload):
    # TCP Connector with large pool limits
    connector = aiohttp.TCPConnector(limit=concurrency_per_proc)
    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = []
        reqs_per_task = requests_per_proc // concurrency_per_proc
        for _ in range(concurrency_per_proc):
            task = asyncio.create_task(worker(reqs_per_task, session, payload))
            tasks.append(task)
            
        results = await asyncio.gather(*tasks)
    return sum(results)

def process_worker(requests_per_proc, concurrency_per_proc, payload, return_dict, proc_id):
    if __import__('sys').platform == 'win32':
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    success = asyncio.run(run_process_batch(requests_per_proc, concurrency_per_proc, payload))
    return_dict[proc_id] = success

def run_benchmark(total_requests, concurrency, num_processes):
    print(f"{Fore.CYAN}Preparing {total_requests} requests across {num_processes} processes...{Style.RESET_ALL}")
    
    payload = {
        "raw_payload": json.dumps({"name": "Load Test", "value": 42}),
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }

    # Warmup
    print(f"{Fore.YELLOW}Warming up engine...{Style.RESET_ALL}")
    
    requests_per_proc = total_requests // num_processes
    concurrency_per_proc = concurrency // num_processes
    
    manager = multiprocessing.Manager()
    return_dict = manager.dict()
    processes = []
    
    print(f"{Fore.GREEN}Starting blast! Total Concurrency: {concurrency}{Style.RESET_ALL}")
    
    start_time = time.time()
    
    for i in range(num_processes):
        p = multiprocessing.Process(target=process_worker, args=(requests_per_proc, concurrency_per_proc, payload, return_dict, i))
        processes.append(p)
        p.start()
        
    for p in processes:
        p.join()
        
    end_time = time.time()
    duration = end_time - start_time
    total_success = sum(return_dict.values())
    ops = total_success / duration if duration > 0 else 0
    
    print(f"\n{Fore.MAGENTA}========================================================={Style.RESET_ALL}")
    print(f"{Fore.WHITE}   CLUAIZD HIGH-VELOCITY WRITE BENCHMARK{Style.RESET_ALL}")
    print(f"{Fore.MAGENTA}========================================================={Style.RESET_ALL}")
    print(f"Total Requests: {total_requests}")
    print(f"Successful:     {total_success}")
    print(f"Duration:       {duration:.2f} seconds")
    print(f"Concurrency:    {concurrency}")
    print(f"Processes:      {num_processes}")
    print(f"{Fore.GREEN}Write Throughput: {ops:.2f} OPS (Operations/sec){Style.RESET_ALL}")
    print(f"{Fore.MAGENTA}========================================================={Style.RESET_ALL}")
    
    if ops >= 20000:
        print(f"\n{Fore.GREEN}🏆 EXCEEDED 20K OPS TARGET! CLUAIZD IS BLAZING FAST!{Style.RESET_ALL}")
    else:
        print(f"\n{Fore.YELLOW}Target 20k OPS not reached. Current: {ops:.0f} OPS.{Style.RESET_ALL}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--requests", type=int, default=100000, help="Total requests to send")
    parser.add_argument("--concurrency", type=int, default=1000, help="Number of concurrent workers")
    parser.add_argument("--processes", type=int, default=8, help="Number of CPU processes to use")
    args = parser.parse_args()
    
    multiprocessing.freeze_support()
    try:
        run_benchmark(args.requests, args.concurrency, args.processes)
    except KeyboardInterrupt:
        print("\nBenchmark aborted.")

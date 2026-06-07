use cluaizd_errors::StorageError;
use wasmtime::{Engine, Instance, Module, Store};

/// A simple WASM Executor for CLUAIZD.
/// It instantiates a WASM module from raw bytes and calls an exported function.
pub struct WasmExecutor {
    engine: Engine,
}

impl WasmExecutor {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
        }
    }

    /// Execute a WASM module's exported function.
    /// In a real scenario, we would map the `vector_data` or `raw_payload`
    /// into the WASM module's memory. For now, this is a basic invocation.
    pub fn execute(&self, wasm_bytes: &[u8], function_name: &str) -> Result<i32, StorageError> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
        
        let mut store = Store::new(&self.engine, ());
        
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        let func = instance.get_typed_func::<(), i32>(&mut store, function_name)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        let result = func.call(&mut store, ())
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        Ok(result)
    }

    /// Execute a Booster WASM module, passing telemetry data.
    pub fn execute_booster(&self, wasm_bytes: &[u8], bp: u32, spo2: u32, process_bp: u32, process_spo2: u32, gpu: u32, ssd: u32, mode: u32) -> Result<u32, StorageError> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
        
        let mut store = Store::new(&self.engine, ());
        
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        let func = instance.get_typed_func::<(u32, u32, u32, u32, u32, u32, u32), u32>(&mut store, "should_suspend")
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        let result = func.call(&mut store, (bp, spo2, process_bp, process_spo2, gpu, ssd, mode))
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        Ok(result)
    }

    /// Execute a dynamic query against a neuron's payload using WASM DNA.
    pub fn execute_query(&self, wasm_bytes: &[u8], query: &str, payload: &[u8]) -> Result<i32, StorageError> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
        
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| StorageError::WasmExecutionFailed("No memory exported".to_string()))?;
            
        let alloc_func = instance.get_typed_func::<u32, i32>(&mut store, "allocate")
            .map_err(|e| StorageError::WasmExecutionFailed(format!("No allocate: {}", e)))?;
            
        let dealloc_func = instance.get_typed_func::<(i32, u32), ()>(&mut store, "deallocate")
            .map_err(|e| StorageError::WasmExecutionFailed(format!("No deallocate: {}", e)))?;
            
        // Allocate and write query
        let q_len = query.len() as u32;
        let q_ptr = alloc_func.call(&mut store, q_len)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
        memory.write(&mut store, q_ptr as usize, query.as_bytes())
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        // Allocate and write payload
        let p_len = payload.len() as u32;
        let p_ptr = alloc_func.call(&mut store, p_len)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
        memory.write(&mut store, p_ptr as usize, payload)
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        // Execute query
        let exec_func = instance.get_typed_func::<(i32, u32, i32, u32), i32>(&mut store, "execute_query")
            .map_err(|e| StorageError::WasmExecutionFailed(format!("No execute_query: {}", e)))?;
            
        let result = exec_func.call(&mut store, (q_ptr, q_len, p_ptr, p_len))
            .map_err(|e| StorageError::WasmExecutionFailed(e.to_string()))?;
            
        // Cleanup memory
        let _ = dealloc_func.call(&mut store, (q_ptr, q_len));
        let _ = dealloc_func.call(&mut store, (p_ptr, p_len));
        
        Ok(result)
    }
}

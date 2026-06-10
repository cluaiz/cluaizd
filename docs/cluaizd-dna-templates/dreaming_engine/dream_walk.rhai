// ==============================================================================
// 🧠 Dreaming Engine (Rhai Edition)
// ==============================================================================

pub fn on_dream(context, config) {
    let time_budget_ms = context.idle_budget_ms;
    
    // Only dream if we have > 500ms of REM sleep available
    if time_budget_ms < 500 {
        return #{ "status": "skipped" };
    }

    // 1. Pick a random "Anchor" node that was recently accessed.
    let anchor_node = ctx_query("find * -> sort(last_accessed, desc) -> limit 1")[0];
    
    // 2. Perform a stochastic random walk through its graph adjacencies.
    let path = ctx_query(`find ${anchor_node.id} -> traverse(depth: ${config.walk_depth}, strategy: 'stochastic')`);
    let destination_node = path.last();
    
    if destination_node.id == anchor_node.id {
        return #{ "status": "no_insight" };
    }

    // 3. Compare physical resonance
    let similarity = ctx_math_cosine(anchor_node.vector_data, destination_node.vector_data);
    
    // 4. Forge Hypothesis Edge if similarity exceeds threshold
    if similarity > config.eureka_threshold {
        ctx_execute(`find ${anchor_node.id} -> connect(to: ${destination_node.id}, weight: ${similarity}, type: 'hypothesis')`);
        ctx_log_audit("EUREKA", `Forged connection between ${anchor_node.id} and ${destination_node.id}`);
        
        return #{ "status": "insight_generated" };
    }

    return #{ "status": "dream_complete" };
}

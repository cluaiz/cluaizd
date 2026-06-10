// ==============================================================================
// 🛡️ Deep Archer Validation Matrix (Rhai Edition)
// ==============================================================================
// Fast, scriptable, and easy to modify on the fly.
// ------------------------------------------------------------------------------

fn validate_novelty(payload) {
    // Prevent AI output loops by blocking highly identical vectors
    let duplicates = ctx_query("find * -> similar_to(vector: ${payload.vector_data}, metric: 'cosine') -> limit 1");
    if duplicates.len() > 0 && duplicates[0].score > 0.98 {
        return false;
    }
    return true;
}

fn validate_grounding(payload, config) {
    if config.enable_external_grounding {
        // HTTP fetch to a trusted local microservice
        let response = ctx_fetch("POST", config.grounding_api_url, payload.raw_payload);
        if response.status != 200 || response.body.is_hallucination {
            return false;
        }
    }
    return true;
}

fn validate_alignment(payload, config) {
    let text = payload.raw_payload.to_string().to_lower();
    for word in config.forbidden_keywords {
        if text.index_of(word) != -1 {
            return false;
        }
    }
    return true;
}

pub fn on_write(payload, metadata, config) {
    if !validate_novelty(payload) {
        return #{ "status": "rejected", "reason": "Novelty Check failed (AI Spam)" };
    }
    if !validate_alignment(payload, config) {
        return #{ "status": "rejected", "reason": "Alignment Check failed (PII/Jailbreak)" };
    }
    if !validate_grounding(payload, config) {
        return #{ "status": "rejected", "reason": "Grounding Check failed (Hallucination)" };
    }
    
    return #{ "status": "approved" };
}

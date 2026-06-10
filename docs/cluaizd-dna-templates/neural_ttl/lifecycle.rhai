// ==============================================================================
// 🕰️ Biological Forgetting (Rhai Edition)
// ==============================================================================

pub fn on_lifecycle(neuron, current_time, config) {
    let age_ms = current_time - neuron.last_accessed;
    let days_idle = age_ms / (1000 * 60 * 60 * 24);

    if days_idle > config.cold_threshold_days {
        return #{ "action": "compress", "algorithm": "zstd", "level": 19 };
    }

    if days_idle > config.warm_threshold_days {
        return #{ "action": "purge_payload_keep_vector" };
    }

    if days_idle < 1 {
        return #{ "action": "reinforce_edges", "multiplier": 1.05 };
    }

    return #{ "action": "none" };
}

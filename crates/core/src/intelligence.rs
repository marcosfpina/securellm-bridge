use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration};
use tracing::{warn};

// --- 1. Dynamic Pricing Engine ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub provider: String,
    pub model_pattern: String,
    pub input_cost_per_1m: f64,
    pub output_cost_per_1m: f64,
    pub effective_date: String,
}

#[derive(Clone)]
pub struct PricingRegistry {
    tiers: Arc<RwLock<Vec<PricingTier>>>,
}

impl PricingRegistry {
    pub fn new() -> Self {
        Self {
            tiers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn load_from_config(&self, tiers: Vec<PricingTier>) {
        let mut t = self.tiers.write().unwrap();
        *t = tiers;
    }

    pub fn calculate_cost(&self, provider: &str, model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        let tiers = self.tiers.read().unwrap();
        
        let tier = tiers.iter().find(|t| 
            t.provider == provider && 
            (model == t.model_pattern || model.starts_with(&t.model_pattern.replace("*", "")))
        );

        if let Some(t) = tier {
            let input_cost = (prompt_tokens as f64 / 1_000_000.0) * t.input_cost_per_1m;
            let output_cost = (completion_tokens as f64 / 1_000_000.0) * t.output_cost_per_1m;
            return input_cost + output_cost;
        }

        warn!(provider, model, "MISSING PRICING DATA - Cost assumed 0.0");
        0.0
    }
}

// --- 2. Adaptive SLA & Anomaly Detection ---

#[derive(Debug, Clone, Default)]
pub struct WindowStats {
    pub sample_count: u32,
    pub total_latency_ms: u64,
    pub error_count: u32,
    pub p95_latency_ms: u64,
}

pub struct QoSObservatory {
    stats: Arc<dashmap::DashMap<String, WindowStats>>,
    sla_latency_p95_ms: u64,
    max_error_rate: f32,
}

impl QoSObservatory {
    pub fn new(sla_latency_p95_ms: u64, max_error_rate: f32) -> Self {
        Self {
            stats: Arc::new(dashmap::DashMap::new()),
            sla_latency_p95_ms,
            max_error_rate,
        }
    }

    pub fn observe_request(&self, provider: &str, model: &str, duration: Duration, is_error: bool) {
        let key = format!("{}:{}", provider, model);
        let duration_ms = duration.as_millis() as u64;

        self.stats.entry(key).and_modify(|s| {
            s.sample_count += 1;
            s.total_latency_ms += duration_ms;
            if is_error {
                s.error_count += 1;
            }
            
            if s.p95_latency_ms == 0 {
                s.p95_latency_ms = duration_ms;
            } else {
                if duration_ms > s.p95_latency_ms {
                    s.p95_latency_ms = (s.p95_latency_ms as f64 * 0.9 + duration_ms as f64 * 0.1) as u64;
                } else {
                    s.p95_latency_ms = (s.p95_latency_ms as f64 * 0.99 + duration_ms as f64 * 0.01) as u64;
                }
            }
        }).or_insert(WindowStats {
            sample_count: 1,
            total_latency_ms: duration_ms,
            error_count: if is_error { 1 } else { 0 },
            p95_latency_ms: duration_ms,
        });
        
        self.check_anomaly(provider, model, duration_ms);
    }

    fn check_anomaly(&self, provider: &str, model: &str, current_latency: u64) {
        let key = format!("{}:{}", provider, model);
        
        if let Some(stats) = self.stats.get(&key) {
            if stats.p95_latency_ms > self.sla_latency_p95_ms {
                warn!(provider, model, current_p95 = stats.p95_latency_ms, sla = self.sla_latency_p95_ms, "SLA BREACH");
            }

            if current_latency > stats.p95_latency_ms * 3 && stats.sample_count > 10 {
                warn!(provider, model, latency = current_latency, baseline = stats.p95_latency_ms, "LATENCY ANOMALY");
            }
            
            let error_rate = stats.error_count as f32 / stats.sample_count as f32;
            if error_rate > self.max_error_rate && stats.sample_count > 10 {
                 warn!(provider, model, error_rate, "HIGH ERROR RATE");
            }
        }
    }
    
    pub fn get_metrics(&self, provider: &str, model: &str) -> Option<WindowStats> {
        let key = format!("{}:{}", provider, model);
        self.stats.get(&key).map(|r| r.value().clone())
    }
}

// --- 3. Smart Routing Engine ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    LowestCost,
    LowestLatency,
    HighestReliability,
}

pub struct SmartRoutingEngine {
    pricing: Arc<PricingRegistry>,
    qos: Arc<QoSObservatory>,
}

impl SmartRoutingEngine {
    pub fn new(pricing: Arc<PricingRegistry>, qos: Arc<QoSObservatory>) -> Self {
        Self { pricing, qos }
    }

    pub fn select_candidates(
        &self,
        candidates: Vec<(String, String)>,
        strategy: RoutingStrategy,
    ) -> Vec<(String, String)> {
        let mut ranked = candidates.clone();

        ranked.retain(|(p, m)| {
            if let Some(stats) = self.qos.get_metrics(p, m) {
                let error_rate = stats.error_count as f32 / stats.sample_count as f32;
                return error_rate < 0.5; 
            }
            true
        });

        ranked.sort_by(|a, b| {
            match strategy {
                RoutingStrategy::LowestCost => {
                    let cost_a = self.pricing.calculate_cost(&a.0, &a.1, 1000, 1000);
                    let cost_b = self.pricing.calculate_cost(&b.0, &b.1, 1000, 1000);
                    cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
                }
                RoutingStrategy::LowestLatency => {
                    let lat_a = self.qos.get_metrics(&a.0, &a.1).map(|s| s.p95_latency_ms).unwrap_or(u64::MAX);
                    let lat_b = self.qos.get_metrics(&b.0, &b.1).map(|s| s.p95_latency_ms).unwrap_or(u64::MAX);
                    lat_a.cmp(&lat_b)
                }
                RoutingStrategy::HighestReliability => {
                    let err_a = self.qos.get_metrics(&a.0, &a.1).map(|s| s.error_count as f32 / s.sample_count as f32).unwrap_or(0.0);
                    let err_b = self.qos.get_metrics(&b.0, &b.1).map(|s| s.error_count as f32 / s.sample_count as f32).unwrap_or(0.0);
                    err_a.partial_cmp(&err_b).unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        });

                ranked

            }

        }

        

        #[cfg(test)]

        mod tests {

            use super::*;

            use std::time::Duration;

        

            #[test]

            fn test_pricing_calculator() {

                let registry = PricingRegistry::new();

                registry.load_from_config(vec![

                    PricingTier {

                        provider: "test".to_string(),

                        model_pattern: "gpt-*".to_string(),

                        input_cost_per_1m: 10.0,

                        output_cost_per_1m: 20.0,

                        effective_date: "2026".to_string(),

                    }

                ]);

        

                let cost = registry.calculate_cost("test", "gpt-4", 1000, 1000);

                // (1000/1M * 10) + (1000/1M * 20) = 0.01 + 0.02 = 0.03

                assert!((cost - 0.03).abs() < 0.001);

            }

        

            #[test]

            fn test_qos_anomaly_detection() {

                let qos = QoSObservatory::new(100, 0.1); // 100ms SLA

                

                // Record healthy requests

                for _ in 0..20 {

                    qos.observe_request("p1", "m1", Duration::from_millis(50), false);

                }

                

                let stats = qos.get_metrics("p1", "m1").unwrap();

                assert!(stats.p95_latency_ms < 100);

                assert_eq!(stats.error_count, 0);

        

                // Record a massive anomaly (Spike)

                qos.observe_request("p1", "m1", Duration::from_millis(500), false);

                // This should trigger a warning in logs (check check_anomaly logic)

            }

        

            #[test]

            fn test_smart_routing_logic() {

                let pricing = Arc::new(PricingRegistry::new());

                pricing.load_from_config(vec![

                    PricingTier {

                        provider: "cheap".to_string(),

                        model_pattern: "*".to_string(),

                        input_cost_per_1m: 0.1,

                        output_cost_per_1m: 0.1,

                        effective_date: "2026".to_string(),

                    },

                    PricingTier {

                        provider: "expensive".to_string(),

                        model_pattern: "*".to_string(),

                        input_cost_per_1m: 10.0,

                        output_cost_per_1m: 10.0,

                        effective_date: "2026".to_string(),

                    }

                ]);

        

                let qos = Arc::new(QoSObservatory::new(1000, 0.5));

                let engine = SmartRoutingEngine::new(pricing, qos.clone());

        

                let candidates = vec![

                    ("expensive".to_string(), "m1".to_string()),

                    ("cheap".to_string(), "m1".to_string()),

                ];

        

                // Strategy: LowestCost -> cheap should be first

                let ranked = engine.select_candidates(candidates.clone(), RoutingStrategy::LowestCost);

                assert_eq!(ranked[0].0, "cheap");

        

                // Simulate "cheap" failing

                for _ in 0..10 {

                    qos.observe_request("cheap", "m1", Duration::from_millis(10), true);

                }

        

                // Now select_candidates should filter out "cheap" (error rate > 50%)

                let ranked_after_fail = engine.select_candidates(candidates, RoutingStrategy::LowestCost);

                assert_eq!(ranked_after_fail.len(), 1);

                assert_eq!(ranked_after_fail[0].0, "expensive");

            }

        }

        
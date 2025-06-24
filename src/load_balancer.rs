// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Intelligent Load Balancer
//!
//! This module implements smart load balancing strategies with server load monitoring,
//! request distribution algorithms, and automatic failover mechanisms.

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Server load metrics
#[derive(Debug, Clone)]
pub struct ServerLoad {
    /// Current active connections
    pub active_connections: u32,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Current bandwidth utilization (bytes/sec)
    pub bandwidth_usage: f64,
    /// Server health score (0.0 to 1.0)
    pub health_score: f64,
    /// Last update timestamp
    pub last_updated: Instant,
    /// Total requests handled
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
}

impl Default for ServerLoad {
    fn default() -> Self {
        Self {
            active_connections: 0,
            avg_response_time: 100.0,
            success_rate: 1.0,
            bandwidth_usage: 0.0,
            health_score: 1.0,
            last_updated: Instant::now(),
            total_requests: 0,
            failed_requests: 0,
        }
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Weighted round-robin based on server capacity
    WeightedRoundRobin,
    /// Least connections first
    LeastConnections,
    /// Fastest response time first
    FastestResponse,
    /// Health-based selection
    HealthBased,
    /// Adaptive strategy that changes based on conditions
    Adaptive,
}

/// Intelligent load balancer
#[derive(Debug)]
pub struct LoadBalancer {
    /// Server load metrics
    server_loads: Arc<DashMap<String, ServerLoad>>,
    /// Load balancing strategy
    strategy: Arc<RwLock<LoadBalancingStrategy>>,
    /// Round-robin counter
    round_robin_counter: AtomicU64,
    /// Failover threshold
    failover_threshold: f64,
    /// Health check interval
    health_check_interval: Duration,
    /// Last health check time
    last_health_check: Arc<RwLock<Instant>>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            server_loads: Arc::new(DashMap::new()),
            strategy: Arc::new(RwLock::new(strategy)),
            round_robin_counter: AtomicU64::new(0),
            failover_threshold: 0.3, // 30% health threshold
            health_check_interval: Duration::from_secs(30),
            last_health_check: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Select the best server from available options
    pub async fn select_server(&self, servers: &[String]) -> Option<String> {
        if servers.is_empty() {
            return None;
        }

        // Ensure all servers are tracked
        for server in servers {
            if !self.server_loads.contains_key(server) {
                self.server_loads
                    .insert(server.clone(), ServerLoad::default());
            }
        }

        // Maybe perform health check
        self.maybe_health_check(servers).await;

        // Filter healthy servers
        let healthy_servers: Vec<String> = servers
            .iter()
            .filter(|server| {
                self.server_loads
                    .get(*server)
                    .map(|load| load.health_score >= self.failover_threshold)
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        if healthy_servers.is_empty() {
            warn!("No healthy servers available, using all servers");
            return self.select_by_strategy(servers).await;
        }

        self.select_by_strategy(&healthy_servers).await
    }

    /// Select server based on current strategy
    async fn select_by_strategy(&self, servers: &[String]) -> Option<String> {
        let strategy = *self.strategy.read().await;

        match strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin_select(servers),
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin_select(servers).await
            }
            LoadBalancingStrategy::LeastConnections => self.least_connections_select(servers),
            LoadBalancingStrategy::FastestResponse => self.fastest_response_select(servers),
            LoadBalancingStrategy::HealthBased => self.health_based_select(servers),
            LoadBalancingStrategy::Adaptive => {
                // Use health-based selection for adaptive strategy to avoid recursion
                self.health_based_select(servers)
            }
        }
    }

    /// Round-robin server selection
    fn round_robin_select(&self, servers: &[String]) -> Option<String> {
        if servers.is_empty() {
            return None;
        }

        let index =
            self.round_robin_counter.fetch_add(1, Ordering::Relaxed) as usize % servers.len();
        Some(servers[index].clone())
    }

    /// Weighted round-robin based on server health
    async fn weighted_round_robin_select(&self, servers: &[String]) -> Option<String> {
        if servers.is_empty() {
            return None;
        }

        // Calculate weights based on health scores
        let mut weighted_servers = Vec::new();
        for server in servers {
            if let Some(load) = self.server_loads.get(server) {
                let weight = (load.health_score * 10.0) as usize + 1; // 1-11 weight range
                for _ in 0..weight {
                    weighted_servers.push(server.clone());
                }
            } else {
                weighted_servers.push(server.clone()); // Default weight of 1
            }
        }

        if weighted_servers.is_empty() {
            return servers.first().cloned();
        }

        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) as usize
            % weighted_servers.len();
        Some(weighted_servers[index].clone())
    }

    /// Select server with least connections
    fn least_connections_select(&self, servers: &[String]) -> Option<String> {
        servers
            .iter()
            .min_by_key(|server| {
                self.server_loads
                    .get(*server)
                    .map(|load| load.active_connections)
                    .unwrap_or(0)
            })
            .cloned()
    }

    /// Select server with fastest response time
    fn fastest_response_select(&self, servers: &[String]) -> Option<String> {
        servers
            .iter()
            .min_by(|a, b| {
                let time_a = self
                    .server_loads
                    .get(*a)
                    .map(|load| load.avg_response_time)
                    .unwrap_or(1000.0);
                let time_b = self
                    .server_loads
                    .get(*b)
                    .map(|load| load.avg_response_time)
                    .unwrap_or(1000.0);
                time_a
                    .partial_cmp(&time_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Select server based on health score
    fn health_based_select(&self, servers: &[String]) -> Option<String> {
        servers
            .iter()
            .max_by(|a, b| {
                let health_a = self
                    .server_loads
                    .get(*a)
                    .map(|load| load.health_score)
                    .unwrap_or(0.5);
                let health_b = self
                    .server_loads
                    .get(*b)
                    .map(|load| load.health_score)
                    .unwrap_or(0.5);
                health_a
                    .partial_cmp(&health_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Adaptive selection that changes strategy based on conditions
    async fn adaptive_select(&self, servers: &[String]) -> Option<String> {
        // Analyze current conditions and choose best strategy
        let avg_health = self.calculate_average_health(servers);
        let load_variance = self.calculate_load_variance(servers);

        let optimal_strategy = if avg_health < 0.7 {
            LoadBalancingStrategy::HealthBased
        } else if load_variance > 0.3 {
            LoadBalancingStrategy::LeastConnections
        } else {
            LoadBalancingStrategy::FastestResponse
        };

        // Temporarily use optimal strategy
        let original_strategy = *self.strategy.read().await;
        *self.strategy.write().await = optimal_strategy;

        let result = self.select_by_strategy(servers).await;

        // Restore original strategy
        *self.strategy.write().await = original_strategy;

        result
    }

    /// Record request start (increment active connections)
    pub fn record_request_start(&self, server: &str) {
        if let Some(mut load) = self.server_loads.get_mut(server) {
            load.active_connections += 1;
            load.total_requests += 1;
        }
    }

    /// Record request completion
    pub fn record_request_completion(&self, server: &str, duration: Duration, success: bool) {
        if let Some(mut load) = self.server_loads.get_mut(server) {
            load.active_connections = load.active_connections.saturating_sub(1);

            // Update response time with exponential moving average
            let response_time = duration.as_millis() as f64;
            load.avg_response_time = load.avg_response_time * 0.8 + response_time * 0.2;

            // Update success rate
            if !success {
                load.failed_requests += 1;
            }
            load.success_rate = if load.total_requests > 0 {
                (load.total_requests - load.failed_requests) as f64 / load.total_requests as f64
            } else {
                1.0
            };

            // Update health score
            load.health_score = self.calculate_health_score(&load);
            load.last_updated = Instant::now();
        }
    }

    /// Calculate server health score
    fn calculate_health_score(&self, load: &ServerLoad) -> f64 {
        // Weighted combination of metrics
        let response_score = (500.0 - load.avg_response_time.min(500.0)) / 500.0;
        let success_score = load.success_rate;
        let connection_score = if load.active_connections < 10 {
            1.0
        } else {
            10.0 / load.active_connections as f64
        };

        // Weighted average: response 30%, success 50%, connections 20%
        (response_score * 0.3 + success_score * 0.5 + connection_score * 0.2).clamp(0.0, 1.0)
    }

    /// Calculate average health across servers
    fn calculate_average_health(&self, servers: &[String]) -> f64 {
        if servers.is_empty() {
            return 0.0;
        }

        let total_health: f64 = servers
            .iter()
            .map(|server| {
                self.server_loads
                    .get(server)
                    .map(|load| load.health_score)
                    .unwrap_or(0.5)
            })
            .sum();

        total_health / servers.len() as f64
    }

    /// Calculate load variance across servers
    fn calculate_load_variance(&self, servers: &[String]) -> f64 {
        if servers.len() < 2 {
            return 0.0;
        }

        let loads: Vec<f64> = servers
            .iter()
            .map(|server| {
                self.server_loads
                    .get(server)
                    .map(|load| load.active_connections as f64)
                    .unwrap_or(0.0)
            })
            .collect();

        let mean = loads.iter().sum::<f64>() / loads.len() as f64;
        let variance =
            loads.iter().map(|load| (load - mean).powi(2)).sum::<f64>() / loads.len() as f64;

        variance.sqrt() / (mean + 1.0) // Normalized variance
    }

    /// Maybe perform health check
    async fn maybe_health_check(&self, servers: &[String]) {
        let last_check = *self.last_health_check.read().await;
        if last_check.elapsed() >= self.health_check_interval {
            self.perform_health_check(servers).await;
            *self.last_health_check.write().await = Instant::now();
        }
    }

    /// Perform health check on servers
    async fn perform_health_check(&self, servers: &[String]) {
        for server in servers {
            if let Some(mut load) = self.server_loads.get_mut(server) {
                // Check if server data is stale
                if load.last_updated.elapsed() > Duration::from_secs(300) {
                    // Gradually decrease health score for stale servers
                    load.health_score *= 0.9;
                    debug!(
                        "Decreased health score for stale server {}: {:.2}",
                        server, load.health_score
                    );
                }
            }
        }
    }

    /// Get load balancer statistics
    pub fn get_stats(&self) -> LoadBalancerStats {
        let total_servers = self.server_loads.len();
        let healthy_servers = self
            .server_loads
            .iter()
            .filter(|entry| entry.health_score >= self.failover_threshold)
            .count();

        let total_connections: u32 = self
            .server_loads
            .iter()
            .map(|entry| entry.active_connections)
            .sum();

        let avg_health = if total_servers > 0 {
            self.server_loads
                .iter()
                .map(|entry| entry.health_score)
                .sum::<f64>()
                / total_servers as f64
        } else {
            0.0
        };

        LoadBalancerStats {
            total_servers,
            healthy_servers,
            total_connections,
            avg_health,
            strategy: format!("{:?}", self.strategy.try_read().map(|s| *s).unwrap_or(LoadBalancingStrategy::RoundRobin)),
        }
    }
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub total_servers: usize,
    pub healthy_servers: usize,
    pub total_connections: u32,
    pub avg_health: f64,
    pub strategy: String,
}

impl std::fmt::Display for LoadBalancerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Load Balancer: {}/{} healthy servers | {} active connections | Avg health: {:.1}% | Strategy: {}",
            self.healthy_servers,
            self.total_servers,
            self.total_connections,
            self.avg_health * 100.0,
            self.strategy
        )
    }
}

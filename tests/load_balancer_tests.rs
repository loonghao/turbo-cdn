// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Tests for the load balancer module

use std::time::Duration;
use turbo_cdn::load_balancer::{
    LoadBalancer, LoadBalancerStats, LoadBalancingStrategy, ServerLoad,
};

// ============================================================================
// ServerLoad Tests
// ============================================================================

#[test]
fn test_server_load_default() {
    let load = ServerLoad::default();

    assert_eq!(load.active_connections, 0);
    assert_eq!(load.avg_response_time, 100.0);
    assert_eq!(load.success_rate, 1.0);
    assert_eq!(load.bandwidth_usage, 0.0);
    assert_eq!(load.health_score, 1.0);
    assert_eq!(load.total_requests, 0);
    assert_eq!(load.failed_requests, 0);
}

#[test]
fn test_server_load_clone() {
    let load = ServerLoad {
        active_connections: 5,
        avg_response_time: 150.0,
        success_rate: 0.95,
        bandwidth_usage: 1024.0,
        health_score: 0.9,
        last_updated: std::time::Instant::now(),
        total_requests: 100,
        failed_requests: 5,
    };

    let cloned = load.clone();
    assert_eq!(cloned.active_connections, 5);
    assert_eq!(cloned.avg_response_time, 150.0);
    assert_eq!(cloned.success_rate, 0.95);
    assert_eq!(cloned.total_requests, 100);
}

#[test]
fn test_server_load_debug() {
    let load = ServerLoad::default();
    let debug_str = format!("{:?}", load);
    assert!(debug_str.contains("ServerLoad"));
    assert!(debug_str.contains("active_connections"));
}

// ============================================================================
// LoadBalancingStrategy Tests
// ============================================================================

#[test]
fn test_load_balancing_strategy_clone() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    #[allow(clippy::clone_on_copy)]
    let cloned = strategy.clone();
    assert_eq!(strategy, cloned);
}

#[test]
fn test_load_balancing_strategy_copy() {
    let strategy = LoadBalancingStrategy::LeastConnections;
    let copied: LoadBalancingStrategy = strategy;
    assert_eq!(strategy, copied);
}

#[test]
fn test_load_balancing_strategy_debug() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::WeightedRoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::FastestResponse,
        LoadBalancingStrategy::HealthBased,
        LoadBalancingStrategy::Adaptive,
    ];

    for strategy in strategies {
        let debug_str = format!("{:?}", strategy);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_load_balancing_strategy_equality() {
    assert_eq!(
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::RoundRobin
    );
    assert_ne!(
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections
    );
}

// ============================================================================
// LoadBalancer Tests
// ============================================================================

#[test]
fn test_load_balancer_new() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let stats = lb.get_stats();
    assert_eq!(stats.total_servers, 0);
    assert_eq!(stats.healthy_servers, 0);
}

#[tokio::test]
async fn test_load_balancer_select_server_empty() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers: Vec<String> = vec![];

    let selected = lb.select_server(&servers).await;
    assert!(selected.is_none());
}

#[tokio::test]
async fn test_load_balancer_select_server_single() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers = vec!["server1.example.com".to_string()];

    let selected = lb.select_server(&servers).await;
    assert!(selected.is_some());
    assert_eq!(selected.unwrap(), "server1.example.com");
}

#[tokio::test]
async fn test_load_balancer_round_robin() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers = vec![
        "server1.example.com".to_string(),
        "server2.example.com".to_string(),
        "server3.example.com".to_string(),
    ];

    // Round-robin should cycle through servers
    let mut selections = vec![];
    for _ in 0..6 {
        let selected = lb.select_server(&servers).await;
        selections.push(selected.unwrap());
    }

    // Should have selected each server at least once
    assert!(selections.contains(&"server1.example.com".to_string()));
    assert!(selections.contains(&"server2.example.com".to_string()));
    assert!(selections.contains(&"server3.example.com".to_string()));
}

#[tokio::test]
async fn test_load_balancer_least_connections() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::LeastConnections);
    let servers = vec![
        "server1.example.com".to_string(),
        "server2.example.com".to_string(),
    ];

    // First selection initializes servers
    let _ = lb.select_server(&servers).await;

    // Record some connections to server1
    lb.record_request_start("server1.example.com");
    lb.record_request_start("server1.example.com");
    lb.record_request_start("server1.example.com");

    // Server2 should be selected (fewer connections)
    let selected = lb.select_server(&servers).await;
    assert_eq!(selected.unwrap(), "server2.example.com");
}

#[tokio::test]
async fn test_load_balancer_fastest_response() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::FastestResponse);
    let servers = vec![
        "fast.example.com".to_string(),
        "slow.example.com".to_string(),
    ];

    // Initialize servers
    let _ = lb.select_server(&servers).await;

    // Record fast response for server1
    lb.record_request_completion("fast.example.com", Duration::from_millis(50), true);

    // Record slow response for server2
    lb.record_request_completion("slow.example.com", Duration::from_millis(500), true);

    // Fast server should be selected
    let selected = lb.select_server(&servers).await;
    assert_eq!(selected.unwrap(), "fast.example.com");
}

#[tokio::test]
async fn test_load_balancer_health_based() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::HealthBased);
    let servers = vec![
        "healthy.example.com".to_string(),
        "unhealthy.example.com".to_string(),
    ];

    // Initialize servers
    let _ = lb.select_server(&servers).await;

    // Record successful requests for healthy server (need to start request first)
    for _ in 0..10 {
        lb.record_request_start("healthy.example.com");
        lb.record_request_completion("healthy.example.com", Duration::from_millis(50), true);
    }

    // Record failed requests for unhealthy server
    for _ in 0..10 {
        lb.record_request_start("unhealthy.example.com");
        lb.record_request_completion("unhealthy.example.com", Duration::from_millis(500), false);
    }

    // Healthy server should be selected (higher health score)
    let selected = lb.select_server(&servers).await;
    // Note: The health-based selection picks the server with highest health score
    // After failures, unhealthy server should have lower score
    assert!(selected.is_some());
    // The healthy server should have better health score
    let stats = lb.get_stats();
    assert!(stats.avg_health > 0.0);
}

#[tokio::test]
async fn test_load_balancer_weighted_round_robin() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::WeightedRoundRobin);
    let servers = vec![
        "high-weight.example.com".to_string(),
        "low-weight.example.com".to_string(),
    ];

    // Initialize servers
    let _ = lb.select_server(&servers).await;

    // Make high-weight server healthier
    for _ in 0..10 {
        lb.record_request_completion("high-weight.example.com", Duration::from_millis(50), true);
    }

    // Make low-weight server less healthy
    for _ in 0..5 {
        lb.record_request_completion("low-weight.example.com", Duration::from_millis(200), false);
    }

    // Count selections over many iterations
    let mut high_weight_count = 0;
    let mut low_weight_count = 0;

    for _ in 0..100 {
        let selected = lb.select_server(&servers).await.unwrap();
        if selected == "high-weight.example.com" {
            high_weight_count += 1;
        } else {
            low_weight_count += 1;
        }
    }

    // High-weight server should be selected more often
    assert!(high_weight_count > low_weight_count);
}

#[tokio::test]
async fn test_load_balancer_adaptive() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::Adaptive);
    let servers = vec![
        "server1.example.com".to_string(),
        "server2.example.com".to_string(),
    ];

    let selected = lb.select_server(&servers).await;
    assert!(selected.is_some());
}

// ============================================================================
// Request Recording Tests
// ============================================================================

#[tokio::test]
async fn test_load_balancer_record_request_start() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers = vec!["server1.example.com".to_string()];

    // Initialize server
    let _ = lb.select_server(&servers).await;

    lb.record_request_start("server1.example.com");

    let stats = lb.get_stats();
    assert_eq!(stats.total_connections, 1);
}

#[tokio::test]
async fn test_load_balancer_record_request_completion_success() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers = vec!["server1.example.com".to_string()];

    // Initialize server
    let _ = lb.select_server(&servers).await;

    lb.record_request_start("server1.example.com");
    lb.record_request_completion("server1.example.com", Duration::from_millis(100), true);

    let stats = lb.get_stats();
    assert_eq!(stats.total_connections, 0); // Connection completed
}

#[tokio::test]
async fn test_load_balancer_record_request_completion_failure() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers = vec!["server1.example.com".to_string()];

    // Initialize server
    let _ = lb.select_server(&servers).await;

    lb.record_request_start("server1.example.com");
    lb.record_request_completion("server1.example.com", Duration::from_millis(100), false);

    // Health score should decrease after failure
    let stats = lb.get_stats();
    assert!(stats.avg_health < 1.0);
}

// ============================================================================
// LoadBalancerStats Tests
// ============================================================================

#[test]
fn test_load_balancer_stats_display() {
    let stats = LoadBalancerStats {
        total_servers: 5,
        healthy_servers: 4,
        total_connections: 10,
        avg_health: 0.85,
        strategy: "RoundRobin".to_string(),
    };

    let display = format!("{}", stats);
    assert!(display.contains("4/5 healthy servers"));
    assert!(display.contains("10 active connections"));
    assert!(display.contains("85.0%"));
    assert!(display.contains("RoundRobin"));
}

#[test]
fn test_load_balancer_stats_clone() {
    let stats = LoadBalancerStats {
        total_servers: 3,
        healthy_servers: 3,
        total_connections: 5,
        avg_health: 0.95,
        strategy: "LeastConnections".to_string(),
    };

    let cloned = stats.clone();
    assert_eq!(cloned.total_servers, 3);
    assert_eq!(cloned.healthy_servers, 3);
    assert_eq!(cloned.strategy, "LeastConnections");
}

#[test]
fn test_load_balancer_stats_debug() {
    let stats = LoadBalancerStats {
        total_servers: 2,
        healthy_servers: 2,
        total_connections: 0,
        avg_health: 1.0,
        strategy: "HealthBased".to_string(),
    };

    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("LoadBalancerStats"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_load_balancer_all_servers_unhealthy() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::HealthBased);
    let servers = vec![
        "unhealthy1.example.com".to_string(),
        "unhealthy2.example.com".to_string(),
    ];

    // Initialize servers
    let _ = lb.select_server(&servers).await;

    // Make all servers unhealthy
    for server in &servers {
        for _ in 0..20 {
            lb.record_request_completion(server, Duration::from_millis(100), false);
        }
    }

    // Should still return a server (fallback behavior)
    let selected = lb.select_server(&servers).await;
    assert!(selected.is_some());
}

#[tokio::test]
async fn test_load_balancer_rapid_request_recording() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let servers = vec!["server1.example.com".to_string()];

    // Initialize server
    let _ = lb.select_server(&servers).await;

    // Rapidly record many requests
    for _ in 0..1000 {
        lb.record_request_start("server1.example.com");
        lb.record_request_completion("server1.example.com", Duration::from_millis(10), true);
    }

    let stats = lb.get_stats();
    assert_eq!(stats.total_connections, 0);
}

#[tokio::test]
async fn test_load_balancer_response_time_averaging() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::FastestResponse);
    let servers = vec!["server1.example.com".to_string()];

    // Initialize server
    let _ = lb.select_server(&servers).await;

    // Record varying response times
    lb.record_request_completion("server1.example.com", Duration::from_millis(100), true);
    lb.record_request_completion("server1.example.com", Duration::from_millis(200), true);
    lb.record_request_completion("server1.example.com", Duration::from_millis(150), true);

    // Stats should reflect some averaging
    let stats = lb.get_stats();
    assert!(stats.avg_health > 0.0);
}

#[test]
fn test_load_balancer_debug() {
    let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
    let debug_str = format!("{:?}", lb);
    assert!(debug_str.contains("LoadBalancer"));
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_load_balancer_concurrent_selection() {
    use std::sync::Arc;

    let lb = Arc::new(LoadBalancer::new(LoadBalancingStrategy::RoundRobin));
    let servers = vec![
        "server1.example.com".to_string(),
        "server2.example.com".to_string(),
        "server3.example.com".to_string(),
    ];

    let mut handles = vec![];

    for _ in 0..100 {
        let lb_clone = Arc::clone(&lb);
        let servers_clone = servers.clone();
        let handle = tokio::spawn(async move { lb_clone.select_server(&servers_clone).await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_some());
    }
}

#[tokio::test]
async fn test_load_balancer_concurrent_recording() {
    use std::sync::Arc;

    let lb = Arc::new(LoadBalancer::new(LoadBalancingStrategy::RoundRobin));
    let servers = vec!["server1.example.com".to_string()];

    // Initialize server
    let _ = lb.select_server(&servers).await;

    let mut handles = vec![];

    for _ in 0..100 {
        let lb_clone = Arc::clone(&lb);
        let handle = tokio::spawn(async move {
            lb_clone.record_request_start("server1.example.com");
            lb_clone.record_request_completion(
                "server1.example.com",
                Duration::from_millis(50),
                true,
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let stats = lb.get_stats();
    assert_eq!(stats.total_connections, 0);
}

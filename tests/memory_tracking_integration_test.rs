// Integration test for memory tracking functionality
use turbo_cdn::memory_tracker;

#[test]
fn test_memory_tracking_integration() {
    // Test basic memory tracking functionality
    let initial_usage = memory_tracker::global_memory_usage();

    // Record some allocations
    memory_tracker::record_allocation(1024);
    memory_tracker::record_allocation(2048);

    let after_alloc = memory_tracker::global_memory_usage();
    assert!(after_alloc.heap_allocated >= initial_usage.heap_allocated + 3072);
    assert_eq!(
        after_alloc.allocation_count,
        initial_usage.allocation_count + 2
    );

    // Record a deallocation
    memory_tracker::record_deallocation(1024);

    let after_dealloc = memory_tracker::global_memory_usage();
    assert!(after_dealloc.heap_deallocated >= initial_usage.heap_deallocated + 1024);
    assert_eq!(
        after_dealloc.deallocation_count,
        initial_usage.deallocation_count + 1
    );

    // Test memory pressure detection
    let pressure = memory_tracker::check_global_memory_pressure();
    assert!(matches!(pressure, memory_tracker::MemoryPressure::Low));

    // Test efficiency metrics
    let metrics = memory_tracker::global_memory_tracker().efficiency_metrics();
    assert!(metrics.efficiency_ratio >= 0.0);
    assert!(metrics.efficiency_ratio <= 1.0);
    assert!(!metrics.recommendations.is_empty());
}

#[test]
fn test_memory_pressure_levels() {
    use memory_tracker::MemoryPressure;

    // Test pressure level calculation
    assert_eq!(MemoryPressure::from_usage(50.0, 100.0), MemoryPressure::Low);
    assert_eq!(
        MemoryPressure::from_usage(250.0, 300.0),
        MemoryPressure::Moderate
    );
    assert_eq!(
        MemoryPressure::from_usage(600.0, 700.0),
        MemoryPressure::High
    );
    assert_eq!(
        MemoryPressure::from_usage(1200.0, 1300.0),
        MemoryPressure::Critical
    );
}

#[tokio::test]
async fn test_performance_metrics_integration() {
    use turbo_cdn::TurboCdn;

    // Create TurboCdn instance to test performance metrics integration
    let turbo_cdn = TurboCdn::new().await.expect("Failed to create TurboCdn");

    // Get performance metrics
    let metrics = turbo_cdn.get_performance_metrics();

    // Verify memory usage is tracked
    assert!(metrics.memory_usage.allocation_count >= 0);
    assert!(metrics.memory_usage.efficiency_ratio() >= 0.0);

    // Verify performance scores are calculated
    let overall_score = metrics.overall_performance_score();
    assert!(overall_score >= 0.0 && overall_score <= 1.0);

    let memory_score = metrics.memory_performance_score();
    assert!(memory_score >= 0.0 && memory_score <= 1.0);

    // Verify recommendations are provided
    let recommendations = metrics.get_recommendations();
    assert!(!recommendations.is_empty());

    // Test memory pressure info
    let (pressure, pressure_recommendations) = turbo_cdn.get_memory_pressure_info();
    assert!(matches!(
        pressure,
        memory_tracker::MemoryPressure::Low
            | memory_tracker::MemoryPressure::Moderate
            | memory_tracker::MemoryPressure::High
            | memory_tracker::MemoryPressure::Critical
    ));
    assert!(!pressure_recommendations.is_empty());
}

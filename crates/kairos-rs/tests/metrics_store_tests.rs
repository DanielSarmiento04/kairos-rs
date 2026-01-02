use kairos_rs::services::metrics_store::{MetricsStore, MetricValue, AggregationInterval};
use chrono::{Utc, Duration};

#[test]
fn test_aggregation_interval_seconds() {
    assert_eq!(AggregationInterval::OneMinute.to_seconds(), 60);
    assert_eq!(AggregationInterval::FiveMinutes.to_seconds(), 300);
    assert_eq!(AggregationInterval::OneHour.to_seconds(), 3600);
    assert_eq!(AggregationInterval::OneDay.to_seconds(), 86400);
}

#[test]
fn test_record_and_query() {
    let store = MetricsStore::new(1000, Duration::hours(1));
    
    store.record("test_metric", MetricValue::Counter(100));
    
    let end = Utc::now() + Duration::seconds(1);
    let start = end - Duration::minutes(5);
    
    let points = store.query("test_metric", start, end);
    assert_eq!(points.len(), 1);
    
    match points[0].value {
        MetricValue::Counter(v) => assert_eq!(v, 100),
        _ => panic!("Expected counter value"),
    }
}

#[test]
fn test_retention_limit() {
    let store = MetricsStore::new(5, Duration::hours(1));
    
    // Add more points than max_points
    for i in 0..10 {
        store.record("test", MetricValue::Counter(i));
    }
    
    let end = Utc::now() + Duration::seconds(1);
    let start = end - Duration::hours(2);
    let points = store.query("test", start, end);
    
    // Should only keep max_points
    assert!(points.len() <= 5);
}

#[test]
fn test_list_metrics() {
    let store = MetricsStore::new(1000, Duration::hours(1));
    
    store.record("metric1", MetricValue::Counter(1));
    store.record("metric2", MetricValue::Gauge(2.0));
    store.record("metric3", MetricValue::Counter(3));
    
    let names = store.list_metrics();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"metric1".to_string()));
    assert!(names.contains(&"metric2".to_string()));
    assert!(names.contains(&"metric3".to_string()));
}

#[test]
fn test_clear() {
    let store = MetricsStore::new(1000, Duration::hours(1));
    
    store.record("test", MetricValue::Counter(1));
    assert_eq!(store.list_metrics().len(), 1);
    
    store.clear();
    assert_eq!(store.list_metrics().len(), 0);
}

#[test]
fn test_aggregation() {
    let store = MetricsStore::new(1000, Duration::hours(1));
    
    // Add multiple gauge values
    for i in 1..=10 {
        store.record("response_time", MetricValue::Gauge(i as f64 * 10.0));
    }
    
    let end = Utc::now() + Duration::seconds(1);
    let start = end - Duration::minutes(5);
    
    let aggregated = store.query_aggregated(
        "response_time",
        start,
        end,
        AggregationInterval::OneMinute,
    );
    
    // Should have aggregated data
    assert!(!aggregated.is_empty());
}

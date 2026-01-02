use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

/// Time-series metric data point.
///
/// Represents a single measurement at a specific point in time.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::services::metrics_store::{MetricPoint, MetricValue};
/// use chrono::Utc;
/// 
/// let point = MetricPoint {
///     timestamp: Utc::now(),
///     value: MetricValue::Counter(100),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Timestamp when the metric was recorded
    pub timestamp: DateTime<Utc>,
    
    /// Metric value
    pub value: MetricValue,
}

/// Metric value types.
///
/// Supports different types of metrics for flexible data collection.
/// 
/// # Types
/// 
/// - **Counter**: Monotonically increasing value (requests, errors)
/// - **Gauge**: Point-in-time value that can go up or down (active connections, memory usage)
/// - **Histogram**: Distribution of values (response times)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    /// Monotonically increasing counter
    Counter(u64),
    
    /// Point-in-time gauge value
    Gauge(f64),
    
    /// Histogram bucket (value, count)
    Histogram {
        /// Bucket upper bound
        le: f64,
        /// Count of values in bucket
        count: u64,
    },
}

/// Aggregation interval for metrics.
///
/// Defines how metrics should be grouped over time.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::services::metrics_store::AggregationInterval;
/// 
/// let interval = AggregationInterval::FiveMinutes;
/// assert_eq!(interval.to_seconds(), 300);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AggregationInterval {
    /// 1 minute intervals
    OneMinute,
    
    /// 5 minute intervals
    FiveMinutes,
    
    /// 1 hour intervals
    OneHour,
    
    /// 1 day intervals
    OneDay,
}

impl AggregationInterval {
    /// Returns the interval duration in seconds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::AggregationInterval;
    /// 
    /// assert_eq!(AggregationInterval::OneMinute.to_seconds(), 60);
    /// assert_eq!(AggregationInterval::FiveMinutes.to_seconds(), 300);
    /// assert_eq!(AggregationInterval::OneHour.to_seconds(), 3600);
    /// assert_eq!(AggregationInterval::OneDay.to_seconds(), 86400);
    /// ```
    pub fn to_seconds(&self) -> i64 {
        match self {
            AggregationInterval::OneMinute => 60,
            AggregationInterval::FiveMinutes => 300,
            AggregationInterval::OneHour => 3600,
            AggregationInterval::OneDay => 86400,
        }
    }
    
    /// Returns the Duration for this interval.
    pub fn to_duration(&self) -> Duration {
        Duration::seconds(self.to_seconds())
    }
}

/// Aggregated metric data.
///
/// Contains aggregated statistics for a time window.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::services::metrics_store::AggregatedMetric;
/// use chrono::Utc;
/// 
/// let metric = AggregatedMetric {
///     start_time: Utc::now(),
///     end_time: Utc::now(),
///     count: 100,
///     sum: 5000.0,
///     min: 10.0,
///     max: 200.0,
///     avg: 50.0,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Window start time
    pub start_time: DateTime<Utc>,
    
    /// Window end time
    pub end_time: DateTime<Utc>,
    
    /// Number of data points
    pub count: u64,
    
    /// Sum of all values
    pub sum: f64,
    
    /// Minimum value
    pub min: f64,
    
    /// Maximum value
    pub max: f64,
    
    /// Average value
    pub avg: f64,
}

/// Time-series data for a specific metric.
///
/// Stores historical data points with automatic rotation based on retention policy.
struct TimeSeries {
    /// Metric name
    #[allow(dead_code)]
    name: String,
    
    /// Data points (limited by retention policy)
    points: VecDeque<MetricPoint>,
    
    /// Maximum number of points to retain
    max_points: usize,
    
    /// Retention duration
    retention_duration: Duration,
}

impl TimeSeries {
    fn new(name: String, max_points: usize, retention_duration: Duration) -> Self {
        Self {
            name,
            points: VecDeque::with_capacity(max_points),
            max_points,
            retention_duration,
        }
    }
    
    fn add_point(&mut self, point: MetricPoint) {
        // Remove old points beyond retention
        let cutoff = Utc::now() - self.retention_duration;
        while let Some(oldest) = self.points.front() {
            if oldest.timestamp < cutoff {
                self.points.pop_front();
            } else {
                break;
            }
        }
        
        // Add new point
        self.points.push_back(point);
        
        // Enforce max points limit
        while self.points.len() > self.max_points {
            self.points.pop_front();
        }
    }
    
    fn get_points(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<MetricPoint> {
        self.points
            .iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .cloned()
            .collect()
    }
    
    fn aggregate(&self, start: DateTime<Utc>, end: DateTime<Utc>, interval: AggregationInterval) -> Vec<AggregatedMetric> {
        let interval_duration = interval.to_duration();
        let mut results = Vec::new();
        
        let mut current_start = start;
        while current_start < end {
            let current_end = current_start + interval_duration;
            if current_end > end {
                break;
            }
            
            let window_points: Vec<f64> = self.points
                .iter()
                .filter(|p| p.timestamp >= current_start && p.timestamp < current_end)
                .filter_map(|p| match p.value {
                    MetricValue::Counter(v) => Some(v as f64),
                    MetricValue::Gauge(v) => Some(v),
                    MetricValue::Histogram { .. } => None, // Skip histograms for now
                })
                .collect();
            
            if !window_points.is_empty() {
                let sum: f64 = window_points.iter().sum();
                let count = window_points.len() as u64;
                let min = window_points.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = window_points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let avg = sum / count as f64;
                
                results.push(AggregatedMetric {
                    start_time: current_start,
                    end_time: current_end,
                    count,
                    sum,
                    min,
                    max,
                    avg,
                });
            }
            
            current_start = current_end;
        }
        
        results
    }
}

/// Historical metrics storage service.
///
/// Stores time-series metrics data with configurable retention and aggregation.
/// Provides efficient query capabilities for time-range based analysis.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::services::metrics_store::{MetricsStore, MetricPoint, MetricValue};
/// use chrono::{Utc, Duration};
/// 
/// let store = MetricsStore::new(10000, Duration::hours(24));
/// 
/// // Record a metric
/// store.record("request_count", MetricValue::Counter(1));
/// 
/// // Query historical data
/// let end = Utc::now();
/// let start = end - Duration::hours(1);
/// let points = store.query("request_count", start, end);
/// ```
#[derive(Clone)]
pub struct MetricsStore {
    /// Time-series data per metric
    series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    
    /// Maximum points per metric
    max_points: usize,
    
    /// Retention duration
    retention_duration: Duration,
}

impl MetricsStore {
    /// Creates a new metrics store with specified retention policy.
    /// 
    /// # Arguments
    /// 
    /// * `max_points` - Maximum number of data points per metric
    /// * `retention_duration` - How long to keep historical data
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::MetricsStore;
    /// use chrono::Duration;
    /// 
    /// // Store last 10,000 points, keep data for 24 hours
    /// let store = MetricsStore::new(10000, Duration::hours(24));
    /// ```
    pub fn new(max_points: usize, retention_duration: Duration) -> Self {
        Self {
            series: Arc::new(RwLock::new(HashMap::new())),
            max_points,
            retention_duration,
        }
    }
    
    /// Records a new metric value.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Metric name
    /// * `value` - Metric value
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::{MetricsStore, MetricValue};
    /// use chrono::Duration;
    /// 
    /// let store = MetricsStore::new(1000, Duration::hours(1));
    /// store.record("requests_total", MetricValue::Counter(100));
    /// store.record("cpu_usage", MetricValue::Gauge(75.5));
    /// ```
    pub fn record(&self, name: &str, value: MetricValue) {
        let point = MetricPoint {
            timestamp: Utc::now(),
            value,
        };
        
        let mut series = self.series.write().unwrap();
        series
            .entry(name.to_string())
            .or_insert_with(|| TimeSeries::new(name.to_string(), self.max_points, self.retention_duration))
            .add_point(point);
    }
    
    /// Queries metric data for a time range.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Metric name
    /// * `start` - Start time (inclusive)
    /// * `end` - End time (inclusive)
    /// 
    /// # Returns
    /// 
    /// Vector of metric points in the specified time range
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::{MetricsStore, MetricValue};
    /// use chrono::{Utc, Duration};
    /// 
    /// let store = MetricsStore::new(1000, Duration::hours(1));
    /// store.record("requests", MetricValue::Counter(1));
    /// 
    /// let end = Utc::now();
    /// let start = end - Duration::minutes(5);
    /// let points = store.query("requests", start, end);
    /// ```
    pub fn query(&self, name: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<MetricPoint> {
        let series = self.series.read().unwrap();
        series
            .get(name)
            .map(|ts| ts.get_points(start, end))
            .unwrap_or_default()
    }
    
    /// Queries aggregated metric data for a time range.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Metric name
    /// * `start` - Start time (inclusive)
    /// * `end` - End time (inclusive)
    /// * `interval` - Aggregation interval
    /// 
    /// # Returns
    /// 
    /// Vector of aggregated metrics for each time window
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::{MetricsStore, MetricValue, AggregationInterval};
    /// use chrono::{Utc, Duration};
    /// 
    /// let store = MetricsStore::new(1000, Duration::hours(1));
    /// store.record("response_time", MetricValue::Gauge(50.0));
    /// 
    /// let end = Utc::now();
    /// let start = end - Duration::minutes(30);
    /// let aggregated = store.query_aggregated("response_time", start, end, AggregationInterval::FiveMinutes);
    /// ```
    pub fn query_aggregated(
        &self,
        name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: AggregationInterval,
    ) -> Vec<AggregatedMetric> {
        let series = self.series.read().unwrap();
        series
            .get(name)
            .map(|ts| ts.aggregate(start, end, interval))
            .unwrap_or_default()
    }
    
    /// Returns list of all metric names being tracked.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::{MetricsStore, MetricValue};
    /// use chrono::Duration;
    /// 
    /// let store = MetricsStore::new(1000, Duration::hours(1));
    /// store.record("metric1", MetricValue::Counter(1));
    /// store.record("metric2", MetricValue::Gauge(2.0));
    /// 
    /// let names = store.list_metrics();
    /// assert_eq!(names.len(), 2);
    /// ```
    pub fn list_metrics(&self) -> Vec<String> {
        let series = self.series.read().unwrap();
        series.keys().cloned().collect()
    }
    
    /// Clears all historical data.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::metrics_store::{MetricsStore, MetricValue};
    /// use chrono::Duration;
    /// 
    /// let store = MetricsStore::new(1000, Duration::hours(1));
    /// store.record("test", MetricValue::Counter(1));
    /// store.clear();
    /// assert_eq!(store.list_metrics().len(), 0);
    /// ```
    pub fn clear(&self) {
        let mut series = self.series.write().unwrap();
        series.clear();
    }
}



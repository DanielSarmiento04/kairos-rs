# WebSocket Metrics Streaming Guide

## Overview

Kairos-rs provides a WebSocket endpoint for real-time metrics streaming, allowing clients to receive live updates about gateway performance without polling.

## Endpoint

```
ws://your-gateway:5900/ws/metrics
```

## Features

- **Real-time Updates**: Receive metrics updates at configurable intervals (1-60 seconds)
- **Selective Subscriptions**: Subscribe to specific metrics or all available metrics
- **Historical Data**: Request historical time-series data on connection
- **Automatic Reconnection**: WebSocket connection with ping/pong heartbeat support
- **JSON Format**: Easy-to-parse JSON messages for all events

## Connection Protocol

### 1. Establish WebSocket Connection

Connect to the `/ws/metrics` endpoint using any WebSocket client.

### 2. Receive Initial Snapshot

Upon connection, the server immediately sends a snapshot of current metrics:

```json
{
  "type": "Snapshot",
  "timestamp": "2024-11-07T10:30:00Z",
  "requests_total": 1547,
  "requests_success": 1520,
  "requests_error": 27,
  "active_connections": 15,
  "avg_response_time": 45.2,
  "success_rate": 98.25
}
```

### 3. Send Subscription (Optional)

Customize which metrics you want to receive and how often:

```json
{
  "metrics": ["requests_total", "active_connections", "response_time_avg"],
  "interval_seconds": 2,
  "include_history": false
}
```

**Subscription Options:**

- `metrics`: Array of metric names (empty array = all metrics)
- `interval_seconds`: Update frequency in seconds (default: 1, range: 1-60)
- `include_history`: Request historical data (default: false)

### 4. Receive Periodic Updates

The server sends snapshot updates at the configured interval:

```json
{
  "type": "Snapshot",
  "timestamp": "2024-11-07T10:30:02Z",
  "requests_total": 1550,
  "requests_success": 1523,
  "requests_error": 27,
  "active_connections": 16,
  "avg_response_time": 44.8,
  "success_rate": 98.26
}
```

### 5. Historical Data (if requested)

When `include_history: true`, the server sends time-series data:

```json
{
  "type": "TimeSeries",
  "metric_name": "requests_total",
  "data_points": [
    {
      "timestamp": "2024-11-07T09:30:00Z",
      "value": 1000.0
    },
    {
      "timestamp": "2024-11-07T09:45:00Z",
      "value": 1200.0
    },
    {
      "timestamp": "2024-11-07T10:00:00Z",
      "value": 1400.0
    }
  ]
}
```

## Event Types

### Snapshot Event

Current metrics snapshot with all key performance indicators.

**Fields:**
- `type`: "Snapshot"
- `timestamp`: ISO 8601 timestamp
- `requests_total`: Total requests processed
- `requests_success`: Successful requests (2xx status codes)
- `requests_error`: Failed requests (4xx, 5xx status codes)
- `active_connections`: Current active connections
- `avg_response_time`: Average response time in milliseconds
- `success_rate`: Success rate percentage (0-100)

### TimeSeries Event

Historical time-series data for a specific metric.

**Fields:**
- `type`: "TimeSeries"
- `metric_name`: Name of the metric
- `data_points`: Array of {timestamp, value} objects

### Error Event

Error notification from the server.

**Fields:**
- `type`: "Error"
- `message`: Error description

### Heartbeat Event

Keep-alive message to maintain connection.

**Fields:**
- `type`: "Heartbeat"
- `timestamp`: ISO 8601 timestamp

## Client Examples

### JavaScript/Browser

```javascript
// Connect to WebSocket metrics endpoint
const ws = new WebSocket('ws://localhost:5900/ws/metrics');

ws.onopen = () => {
  console.log('Connected to metrics stream');
  
  // Subscribe to specific metrics with 2-second updates
  ws.send(JSON.stringify({
    metrics: ['requests_total', 'active_connections'],
    interval_seconds: 2,
    include_history: false
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'Snapshot':
      console.log('Metrics:', {
        total: message.requests_total,
        success_rate: message.success_rate,
        avg_response_time: message.avg_response_time
      });
      updateDashboard(message);
      break;
      
    case 'TimeSeries':
      console.log(`Historical data for ${message.metric_name}:`, message.data_points);
      renderChart(message);
      break;
      
    case 'Error':
      console.error('Server error:', message.message);
      break;
      
    case 'Heartbeat':
      console.log('Heartbeat received');
      break;
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected from metrics stream');
  // Implement reconnection logic here
  setTimeout(() => reconnect(), 5000);
};

function updateDashboard(metrics) {
  document.getElementById('total-requests').textContent = metrics.requests_total;
  document.getElementById('success-rate').textContent = `${metrics.success_rate.toFixed(2)}%`;
  document.getElementById('avg-response-time').textContent = `${metrics.avg_response_time.toFixed(2)}ms`;
}
```

### Python with websockets

```python
import asyncio
import websockets
import json

async def consume_metrics():
    uri = "ws://localhost:5900/ws/metrics"
    
    async with websockets.connect(uri) as websocket:
        # Send subscription
        subscription = {
            "metrics": ["requests_total", "success_rate"],
            "interval_seconds": 1,
            "include_history": True
        }
        await websocket.send(json.dumps(subscription))
        
        # Receive messages
        async for message in websocket:
            data = json.loads(message)
            
            if data["type"] == "Snapshot":
                print(f"Total: {data['requests_total']}, "
                      f"Success Rate: {data['success_rate']:.2f}%")
            
            elif data["type"] == "TimeSeries":
                print(f"Historical {data['metric_name']}: "
                      f"{len(data['data_points'])} points")
            
            elif data["type"] == "Error":
                print(f"Error: {data['message']}")

if __name__ == "__main__":
    asyncio.run(consume_metrics())
```

### Node.js with ws

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:5900/ws/metrics');

ws.on('open', () => {
  console.log('Connected to metrics stream');
  
  // Subscribe to all metrics with history
  ws.send(JSON.stringify({
    metrics: [],  // Empty = all metrics
    interval_seconds: 5,
    include_history: true
  }));
});

ws.on('message', (data) => {
  const message = JSON.parse(data);
  
  switch (message.type) {
    case 'Snapshot':
      console.log({
        timestamp: message.timestamp,
        requests: message.requests_total,
        errors: message.requests_error,
        active: message.active_connections
      });
      break;
      
    case 'TimeSeries':
      console.log(`${message.metric_name}: ${message.data_points.length} data points`);
      break;
      
    case 'Error':
      console.error('Server error:', message.message);
      break;
  }
});

ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});

ws.on('close', () => {
  console.log('Connection closed');
});
```

### Rust with tokio-tungstenite

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

#[tokio::main]
async fn main() {
    let url = "ws://localhost:5900/ws/metrics";
    
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");
    
    println!("Connected to metrics stream");
    
    let (mut write, mut read) = ws_stream.split();
    
    // Send subscription
    let subscription = json!({
        "metrics": ["requests_total", "response_time_avg"],
        "interval_seconds": 2,
        "include_history": false
    });
    
    write.send(Message::Text(subscription.to_string()))
        .await
        .expect("Failed to send subscription");
    
    // Read messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let event: serde_json::Value = serde_json::from_str(&text)
                    .expect("Failed to parse JSON");
                
                match event["type"].as_str() {
                    Some("Snapshot") => {
                        println!("Metrics - Total: {}, Success Rate: {}%",
                            event["requests_total"],
                            event["success_rate"]);
                    }
                    Some("Error") => {
                        eprintln!("Error: {}", event["message"]);
                    }
                    _ => {}
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
```

## Available Metrics

- `requests_total`: Total HTTP requests processed
- `requests_success`: Successful requests count
- `requests_error`: Failed requests count
- `active_connections`: Current active connections
- `avg_response_time`: Average response time (ms)
- `success_rate`: Success rate percentage

## Best Practices

### 1. Connection Management

```javascript
class MetricsClient {
  constructor(url) {
    this.url = url;
    this.ws = null;
    this.reconnectDelay = 1000;
    this.maxReconnectDelay = 30000;
  }
  
  connect() {
    this.ws = new WebSocket(this.url);
    
    this.ws.onopen = () => {
      console.log('Connected');
      this.reconnectDelay = 1000; // Reset delay on successful connection
      this.sendSubscription();
    };
    
    this.ws.onclose = () => {
      console.log('Disconnected, reconnecting...');
      setTimeout(() => {
        this.reconnectDelay = Math.min(
          this.reconnectDelay * 2,
          this.maxReconnectDelay
        );
        this.connect();
      }, this.reconnectDelay);
    };
    
    this.ws.onmessage = (event) => this.handleMessage(event);
  }
  
  sendSubscription() {
    this.ws.send(JSON.stringify({
      metrics: [],
      interval_seconds: 1,
      include_history: false
    }));
  }
  
  handleMessage(event) {
    const message = JSON.parse(event.data);
    // Process message...
  }
}

const client = new MetricsClient('ws://localhost:5900/ws/metrics');
client.connect();
```

### 2. Error Handling

Always handle WebSocket errors gracefully:

```javascript
ws.onerror = (error) => {
  console.error('WebSocket error:', error);
  // Log error to monitoring system
  // Don't expose sensitive information to users
};
```

### 3. Backpressure Management

For high-frequency updates, implement buffering:

```javascript
const buffer = [];
const BUFFER_SIZE = 100;

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  if (message.type === 'Snapshot') {
    buffer.push(message);
    
    if (buffer.length >= BUFFER_SIZE) {
      processBufferedMetrics(buffer.splice(0, BUFFER_SIZE));
    }
  }
};
```

### 4. Subscription Updates

You can change your subscription at any time:

```javascript
function updateSubscription(newMetrics, newInterval) {
  ws.send(JSON.stringify({
    metrics: newMetrics,
    interval_seconds: newInterval,
    include_history: false
  }));
}

// Change subscription after 10 seconds
setTimeout(() => {
  updateSubscription(['requests_total'], 5);
}, 10000);
```

## Performance Considerations

- **Interval Selection**: Use longer intervals (5-10s) for less critical monitoring
- **Selective Metrics**: Subscribe only to metrics you need
- **Connection Pooling**: Reuse connections instead of creating new ones
- **Buffer Management**: Implement client-side buffering for high-frequency updates
- **Historical Data**: Request historical data only when needed (not on every connection)

## Troubleshooting

### Connection Refused

```
Error: WebSocket connection failed
```

**Solutions:**
- Verify gateway is running
- Check the port (default: 5900)
- Ensure WebSocket endpoint is enabled

### Invalid Subscription

```json
{
  "type": "Error",
  "message": "Invalid subscription: missing field `metrics`"
}
```

**Solutions:**
- Verify JSON format
- Include all required fields
- Check metric names are valid

### No Messages Received

**Solutions:**
- Check `interval_seconds` is not too long
- Verify subscription was sent successfully
- Check network connectivity
- Monitor server logs

## Security Considerations

1. **TLS/SSL**: Use `wss://` protocol for production (secure WebSocket)
2. **Authentication**: Implement token-based authentication if needed
3. **Rate Limiting**: Server-side rate limiting prevents abuse
4. **Input Validation**: All subscription messages are validated server-side

## Related Documentation

- [Metrics Guide](../Readme.md#metrics-and-observability)
- [WebSocket Proxying Guide](./WEBSOCKET_GUIDE.md)
- [API Reference](../Readme.md#api-endpoints)

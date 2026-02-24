# Kairos CLI

[![Crates.io](https://img.shields.io/crates/v/kairos-cli.svg)](https://crates.io/crates/kairos-cli)
[![Documentation](https://docs.rs/kairos-cli/badge.svg)](https://docs.rs/kairos-cli)
[![License](https://img.shields.io/crates/l/kairos-cli.svg)](https://github.com/DanielSarmiento04/kairos-rs/blob/main/License)

Command-line interface for managing and interacting with Kairos API Gateway.

## Overview

`kairos-cli` provides a powerful command-line tool for managing Kairos Gateway instances, testing configurations, and monitoring gateway health.

## Installation

### From crates.io

```bash
cargo install kairos-cli
```

The binary is named `kairos`:

```bash
kairos --help
```

### From Source

```bash
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs
cargo build --release --bin kairos
```

The binary will be located at `target/release/kairos`.

## Features

- 🔧 **Configuration Management**: Validate and test gateway configurations
- 🏥 **Health Checks**: Monitor gateway and backend health
- 📊 **Metrics**: Query gateway metrics and statistics
- 🔍 **Route Testing**: Test route configurations before deployment
- 📝 **Config Generation**: Generate sample configurations
- 🚀 **Quick Setup**: Bootstrap new gateway configurations

## Usage

### Basic Commands

```bash
# Show help
kairos --help

# Show version
kairos --version

# Validate configuration
kairos validate config.json

# Test a route
kairos test-route --config config.json --path /api/users

# Check gateway health
kairos health --url http://localhost:5900

# Generate sample config
kairos init --output my-config.json
```

## Command Reference

### `kairos validate`

Validate a configuration file:

```bash
kairos validate [OPTIONS] <CONFIG_FILE>

Options:
  -v, --verbose    Show detailed validation output
  --strict         Enable strict validation mode
  -h, --help       Print help information
```

**Examples:**

```bash
# Basic validation
kairos validate config.json

# Verbose validation with details
kairos validate -v config.json

# Strict mode (error on warnings)
kairos validate --strict config.json
```

### `kairos test-route`

Test a specific route configuration:

```bash
kairos test-route [OPTIONS] --config <CONFIG> --path <PATH>

Options:
  -c, --config <CONFIG>      Path to configuration file
  -p, --path <PATH>          Route path to test
  -m, --method <METHOD>      HTTP method (default: GET)
  --backend <BACKEND>        Specific backend to test
  -h, --help                 Print help information
```

**Examples:**

```bash
# Test a GET route
kairos test-route --config config.json --path /api/users

# Test a POST route
kairos test-route --config config.json --path /api/users --method POST

# Test specific backend
kairos test-route --config config.json --path /api/users --backend http://localhost:8080
```

### `kairos health`

Check gateway health status:

```bash
kairos health [OPTIONS]

Options:
  -u, --url <URL>          Gateway URL (default: http://localhost:5900)
  --check-backends         Also check backend health
  --timeout <SECONDS>      Request timeout in seconds (default: 5)
  -h, --help              Print help information
```

**Examples:**

```bash
# Check gateway health
kairos health

# Check with custom URL
kairos health --url http://gateway.example.com:5900

# Check gateway and all backends
kairos health --check-backends
```

### `kairos init`

Initialize a new gateway configuration:

```bash
kairos init [OPTIONS]

Options:
  -o, --output <FILE>         Output file path (default: config.json)
  -t, --template <TEMPLATE>   Configuration template to use
  --protocol <PROTOCOL>       Protocol type (http, websocket, ftp, dns)
  -h, --help                  Print help information
```

**Templates:**

- `basic` - Simple HTTP proxy configuration
- `advanced` - Full-featured configuration with all options
- `multi-protocol` - Example with multiple protocol types
- `kubernetes` - Configuration for Kubernetes deployment

**Examples:**

```bash
# Generate basic configuration
kairos init

# Use advanced template
kairos init --template advanced --output gateway-config.json

# Generate WebSocket configuration
kairos init --protocol websocket --output ws-config.json
```

### `kairos metrics`

Query gateway metrics:

```bash
kairos metrics [OPTIONS]

Options:
  -u, --url <URL>          Gateway URL (default: http://localhost:5900)
  --format <FORMAT>        Output format (json, text, prometheus)
  --filter <FILTER>        Metric name filter
  -h, --help              Print help information
```

**Examples:**

```bash
# Get all metrics
kairos metrics

# Get metrics in JSON format
kairos metrics --format json

# Filter specific metrics
kairos metrics --filter request_count
```

### `kairos config`

Configuration management commands:

```bash
kairos config <SUBCOMMAND>

Subcommands:
  show       Show current configuration
  diff       Compare two configurations
  merge      Merge multiple configurations
  format     Format and prettify configuration
  convert    Convert between configuration formats

Options:
  -h, --help    Print help information
```

**Examples:**

```bash
# Show configuration
kairos config show config.json

# Compare configurations
kairos config diff config-old.json config-new.json

# Merge configurations
kairos config merge base.json override.json --output final.json

# Format configuration
kairos config format config.json --indent 2
```

## Configuration File Example

```json
{
  "version": 1,
  "routers": [
    {
      "protocol": "http",
      "external_path": "/api/users",
      "internal_path": "/users",
      "methods": ["GET", "POST"],
      "backends": [
        {
          "host": "http://localhost",
          "port": 8080,
          "weight": 1
        }
      ],
      "load_balancing_strategy": "round_robin",
      "auth_required": false
    }
  ]
}
```

## Use Cases

### 1. Pre-deployment Validation

```bash
# Validate before deploying
kairos validate production-config.json --strict

# Test critical routes
kairos test-route --config production-config.json --path /api/critical

# Check backend connectivity
kairos health --url http://staging-gateway:5900 --check-backends
```

### 2. Configuration Development

```bash
# Generate starting point
kairos init --template advanced --output my-config.json

# Edit configuration...

# Validate changes
kairos validate my-config.json -v

# Test locally
kairos test-route --config my-config.json --path /api/test
```

### 3. Production Monitoring

```bash
# Monitor gateway health
kairos health --url http://production-gateway:5900

# Check metrics
kairos metrics --url http://production-gateway:5900 --format json

# Verify configuration
kairos config show /etc/kairos/config.json
```

### 4. CI/CD Integration

```bash
#!/bin/bash
# validate.sh

# Validate configuration
if ! kairos validate config/gateway.json --strict; then
  echo "Configuration validation failed!"
  exit 1
fi

# Test routes
for route in "/api/users" "/api/products" "/api/orders"; do
  if ! kairos test-route --config config/gateway.json --path "$route"; then
    echo "Route test failed: $route"
    exit 1
  fi
done

echo "All validations passed!"
```

## Environment Variables

- `KAIROS_GATEWAY_URL`: Default gateway URL for commands
- `KAIROS_CONFIG_PATH`: Default configuration file path
- `KAIROS_TIMEOUT`: Default request timeout in seconds

**Example:**

```bash
export KAIROS_GATEWAY_URL=http://localhost:5900
export KAIROS_CONFIG_PATH=/etc/kairos/config.json

# Now you can omit these options
kairos health
kairos validate
```

## Exit Codes

- `0` - Success
- `1` - Validation/test failure
- `2` - Connection error
- `3` - Configuration error
- `4` - Authentication error

## Integration Examples

### Shell Script

```bash
#!/bin/bash

# Check if gateway is healthy
if ! kairos health --url http://localhost:5900; then
  echo "Gateway is unhealthy, restarting..."
  systemctl restart kairos-gateway
  sleep 5
fi

# Validate config before reload
if kairos validate /etc/kairos/config.json; then
  echo "Reloading gateway configuration..."
  systemctl reload kairos-gateway
else
  echo "Invalid configuration, aborting reload"
  exit 1
fi
```

### GitHub Actions

```yaml
name: Validate Configuration

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Kairos CLI
        run: cargo install kairos-cli
      
      - name: Validate Configuration
        run: kairos validate config.json --strict
      
      - name: Test Routes
        run: |
          kairos test-route --config config.json --path /api/users
          kairos test-route --config config.json --path /api/products
```

### Kubernetes Deployment

```bash
# Validate config before creating ConfigMap
kairos validate k8s-config.json --strict

# Create ConfigMap
kubectl create configmap kairos-config --from-file=config.json=k8s-config.json

# Deploy gateway
kubectl apply -f kairos-deployment.yaml

# Wait for gateway to be ready
kubectl wait --for=condition=ready pod -l app=kairos-gateway

# Verify health
kairos health --url http://$(kubectl get svc kairos-gateway -o jsonpath='{.status.loadBalancer.ingress[0].ip}'):5900
```

## Troubleshooting

### Connection Issues

```bash
# Test with increased timeout
kairos health --timeout 30

# Check network connectivity
curl http://localhost:5900/health
```

### Validation Errors

```bash
# Use verbose mode for details
kairos validate config.json -v

# Check specific format issues
kairos config format config.json
```

## Dependencies

This crate works with the [`kairos-gateway`](https://crates.io/crates/kairos-gateway) binary.

## Documentation

- [Main Documentation](../../Readme.md)
- [Guides](../../docs/src/GUIDES.md)
- [API Documentation](https://docs.rs/kairos-cli)
- [Examples](../../examples/)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/DanielSarmiento04/kairos-rs) for contribution guidelines.

## License

Licensed under MIT License. See [LICENSE](../../License) for details.

## Links

- [GitHub Repository](https://github.com/DanielSarmiento04/kairos-rs)
- [Issue Tracker](https://github.com/DanielSarmiento04/kairos-rs/issues)
- [Crates.io](https://crates.io/crates/kairos-cli)
- [Documentation](https://docs.rs/kairos-cli)

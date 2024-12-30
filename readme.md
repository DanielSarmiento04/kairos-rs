# Memo: API Gateway

## Overview

Memo is a lightweight, scalable API Gateway built using Rust and the Actix Web framework. It is designed to route incoming HTTP requests to the appropriate backend services, enforce configuration rules, and handle errors gracefully.

---

## Features

- **Dynamic Routing**: Routes requests based on external paths defined in a YAML configuration file.
- **Protocol Support**: Handles both HTTP and HTTPS protocols.
- **Timeout Management**: Ensures upstream requests do not hang.
- **Header Forwarding**: Forwards all client headers, excluding connection-specific ones.
- **Configurable via YAML**: Easily specify routes, domains, ports, and allowed methods.
- **Error Handling**: Detailed error responses for timeout, invalid configurations, and upstream errors.

---

## Requirements

- **Rust**: Version 1.65 or higher
- **Cargo**: For managing dependencies and building the project
- **YAML Config File**: A `config.yml` file must be present for routing definitions

---

## Installation

### Clone the Repository

```bash
git clone https://github.com/DanielSarmiento04/memo
cd memo
```

### Build the Project

```bash
cargo build --release
```

### Run the Server

```bash
cargo run
```

---

## Configuration

The API Gateway relies on a `config.yml` file for routing definitions. Below is the structure of the configuration file:

```yaml
version: 1

routes:
  - name: localhost
    domain: localhost
    port: 8080
    protocol: http
    external_path: /identity/register
    internal_path: /api/identity/register
    methods:
      - POST
      - GET
  - name: google
    domain: google.com
    port: 443
    protocol: https
    external_path: /identity/register/v2
    internal_path: /api/identity/register/v2
    methods:
      - POST
      - GET
```

### Key Fields

- **name**: Descriptive name for the route. Used for easier identification in the configuration file.
- **domain**: The target domain for forwarding requests. Specifies the upstream service's domain name.
- **port**: The target port of the backend service. Common values are `80` for HTTP and `443` for HTTPS.
- **protocol**: Protocol for forwarding (`http` or `https`). Determines whether the request is sent securely.
- **external\_path**: Path that clients use to access the service. It must begin with `/` and is matched against incoming request paths.
- **internal\_path**: Path on the backend server where requests are forwarded. It must also begin with `/` and specifies the upstream service's endpoint.
- **methods**: List of HTTP methods allowed for this route. Examples include `GET`, `POST`, `PUT`, etc.

---

## Usage

### Example Request

Assume the following route in `config.yml`:

```yaml
routes:
  - name: localhost
    domain: localhost
    port: 8080
    protocol: http
    external_path: /identity/register
    internal_path: /api/identity/register
    methods:
      - POST
      - GET
```

You can make a request to the gateway like this:

```bash
curl --location 'http://localhost:8080/identity/register' \
--header 'Content-Type: application/json' \
--data '{"username": "example", "password": "12345"}'
```

The gateway will forward the request to `http://localhost:8080/api/identity/register`.

---

## Project Structure

- **main.rs**: Contains the main application logic, including server initialization and request routing.
- **yaml\_config.rs**: Handles parsing and validation of the `config.yml` file.
- **error\_handler.rs**: Defines custom error types and their corresponding HTTP responses.
- **redirect\_service.rs**: Contains helper functions for formatting and forwarding requests.

---

## Error Responses

| Error Type            | HTTP Status Code          | Description                                  |
| --------------------- | ------------------------- | -------------------------------------------- |
| Timeout               | 504 Gateway Timeout       | Request to the upstream service timed out.   |
| Internal Server Error | 500 Internal Server Error | An unexpected error occurred in the gateway. |
| Config Error          | 502 Bad Gateway           | Invalid or missing route configuration.      |
| Upstream Error        | 502 Bad Gateway           | Error occurred while forwarding the request. |

---

## Logging

The application uses `env_logger` for structured logging. To enable logging, set the `RUST_LOG` environment variable before running the application:

```bash
RUST_LOG=info cargo run
```

Logs include details about received requests, routing decisions, and upstream responses.

---

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

1. Fork the repository.
2. Create a feature branch: `git checkout -b feature-name`
3. Commit changes: `git commit -m 'Add feature'`
4. Push to the branch: `git push origin feature-name`
5. Open a pull request.

---

## License

This project is licensed under the MIT License. See `LICENSE` for details.

---

## Troubleshooting

### Common Issues

1. **Route Not Found**:

   - Ensure `config.yml` is properly formatted and matches the requested path.
   - Restart the server after modifying the configuration.

2. **Timeouts**:

   - Check upstream service availability.
   - Increase the timeout duration in `main.rs`.

3. **Headers Not Forwarded**:

   - Ensure headers are correctly formatted in the request.

For further assistance, contact support at `support@memo-gateway.com`.


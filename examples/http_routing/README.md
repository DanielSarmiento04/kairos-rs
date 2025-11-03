# HTTP Routing Example with Kairos Gateway

This example demonstrates how to use the Kairos API Gateway with a FastAPI backend service, showcasing HTTP routing capabilities.

## Project Structure

```
.
├── app/
│   ├── __init__.py      # FastAPI app initialization
│   └── views.py         # API endpoints
├── main.py              # Application entry point
├── Dockerfile           # Container configuration for FastAPI app
├── compose.yml          # Docker Compose orchestration
├── config.json          # Kairos gateway configuration
└── requirements.txt     # Python dependencies
```

## Prerequisites

- Docker and Docker Compose
- Python 3.12+ (for local development)

## Configuration

The `config.json` file configures the Kairos gateway:

- **JWT Settings**: Authentication configuration (currently auth is disabled for the example endpoint)
- **Router**: Defines the HTTP routing rule
  - External path: `/test/health` (accessed via gateway)
  - Internal path: `/` (actual backend endpoint)
  - Backend: `http://app_http:80`
  - Methods: `GET`
  - Auth required: `false`

## Running the Application

### Using Docker Compose (Recommended)

1. Build and start the services:
```bash
docker-compose up --build
```

2. The services will be available at:
   - **Kairos Gateway**: `http://localhost:5900`
   - **FastAPI App** (direct access): `http://localhost:80`

### Testing the Endpoints

Access the routed endpoint through the Kairos gateway:
```bash
curl http://localhost:5900/test/health
```

Expected response:
```json
{"Hello": "World"}
```

Direct access to the FastAPI app:
```bash
curl http://localhost:80/
```

## Services

### app_http
- **Image**: Built from local Dockerfile
- **Framework**: FastAPI
- **Port**: 80
- **Endpoint**: `GET /` returns `{"Hello": "World"}`

### gateway
- **Image**: `ghcr.io/danielsarmiento04/kairos-rs:main`
- **Port**: 5900
- **Purpose**: API Gateway routing requests to backend services
- **Configuration**: Loaded from `config.json`

## Local Development

Install dependencies:
```bash
pip install -r requirements.txt
```

Run the FastAPI app locally:
```bash
uvicorn main:app --host 0.0.0.0 --port 80
```

## Networks

Both services are connected via the `kairos-network` bridge network, allowing them to communicate using service names.

## Notes

- The gateway rewrites `/test/health` to `/` when forwarding to the backend
- Authentication is disabled for this example (`auth_required: false`)
- The Kairos gateway image supports both `linux/amd64` and `linux/arm64` architectures

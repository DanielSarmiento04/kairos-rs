#!/usr/bin/env python3
"""
JWT Token Generator for Kairos-rs Gateway

This script generates valid JWT tokens that match your current gateway configuration.
"""

import jwt
import json
import os
import sys
from datetime import datetime, timedelta, timezone

CONFIG_PATH = os.environ.get("KAIROS_CONFIG_PATH", "config.json")


def _load_jwt_config():
    """Load the ``jwt`` section from the gateway config file.

    Honors ``KAIROS_CONFIG_PATH`` (same env var the gateway uses) and falls
    back to ``./config.json``. Exits with a clear error if the file is
    missing or has no ``jwt`` section.
    """
    path = CONFIG_PATH
    if not os.path.isfile(path):
        print(f"❌ Config file not found: {os.path.abspath(path)}")
        print("   Set KAIROS_CONFIG_PATH or pass a path as the first argument.")
        sys.exit(1)
    try:
        with open(path) as f:
            cfg = json.load(f)
    except json.JSONDecodeError as e:
        print(f"❌ Config file is not valid JSON ({path}): {e}")
        sys.exit(1)
    jwt_cfg = cfg.get("jwt")
    if not jwt_cfg:
        print(f"❌ Config file {path} has no 'jwt' section")
        sys.exit(1)
    for key in ("secret", "issuer", "audience"):
        if not jwt_cfg.get(key):
            print(f"❌ Config file {path} jwt.{key} is missing or empty")
            sys.exit(1)
    return jwt_cfg


# Allow ``python3 generate_jwt.py /path/to/config.json`` to override.
if len(sys.argv) > 1:
    CONFIG_PATH = sys.argv[1]

def generate_token(user_id="testuser123", expires_in_hours=24):
    """
    Generate a JWT token with the correct configuration.
    
    Args:
        user_id (str): Subject identifier for the token
        expires_in_hours (int): Token expiration time in hours
    
    Returns:
        str: Valid JWT token
    """
    
    # Current time
    now = datetime.now(timezone.utc)
    
    # Create payload with required claims
    cfg = _load_jwt_config()
    payload = {
        "sub": user_id,  # Subject (required)
        "exp": now + timedelta(hours=expires_in_hours),  # Expiration (required)
        "iat": now,  # Issued at
        "iss": cfg["issuer"],  # Issuer
        "aud": cfg["audience"],  # Audience
        "user_id": user_id,  # Custom claim
        "role": "user"  # Custom claim
    }
    
    # Generate token
    token = jwt.encode(payload, cfg["secret"], algorithm="HS256")
    
    return token

def verify_token(token):
    """
    Verify a JWT token with the current configuration.
    
    Args:
        token (str): JWT token to verify
        
    Returns:
        dict: Decoded payload if valid, None if invalid
    """
    try:
        cfg = _load_jwt_config()
        payload = jwt.decode(
            token,
            cfg["secret"],
            algorithms=["HS256"],
            issuer=cfg["issuer"],
            audience=cfg["audience"],
            options={"require": cfg.get("required_claims", ["sub"])}
        )
        return payload
    except jwt.InvalidTokenError as e:
        print(f"Token validation error: {e}")
        return None

def main():
    print("🔑 Kairos-rs JWT Token Generator")
    print("=" * 50)
    
    # Generate new token
    print("\n1. Generating new JWT token...")
    token = generate_token(user_id="testuser123", expires_in_hours=24)
    print(f"✅ Token generated successfully!")
    
    # Display token
    print(f"\n📋 Your JWT Token (valid for 24 hours):")
    print("-" * 50)
    print(token)
    print("-" * 50)
    
    # Verify the token works
    print(f"\n🔍 Verifying token...")
    payload = verify_token(token)
    if payload:
        print("✅ Token is valid!")
        print(f"📄 Payload: {json.dumps(payload, indent=2, default=str)}")
    else:
        print("❌ Token verification failed!")
    
    # Generate curl command
    print(f"\n🚀 curl command to test:")
    print("-" * 50)
    curl_cmd = f"""curl --location 'http://localhost:5900/protected/cats/404' \\
--header 'Authorization: Bearer {token}'"""
    print(curl_cmd)
    print("-" * 50)
    
    # Generate Postman format
    print(f"\n📮 For Postman:")
    print(f"Authorization Header: Bearer {token}")
    
    print(f"\n✨ Token expires: {datetime.now(timezone.utc) + timedelta(hours=24)}")

if __name__ == "__main__":
    try:
        main()
    except ImportError:
        print("❌ PyJWT library not found!")
        print("📦 Install it with: pip install PyJWT")
        print("🔧 Or run: python3 -m pip install PyJWT")

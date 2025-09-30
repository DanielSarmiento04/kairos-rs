#!/usr/bin/env python3
"""
JWT Token Generator for Kairos-rs Gateway

This script generates valid JWT tokens that match your current gateway configuration.
"""

import jwt
import json
from datetime import datetime, timedelta, timezone

# Configuration from your config.json
SECRET = "your-super-secure-jwt-secret-key-must-be-at-least-32-characters-long"
ISSUER = "kairos-gateway"
AUDIENCE = "api-clients"
REQUIRED_CLAIMS = ["sub", "exp"]

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
    payload = {
        "sub": user_id,  # Subject (required)
        "exp": now + timedelta(hours=expires_in_hours),  # Expiration (required)
        "iat": now,  # Issued at
        "iss": ISSUER,  # Issuer
        "aud": AUDIENCE,  # Audience
        "user_id": user_id,  # Custom claim
        "role": "user"  # Custom claim
    }
    
    # Generate token
    token = jwt.encode(payload, SECRET, algorithm="HS256")
    
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
        payload = jwt.decode(
            token,
            SECRET,
            algorithms=["HS256"],
            issuer=ISSUER,
            audience=AUDIENCE,
            options={"require": ["sub", "exp"]}
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

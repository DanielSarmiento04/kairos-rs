#!/usr/bin/env python3
"""
Simple HTTP server for serving the Kairos UI WASM application
This is a fallback for when the Leptos SSR server has issues
"""

import http.server
import socketserver
import os
import sys
import webbrowser
from pathlib import Path

PORT = 3000

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory="target/site", **kwargs)
    
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

def main():
    # Change to the kairos-ui directory
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    # Check if target/site exists
    if not Path("target/site").exists():
        print("❌ target/site directory not found. Please build the WASM first:")
        print("   cargo build --lib --target wasm32-unknown-unknown")
        print("   or")
        print("   cargo leptos build")
        sys.exit(1)
    
    print(f"🚀 Serving Kairos UI on http://localhost:{PORT}")
    print("📦 Serving WASM application from target/site")
    print("🛑 Press Ctrl+C to stop")
    
    try:
        with socketserver.TCPServer(("", PORT), Handler) as httpd:
            print(f"✅ Server started successfully!")
            
            # Open browser
            webbrowser.open(f"http://localhost:{PORT}")
            
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Server stopped")
    except OSError as e:
        if e.errno == 48:  # Address already in use
            print(f"❌ Port {PORT} is already in use. Please stop other services or use a different port.")
        else:
            print(f"❌ Error starting server: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
/**
 * Simple WebSocket Echo Server for testing Kairos Gateway
 * 
 * This server listens on localhost:3000/ws and echoes back any messages it receives.
 * Use this to test the WebSocket proxying functionality of the Kairos gateway.
 * 
 * Usage:
 *   bun run server.ts
 * 
 * Then connect through the gateway:
 *   wscat -c "ws://localhost:5900/ws/chat"
 */

const server = Bun.serve({
  port: 3000,
  fetch(req, server) {
    const url = new URL(req.url);
    
    // Only handle WebSocket upgrade requests to /ws
    if (url.pathname === "/ws") {
      const success = server.upgrade(req);
      if (success) {
        console.log(`[${new Date().toISOString()}] WebSocket connection upgraded`);
        return undefined;
      }
      return new Response("WebSocket upgrade failed", { status: 500 });
    }
    
    // Return 404 for all other paths
    return new Response("Not Found - Use /ws for WebSocket connections", { 
      status: 404 
    });
  },
  
  websocket: {
    open(ws) {
      console.log(`[${new Date().toISOString()}] Client connected`);
      ws.send(JSON.stringify({ 
        type: "welcome", 
        message: "Connected to WebSocket echo server on localhost:3000/ws" 
      }));
    },
    
    message(ws, message) {
      console.log(`[${new Date().toISOString()}] Received:`, message);
      
      // Echo the message back
      ws.send(JSON.stringify({
        type: "echo",
        original: message,
        timestamp: new Date().toISOString()
      }));
    },
    
    close(ws, code, reason) {
      console.log(`[${new Date().toISOString()}] Client disconnected - Code: ${code}, Reason: ${reason}`);
    }
  }
});

console.log(`🚀 WebSocket Echo Server running on ws://localhost:${server.port}/ws`);
console.log(`📡 Ready to accept connections through Kairos Gateway at ws://localhost:5900/ws/chat`);

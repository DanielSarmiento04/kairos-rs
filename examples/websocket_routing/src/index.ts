import { Hono } from 'hono'
import { upgradeWebSocket } from 'hono/bun'

const app = new Hono()

app.get('/', (c) => {
  return c.text('Hello Hono!')
})

app.get(
  '/ws',
  upgradeWebSocket((c) => {
    return {
      onMessage(event, ws) {
        console.log(`Message from client: ${event.data}`)
        ws.send('Hello from server!')
      },
      onClose: () => {
        console.log('Connection closed')
      },
    }
  })
)

const port = 3000

Bun.serve({
  fetch: app.fetch,
  websocket: {
    message(ws, message) {}, // required but handled by Hono
    open(ws) {}, // required but handled by Hono
    close(ws) {}, // required but handled by Hono
  },
  port,
})

console.log(`Server running at http://localhost:${port}`)

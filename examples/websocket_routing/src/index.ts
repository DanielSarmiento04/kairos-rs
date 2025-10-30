import { Hono } from 'hono'
import { upgradeWebSocket, websocket } from 'hono/bun'

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
        ws.send(`Echo: ${event.data}`)
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
  websocket,
  port,
})

console.log(`Server running at http://localhost:${port}`)

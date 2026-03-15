import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/api/auth/$')({
  server: {
    handlers: {
      GET: () => new Response(null, { status: 404 }),
      POST: () => new Response(null, { status: 404 }),
    },
  },
})

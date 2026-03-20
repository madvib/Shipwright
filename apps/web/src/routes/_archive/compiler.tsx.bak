import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/compiler')({
  beforeLoad: () => { throw redirect({ to: '/studio' }) },
  component: () => null,
})

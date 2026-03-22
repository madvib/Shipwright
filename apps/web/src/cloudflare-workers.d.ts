// Ambient type declaration for cloudflare:workers module.
// At runtime this resolves to the real Cloudflare Workers API.
// For local tsc, this declaration provides the env binding typed
// against the Env interface from worker-configuration.d.ts.
declare module 'cloudflare:workers' {
  const env: Env
}

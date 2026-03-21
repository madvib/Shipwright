// Curated fallback servers — shown when the proxy API is unreachable.
// Shape matches the proxy API response (McpServer from mcp-registry.ts).

export interface ProxyServer {
  id: string
  name: string
  description: string | null
  homepage: string | null
  tags: string[]
  vendor: string | null
  packageRegistry: string | null
  command: string | null
  args: string[]
  vetted: boolean
  imageUrl: string | null
}

export const CURATED_FALLBACK: ProxyServer[] = [
  { id: 'github', name: 'GitHub', description: 'Search repos, manage PRs and issues, create commits.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/github', tags: ['popular', 'dev'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'filesystem', name: 'Filesystem', description: 'Read and write local files within configurable allowed paths.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem', tags: ['popular', 'files'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'brave-search', name: 'Brave Search', description: 'Web search via the Brave Search API.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search', tags: ['search', 'web'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-brave-search'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'memory', name: 'Memory', description: 'Persistent knowledge graph -- remember facts across sessions.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/memory', tags: ['memory', 'context'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-memory'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'slack', name: 'Slack', description: 'Read channels, post messages, and manage Slack workspaces.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/slack', tags: ['comms', 'popular'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-slack'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'linear', name: 'Linear', description: 'Manage issues, projects, and cycles in Linear.', homepage: 'https://github.com/linear/linear/tree/master/packages/mcp', tags: ['pm', 'issues'], vendor: 'Linear', packageRegistry: 'npm', command: 'npx', args: ['-y', '@linear/mcp-server'], vetted: true, imageUrl: 'https://github.com/linear.png' },
  { id: 'playwright', name: 'Playwright', description: 'Browser automation -- navigate, screenshot, interact with web pages.', homepage: 'https://github.com/executeautomation/mcp-playwright', tags: ['browser', 'testing'], vendor: 'ExecuteAutomation', packageRegistry: 'npm', command: 'npx', args: ['-y', '@executeautomation/playwright-mcp-server'], vetted: true, imageUrl: 'https://github.com/executeautomation.png' },
  { id: 'postgres', name: 'PostgreSQL', description: 'Query and inspect a PostgreSQL database with read-only access.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/postgres', tags: ['database', 'sql'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'puppeteer', name: 'Puppeteer', description: 'Headless browser control for scraping, screenshots, and automation.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer', tags: ['browser', 'scraping'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-puppeteer'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'sqlite', name: 'SQLite', description: 'Read and write a local SQLite database file.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite', tags: ['database', 'local'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-sqlite'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
]

import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('#/lib/github-token', () => ({
  getGitHubToken: vi.fn(),
}))

vi.mock('#/lib/fetch-repo-files', () => ({
  fetchRepoFiles: vi.fn(),
}))

vi.mock('#/lib/github-import', () => ({
  extractLibrary: vi.fn(),
}))

import * as ghToken from '#/lib/github-token'
import * as fetchFiles from '#/lib/fetch-repo-files'
import * as ghImport from '#/lib/github-import'
import { Route } from '../repos-config'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const GET = (Route.options.server!.handlers as any).GET!

function makeRequest(params: Record<string, string> = {}): Request {
  const url = new URL('http://localhost/api/github/repos-config')
  for (const [k, v] of Object.entries(params)) {
    url.searchParams.set(k, v)
  }
  return new Request(url.toString())
}

describe('GET /api/github/repos-config', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('returns 401 when no GitHub token is available', async () => {
    vi.mocked(ghToken.getGitHubToken).mockResolvedValue(null)
    const res = await GET({ request: makeRequest() } as Parameters<typeof GET>[0])
    expect(res.status).toBe(401)
  })

  it('returns 400 when owner or repo is missing', async () => {
    vi.mocked(ghToken.getGitHubToken).mockResolvedValue('test-token')
    const res = await GET({ request: makeRequest({ owner: 'foo' }) } as Parameters<typeof GET>[0])
    expect(res.status).toBe(400)
  })

  it('returns 404 when repo is not found', async () => {
    vi.mocked(ghToken.getGitHubToken).mockResolvedValue('test-token')
    vi.mocked(fetchFiles.fetchRepoFiles).mockResolvedValue('not_found')
    const res = await GET({
      request: makeRequest({ owner: 'foo', repo: 'bar' }),
    } as Parameters<typeof GET>[0])
    expect(res.status).toBe(404)
  })

  it('returns 422 when no config is found in repo', async () => {
    vi.mocked(ghToken.getGitHubToken).mockResolvedValue('test-token')
    vi.mocked(fetchFiles.fetchRepoFiles).mockResolvedValue({ 'README.md': '# hello' })
    vi.mocked(ghImport.extractLibrary).mockReturnValue(null)
    const res = await GET({
      request: makeRequest({ owner: 'foo', repo: 'bar' }),
    } as Parameters<typeof GET>[0])
    expect(res.status).toBe(422)
  })

  it('returns library when config is found', async () => {
    vi.mocked(ghToken.getGitHubToken).mockResolvedValue('test-token')
    vi.mocked(fetchFiles.fetchRepoFiles).mockResolvedValue({ 'CLAUDE.md': '# rules' })
    const mockLibrary = {
      modes: [],
      active_agent: null,
      mcp_servers: [],
      skills: [],
      rules: [{ file_name: 'CLAUDE.md', content: '# rules' }],
      agent_profiles: [],
      claude_team_agents: [],
      env: {},
      available_models: [],
      provider_defaults: {},
    }
    vi.mocked(ghImport.extractLibrary).mockReturnValue(mockLibrary)
    const res = await GET({
      request: makeRequest({ owner: 'foo', repo: 'bar' }),
    } as Parameters<typeof GET>[0])
    expect(res.status).toBe(200)
    const body = await res.json()
    expect(body.rules).toHaveLength(1)
    expect(body.rules[0].file_name).toBe('CLAUDE.md')
  })
})

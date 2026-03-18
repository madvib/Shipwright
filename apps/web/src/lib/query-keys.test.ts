import { describe, it, expect } from 'vitest'
import { studioKeys, authKeys, githubKeys } from './query-keys'

describe('studioKeys', () => {
  it('builds hierarchical keys for cache invalidation', () => {
    expect(studioKeys.all).toEqual(['studio'])
    expect(studioKeys.library('lib-1')).toEqual(['studio', 'library', 'lib-1'])
    expect(studioKeys.profiles('prof-1')).toEqual(['studio', 'profiles', 'prof-1'])
    expect(studioKeys.workspaces()).toEqual(['studio', 'workspaces'])
    expect(studioKeys.workspace('ws-1')).toEqual(['studio', 'workspaces', 'ws-1'])
  })

  it('allows invalidating all studio queries by prefix', () => {
    // The `all` key is a prefix of every other key
    const lib = studioKeys.library('x')
    const prof = studioKeys.profiles('y')
    expect(lib[0]).toBe(studioKeys.all[0])
    expect(prof[0]).toBe(studioKeys.all[0])
  })
})

describe('authKeys', () => {
  it('builds auth key hierarchy', () => {
    expect(authKeys.all).toEqual(['auth'])
    expect(authKeys.session()).toEqual(['auth', 'session'])
    expect(authKeys.me()).toEqual(['auth', 'me'])
  })
})

describe('githubKeys', () => {
  it('builds github key hierarchy', () => {
    expect(githubKeys.all).toEqual(['github'])
    expect(githubKeys.repos()).toEqual(['github', 'repos', 1])
    expect(githubKeys.repos(3)).toEqual(['github', 'repos', 3])
  })
})

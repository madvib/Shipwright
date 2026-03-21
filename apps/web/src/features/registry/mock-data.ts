import type { RegistryPackage, PackageVersion, PackageSkill } from './types'

/** Mock packages for development before the API is deployed. */
export const MOCK_PACKAGES: RegistryPackage[] = [
  {
    id: 'pkg_01', path: 'github.com/anthropic/claude-code', name: 'Claude Code Agents',
    description: 'Official agent configuration for Claude Code — includes commit conventions, code review, and testing skills.',
    scope: 'official', repo_url: 'https://github.com/anthropic/claude-code', default_branch: 'main',
    latest_version: '0.4.2', content_hash: 'abc123', source_type: 'native', claimed_by: 'u_anth',
    deprecated_by: null, stars: 342, installs: 12840, indexed_at: '2026-02-15T10:00:00Z', updated_at: '2026-03-18T14:30:00Z',
  },
  {
    id: 'pkg_02', path: 'github.com/ship-dev/ship-standard', name: 'Ship Standard',
    description: 'The default Ship agent configuration. Balanced permissions, common skills, production-ready defaults.',
    scope: 'official', repo_url: 'https://github.com/ship-dev/ship-standard', default_branch: 'main',
    latest_version: '1.0.0', content_hash: 'def456', source_type: 'native', claimed_by: 'u_ship',
    deprecated_by: null, stars: 218, installs: 8920, indexed_at: '2026-01-10T08:00:00Z', updated_at: '2026-03-17T09:15:00Z',
  },
  {
    id: 'pkg_03', path: 'github.com/vercel/next-agents', name: 'Next.js Agent Config',
    description: 'Agent configuration tuned for Next.js development — App Router patterns, RSC conventions, and Vercel deployment.',
    scope: 'community', repo_url: 'https://github.com/vercel/next-agents', default_branch: 'main',
    latest_version: '0.2.1', content_hash: 'ghi789', source_type: 'native', claimed_by: 'u_vercel',
    deprecated_by: null, stars: 156, installs: 5430, indexed_at: '2026-02-20T12:00:00Z', updated_at: '2026-03-16T11:00:00Z',
  },
  {
    id: 'pkg_04', path: 'github.com/rust-lang/rust-agents', name: 'Rust Development Agents',
    description: 'Agent skills for Rust — cargo workflows, borrow checker guidance, unsafe auditing, and clippy integration.',
    scope: 'community', repo_url: 'https://github.com/rust-lang/rust-agents', default_branch: 'main',
    latest_version: '0.3.0', content_hash: 'jkl012', source_type: 'native', claimed_by: null,
    deprecated_by: null, stars: 89, installs: 3210, indexed_at: '2026-03-01T15:00:00Z', updated_at: '2026-03-15T16:45:00Z',
  },
  {
    id: 'pkg_05', path: 'unofficial/tailwindlabs-tailwindcss', name: 'Tailwind CSS Agents',
    description: 'Extracted agent configuration from Tailwind CSS repository. Utility-first CSS patterns and plugin development.',
    scope: 'unofficial', repo_url: 'https://github.com/tailwindlabs/tailwindcss', default_branch: 'main',
    latest_version: '0.1.0', content_hash: 'mno345', source_type: 'imported', claimed_by: null,
    deprecated_by: null, stars: 45, installs: 1890, indexed_at: '2026-03-05T09:00:00Z', updated_at: '2026-03-14T08:30:00Z',
  },
  {
    id: 'pkg_06', path: 'github.com/denoland/deno-agents', name: 'Deno Development',
    description: 'Agent configuration for Deno projects — TypeScript-first, permissions model, and Deno Deploy patterns.',
    scope: 'community', repo_url: 'https://github.com/denoland/deno-agents', default_branch: 'main',
    latest_version: '0.1.3', content_hash: 'pqr678', source_type: 'native', claimed_by: null,
    deprecated_by: null, stars: 67, installs: 2140, indexed_at: '2026-03-08T11:00:00Z', updated_at: '2026-03-13T10:20:00Z',
  },
  {
    id: 'pkg_07', path: 'unofficial/facebook-react', name: 'React Development Agents',
    description: 'Extracted from Facebook React repository. Component patterns, hooks conventions, and concurrent rendering guidance.',
    scope: 'unofficial', repo_url: 'https://github.com/facebook/react', default_branch: 'main',
    latest_version: '0.1.0', content_hash: 'stu901', source_type: 'imported', claimed_by: null,
    deprecated_by: null, stars: 78, installs: 4560, indexed_at: '2026-03-02T14:00:00Z', updated_at: '2026-03-12T15:10:00Z',
  },
  {
    id: 'pkg_08', path: 'github.com/ship-dev/ship-autonomous', name: 'Ship Autonomous',
    description: 'Zero-interruption agent configuration — dontAsk mode, scoped deny rules, full tool access within scope.',
    scope: 'official', repo_url: 'https://github.com/ship-dev/ship-autonomous', default_branch: 'main',
    latest_version: '1.0.0', content_hash: 'vwx234', source_type: 'native', claimed_by: 'u_ship',
    deprecated_by: null, stars: 94, installs: 3780, indexed_at: '2026-01-10T08:00:00Z', updated_at: '2026-03-11T09:00:00Z',
  },
  {
    id: 'pkg_09', path: 'unofficial/sveltejs-svelte', name: 'Svelte Agent Config',
    description: 'Extracted from SvelteJS repository. Svelte 5 runes, component patterns, and SvelteKit conventions.',
    scope: 'unofficial', repo_url: 'https://github.com/sveltejs/svelte', default_branch: 'main',
    latest_version: '0.1.0', content_hash: 'yza567', source_type: 'imported', claimed_by: null,
    deprecated_by: null, stars: 34, installs: 980, indexed_at: '2026-03-06T10:00:00Z', updated_at: '2026-03-10T12:00:00Z',
  },
  {
    id: 'pkg_10', path: 'github.com/oven-sh/bun-agents', name: 'Bun Development',
    description: 'Agent configuration for Bun runtime — fast bundling, native TypeScript, and Bun-specific APIs.',
    scope: 'community', repo_url: 'https://github.com/oven-sh/bun-agents', default_branch: 'main',
    latest_version: '0.2.0', content_hash: 'bcd890', source_type: 'native', claimed_by: null,
    deprecated_by: null, stars: 52, installs: 1650, indexed_at: '2026-03-10T13:00:00Z', updated_at: '2026-03-18T07:00:00Z',
  },
  {
    id: 'pkg_11', path: 'github.com/golang/go-agents', name: 'Go Development Agents',
    description: 'Agent skills for Go — module management, error handling patterns, concurrency best practices, and go vet integration.',
    scope: 'community', repo_url: 'https://github.com/golang/go-agents', default_branch: 'main',
    latest_version: '0.1.2', content_hash: 'efg123', source_type: 'native', claimed_by: null,
    deprecated_by: null, stars: 73, installs: 2890, indexed_at: '2026-02-28T09:00:00Z', updated_at: '2026-03-09T14:00:00Z',
  },
  {
    id: 'pkg_12', path: 'unofficial/microsoft-vscode', name: 'VS Code Extension Agents',
    description: 'Extracted from VS Code repository. Extension development patterns, API conventions, and testing guidelines.',
    scope: 'unofficial', repo_url: 'https://github.com/microsoft/vscode', default_branch: 'main',
    latest_version: '0.1.0', content_hash: 'hij456', source_type: 'imported', claimed_by: null,
    deprecated_by: null, stars: 41, installs: 1230, indexed_at: '2026-03-04T16:00:00Z', updated_at: '2026-03-08T11:30:00Z',
  },
]

/** Mock versions for the detail page. */
export const MOCK_VERSIONS: Record<string, PackageVersion[]> = {
  'pkg_01': [
    { id: 'v_01a', package_id: 'pkg_01', version: '0.4.2', git_tag: 'v0.4.2', commit_sha: 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0', skills: ['commit-conventions', 'code-review'], agents: ['claude'], indexed_at: '2026-03-18T14:30:00Z' },
    { id: 'v_01b', package_id: 'pkg_01', version: '0.4.1', git_tag: 'v0.4.1', commit_sha: 'b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1', skills: ['commit-conventions', 'code-review'], agents: ['claude'], indexed_at: '2026-03-10T10:00:00Z' },
    { id: 'v_01c', package_id: 'pkg_01', version: '0.4.0', git_tag: 'v0.4.0', commit_sha: 'c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2', skills: ['commit-conventions'], agents: ['claude'], indexed_at: '2026-02-28T08:00:00Z' },
    { id: 'v_01d', package_id: 'pkg_01', version: '0.3.0', git_tag: 'v0.3.0', commit_sha: 'd4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3', skills: ['commit-conventions'], agents: ['claude'], indexed_at: '2026-02-15T10:00:00Z' },
  ],
}

/** Mock skills for the detail page. */
export const MOCK_SKILLS: Record<string, PackageSkill[]> = {
  'pkg_01': [
    { id: 'sk_01a', package_id: 'pkg_01', version_id: 'v_01a', skill_id: 'commit-conventions', name: 'Commit Conventions', description: 'Enforces conventional commit format with scope and type validation.', content_hash: 'sha256_cc_001', content_length: 2048 },
    { id: 'sk_01b', package_id: 'pkg_01', version_id: 'v_01a', skill_id: 'code-review', name: 'Code Review', description: 'Structured code review checklist with security, performance, and style checks.', content_hash: 'sha256_cr_001', content_length: 3420 },
  ],
}

/** Fallback versions for packages without specific mock data. */
export const DEFAULT_MOCK_VERSIONS: PackageVersion[] = [
  { id: 'v_default', package_id: '', version: '0.1.0', git_tag: 'v0.1.0', commit_sha: 'e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4', skills: [], agents: [], indexed_at: '2026-03-01T12:00:00Z' },
]

/** Fallback skills for packages without specific mock data. */
export const DEFAULT_MOCK_SKILLS: PackageSkill[] = [
  { id: 'sk_default', package_id: '', version_id: '', skill_id: 'default-skill', name: 'Default Skill', description: 'A placeholder skill from this package.', content_hash: 'sha256_default', content_length: 1024 },
]

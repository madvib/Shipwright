export const sessionKeys = {
  all: ['session'] as const,
  files: (wsId: string) => ['session', 'files', wsId] as const,
  fileContent: (wsId: string, path: string) => ['session', 'file', wsId, path] as const,
  diff: (wsId: string) => ['session', 'diff', wsId] as const,
  gitStatus: (wsId: string) => ['session', 'gitStatus', wsId] as const,
  gitDiff: (wsId: string, base?: string) => ['session', 'gitDiff', wsId, base ?? 'default'] as const,
  gitLog: (wsId: string, limit: number) => ['session', 'gitLog', wsId, limit] as const,
} as const

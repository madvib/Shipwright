export const sessionKeys = {
  all: ['session'] as const,
  files: () => ['session', 'files'] as const,
  fileContent: (path: string) => ['session', 'file', path] as const,
  diff: () => ['session', 'diff'] as const,
  gitStatus: () => ['session', 'gitStatus'] as const,
  gitDiff: (base?: string) => ['session', 'gitDiff', base ?? 'default'] as const,
  gitLog: (limit: number) => ['session', 'gitLog', limit] as const,
} as const

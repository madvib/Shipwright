export const sessionKeys = {
  all: ['session'] as const,
  files: () => ['session', 'files'] as const,
  fileContent: (path: string) => ['session', 'file', path] as const,
  diff: () => ['session', 'diff'] as const,
} as const

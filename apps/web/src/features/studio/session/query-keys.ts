export const sessionKeys = {
  all: ['session'] as const,
  files: () => ['session', 'files'] as const,
  fileContent: (path: string) => ['session', 'file', path] as const,
} as const

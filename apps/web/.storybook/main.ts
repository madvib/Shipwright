import type { StorybookConfig } from '@storybook/react-vite'
import { fileURLToPath, URL } from 'node:url'
import tsconfigPaths from 'vite-tsconfig-paths'
import tailwindcss from '@tailwindcss/vite'

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: ['@storybook/addon-essentials'],
  framework: '@storybook/react-vite',
  viteFinal: async (config) => {
    // Add the tsconfig paths plugin so #/* and other aliases resolve
    config.plugins ??= []
    config.plugins.push(
      tsconfigPaths({ projects: [fileURLToPath(new URL('../tsconfig.json', import.meta.url))] }),
      tailwindcss(),
    )

    // Inherit the project's explicit path aliases
    config.resolve ??= {}
    config.resolve.alias = {
      ...config.resolve.alias,
      '@/': fileURLToPath(new URL('../../../packages/primitives/src/', import.meta.url)),
      '@ship/ui': fileURLToPath(new URL('../../../packages/ui/src/index.ts', import.meta.url)),
      '@ship/primitives': fileURLToPath(new URL('../../../packages/primitives/src/index.tsx', import.meta.url)),
    }
    config.resolve.dedupe = ['react', 'react-dom', '@codemirror/state', '@codemirror/view']

    // Exclude the WASM compiler from optimization to avoid build errors
    config.optimizeDeps ??= {}
    config.optimizeDeps.exclude = ['@ship/compiler']

    // Allow pnpm store for fonts and hoisted deps
    config.server ??= {}
    config.server.fs ??= {}
    config.server.fs.allow = [
      ...(config.server.fs.allow ?? []),
      fileURLToPath(new URL('../../../', import.meta.url)),
      '/home/dev/.local/share/pnpm/store',
    ]

    return config
  },
}

export default config

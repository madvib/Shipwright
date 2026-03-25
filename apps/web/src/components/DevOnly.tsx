import type { ReactNode } from 'react'

const isDev = import.meta.env.DEV

/**
 * Feature flags for dev-only UI. Add entries here to gate features.
 * In production builds, DevOnly renders nothing regardless of flags.
 */
const FLAGS: Record<string, boolean> = {
  'github-import': true,
  'registry-publish': true,
  'account-section': true,
}

interface DevOnlyProps {
  /** Optional feature flag name. If provided, must be enabled in FLAGS. */
  flag?: string
  /** Whether to show the visual DEV indicator. Default true. */
  indicator?: boolean
  children: ReactNode
}

/**
 * Renders children only in development builds.
 * Shows a visual "DEV" badge so developers can distinguish dev-only features.
 * In production, renders nothing (tree-shaken out by Vite).
 *
 * Mirrors the Rust `#[cfg(feature = "unstable")]` pattern from the CLI.
 */
export function DevOnly({ flag, indicator = true, children }: DevOnlyProps) {
  if (!isDev) return null
  if (flag && !FLAGS[flag]) return null

  if (!indicator) return <>{children}</>

  return (
    <div className="relative">
      <div className="absolute -top-1.5 -right-1.5 z-50 rounded bg-violet-500/90 px-1 py-px text-[7px] font-bold uppercase tracking-wider text-white leading-tight pointer-events-none">
        dev{flag ? `: ${flag}` : ''}
      </div>
      <div className="ring-1 ring-dashed ring-violet-500/30 rounded-lg">
        {children}
      </div>
    </div>
  )
}

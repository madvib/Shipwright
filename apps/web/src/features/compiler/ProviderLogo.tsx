// Shared provider logo component — handles dark/light mode per provider.
// cursor: two-image dark/light swap (CUBE_2D)
// codex: OpenAI mono with dark:invert
// claude, gemini: SVG (no invert needed — already themed)

const PROVIDER_COLOR: Record<string, string> = {
  claude: 'bg-amber-500/15 text-amber-700 dark:text-amber-400',
  gemini: 'bg-blue-500/15 text-blue-700 dark:text-blue-400',
  codex: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-400',
  cursor: 'bg-violet-500/15 text-violet-700 dark:text-violet-400',
}

const PROVIDER_SHORT: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
}

interface Props {
  provider: string
  size?: 'sm' | 'md' | 'lg'
  className?: string
}

export function ProviderLogo({ provider, size = 'sm', className = '' }: Props) {
  const cls = size === 'lg' ? 'size-5' : size === 'md' ? 'size-4' : 'size-3.5'

  if (provider === 'cursor') {
    return (
      <>
        <img src="/ide-logos/CUBE_2D_LIGHT.svg" alt="Cursor" className={`${cls} object-contain dark:hidden ${className}`} />
        <img src="/ide-logos/CUBE_2D_DARK.svg" alt="Cursor" className={`${cls} object-contain hidden dark:block ${className}`} />
      </>
    )
  }
  if (provider === 'codex') {
    return <img src="/provider-logos/OpenAI-black-monoblossom.svg" alt="Codex" className={`${cls} object-contain dark:invert ${className}`} />
  }
  if (provider === 'claude') {
    return <img src="/provider-logos/claude.svg" alt="Claude" className={`${cls} object-contain ${className}`} />
  }
  if (provider === 'gemini') {
    return <img src="/provider-logos/googlegemini.svg" alt="Gemini" className={`${cls} object-contain ${className}`} />
  }

  // Fallback: colored initial badge
  const color = PROVIDER_COLOR[provider] ?? 'bg-muted text-muted-foreground'
  const short = PROVIDER_SHORT[provider] ?? provider
  return (
    <span className={`inline-flex items-center justify-center rounded text-[9px] font-bold px-1 ${color} ${className}`}>
      {short.slice(0, 2).toUpperCase()}
    </span>
  )
}

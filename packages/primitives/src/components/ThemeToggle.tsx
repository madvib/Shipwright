import { useEffect, useState } from 'react'
import { Sun, Moon } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Switch } from './switch'

type ThemeMode = 'light' | 'dark'

function getInitialMode(): ThemeMode {
  if (typeof window === 'undefined') return 'dark'
  const stored = window.localStorage.getItem('theme')
  if (stored === 'light' || stored === 'dark') return stored
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(mode: ThemeMode) {
  document.documentElement.classList.remove('light', 'dark')
  document.documentElement.classList.add(mode)
  document.documentElement.setAttribute('data-theme', mode)
  document.documentElement.style.colorScheme = mode
  window.localStorage.setItem('theme', mode)
}

function useThemeMode() {
  const [mode, setMode] = useState<ThemeMode>('dark')

  useEffect(() => {
    const initial = getInitialMode()
    setMode(initial)
    applyTheme(initial)
  }, [])

  const toggle = () => {
    setMode((prev) => {
      const next = prev === 'dark' ? 'light' : 'dark'
      applyTheme(next)
      return next
    })
  }

  const set = (next: ThemeMode) => {
    setMode(next)
    applyTheme(next)
  }

  return { mode, toggle, set }
}

interface ThemeToggleProps {
  /** "switch" renders the full segmented control; "icon" renders a compact button. */
  variant?: 'switch' | 'icon'
  className?: string
}

export function ThemeToggle({ variant = 'switch', className }: ThemeToggleProps) {
  const { mode, toggle, set } = useThemeMode()

  if (variant === 'icon') {
    return (
      <button
        onClick={toggle}
        className={cn(
          'flex items-center justify-center size-8 rounded-md border border-border/60 bg-card text-muted-foreground transition hover:text-foreground hover:border-border',
          className,
        )}
        title={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
      >
        {mode === 'dark' ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
      </button>
    )
  }

  return (
    <div className={cn('flex items-center gap-1 rounded-full border bg-muted/20 p-1', className)}>
      <div
        className={cn(
          'flex cursor-pointer items-center gap-1.5 rounded-full px-2 py-1 transition-all',
          mode === 'light' ? 'bg-background shadow-sm text-foreground' : 'text-muted-foreground'
        )}
        onClick={() => set('light')}
      >
        <Sun className="size-3.5" />
        <span className="text-[10px] font-bold uppercase tracking-tighter">Light</span>
      </div>
      <Switch
        checked={mode === 'dark'}
        onCheckedChange={(checked) => set(checked ? 'dark' : 'light')}
      />
      <div
        className={cn(
          'flex cursor-pointer items-center gap-1.5 rounded-full px-2 py-1 transition-all',
          mode === 'dark' ? 'bg-background shadow-sm text-foreground' : 'text-muted-foreground'
        )}
        onClick={() => set('dark')}
      >
        <Moon className="size-3.5" />
        <span className="text-[10px] font-bold uppercase tracking-tighter">Dark</span>
      </div>
    </div>
  )
}

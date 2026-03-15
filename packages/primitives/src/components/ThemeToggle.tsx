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

export function ThemeToggle() {
  const [mode, setMode] = useState<ThemeMode>('dark')

  useEffect(() => {
    const initial = getInitialMode()
    setMode(initial)
    applyTheme(initial)
  }, [])

  const set = (next: ThemeMode) => {
    setMode(next)
    applyTheme(next)
  }

  return (
    <div className="flex items-center gap-1 rounded-full border bg-muted/20 p-1">
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

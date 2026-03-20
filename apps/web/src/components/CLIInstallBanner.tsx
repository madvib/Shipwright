import { useState } from 'react'
import { Terminal, Check, Copy } from 'lucide-react'

const INSTALL_COMMAND = 'curl -fsSL https://getship.dev/install | sh'

interface CLIInstallBannerProps {
  className?: string
}

export function CLIInstallBanner({ className = '' }: CLIInstallBannerProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    void navigator.clipboard.writeText(INSTALL_COMMAND).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <div
      className={`flex items-center gap-3 rounded-xl border border-border/60 bg-card p-3.5 ${className}`}
    >
      <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-emerald-500/10">
        <Terminal className="size-4 text-emerald-500" />
      </div>

      <div className="min-w-0 flex-1">
        <div className="text-xs font-medium text-muted-foreground">
          Install the CLI for local compilation
        </div>
        <code className="mt-1 inline-block rounded bg-emerald-500/10 px-2 py-0.5 font-mono text-[11px] text-emerald-600 dark:text-emerald-400">
          {INSTALL_COMMAND}
        </code>
      </div>

      <button
        onClick={handleCopy}
        className="shrink-0 rounded-md border border-border/60 px-2.5 py-1 text-[10px] font-medium text-muted-foreground transition hover:border-emerald-500/50 hover:text-emerald-600 dark:hover:text-emerald-400"
      >
        {copied ? (
          <span className="flex items-center gap-1">
            <Check className="size-3" />
            Copied
          </span>
        ) : (
          <span className="flex items-center gap-1">
            <Copy className="size-3" />
            Copy
          </span>
        )}
      </button>
    </div>
  )
}

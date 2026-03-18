import { Loader2 } from 'lucide-react'

interface LoadingSpinnerProps {
  label?: string
  className?: string
}

export function LoadingSpinner({ label = 'Loading...', className = '' }: LoadingSpinnerProps) {
  return (
    <div className={`flex flex-col items-center justify-center gap-3 py-16 px-6 ${className}`}>
      <Loader2 className="size-6 animate-spin text-muted-foreground" />
      <p className="text-xs text-muted-foreground">{label}</p>
    </div>
  )
}

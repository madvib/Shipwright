import * as React from "react"

import { cn } from "@/lib/utils"

interface ProgressProps extends React.ComponentProps<"div"> {
  value?: number
  max?: number
  indicatorClassName?: string
}

function Progress({
  className,
  value = 0,
  max = 100,
  indicatorClassName,
  ...props
}: ProgressProps) {
  const safeMax = max > 0 ? max : 100
  const clamped = Math.max(0, Math.min(value, safeMax))
  const percent = (clamped / safeMax) * 100

  return (
    <div
      data-slot="progress"
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={safeMax}
      aria-valuenow={Math.round(clamped)}
      className={cn("bg-muted h-1.5 w-full overflow-hidden rounded-full", className)}
      {...props}
    >
      <div
        data-slot="progress-indicator"
        className={cn("bg-primary h-full rounded-full transition-all", indicatorClassName)}
        style={{ width: `${percent}%` }}
      />
    </div>
  )
}

export { Progress }

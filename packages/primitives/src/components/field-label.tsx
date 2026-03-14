import { cn } from '@/lib/utils'

interface FieldLabelProps {
  children: React.ReactNode
  className?: string
  as?: 'label' | 'span'
  htmlFor?: string
}

export function FieldLabel({
  children,
  className,
  as = 'label',
  htmlFor,
}: FieldLabelProps) {
  const Component = as
  return (
    <Component
      htmlFor={htmlFor}
      className={cn(
        'text-muted-foreground block text-xs font-medium uppercase tracking-wide',
        className
      )}
    >
      {children}
    </Component>
  )
}

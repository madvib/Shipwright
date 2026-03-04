import { ReactNode } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface EmptyStateProps {
  icon?: ReactNode
  title: string
  description?: string
  action?: ReactNode
  className?: string
}

export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <Card size="sm" className={cn('items-center text-center', className)}>
      <CardHeader>
        {icon && (
          <div className="text-muted-foreground mb-2 flex justify-center">{icon}</div>
        )}
        <CardTitle>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      {action && <CardContent>{action}</CardContent>}
    </Card>
  )
}

interface EmptyStateActionProps {
  children: ReactNode
  onClick?: () => void
  variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'link' | 'destructive'
  size?: 'default' | 'sm' | 'lg' | 'icon' | 'icon-sm' | 'icon-lg' | 'xs'
}

export function EmptyStateAction({
  children,
  onClick,
  variant = 'default',
  size = 'default',
}: EmptyStateActionProps) {
  return (
    <Button variant={variant} size={size} onClick={onClick}>
      {children}
    </Button>
  )
}

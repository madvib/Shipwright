import { useRef, useCallback } from 'react'

interface BentoCardProps {
  children: React.ReactNode
  className?: string
  glowColor?: string
  span?: string // grid span class, e.g. 'col-span-2 row-span-2'
  onClick?: () => void
}

export function BentoCard({ children, className = '', glowColor = '139, 92, 246', span = '', onClick }: BentoCardProps) {
  const ref = useRef<HTMLDivElement>(null)

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const el = ref.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    const x = ((e.clientX - rect.left) / rect.width) * 100
    const y = ((e.clientY - rect.top) / rect.height) * 100
    el.style.setProperty('--glow-x', `${x}%`)
    el.style.setProperty('--glow-y', `${y}%`)
    el.style.setProperty('--glow-opacity', '1')
  }, [])

  const handleMouseLeave = useCallback(() => {
    ref.current?.style.setProperty('--glow-opacity', '0')
  }, [])

  const Tag = onClick ? 'button' : 'div'

  return (
    <>
      <Tag
        ref={ref as never}
        className={`bento-card ${span} ${className}`}
        style={{ '--glow-rgb': glowColor } as React.CSSProperties}
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
        onClick={onClick}
      >
        {/* Border glow overlay */}
        <div className="bento-glow" aria-hidden />
        {children}
      </Tag>
    </>
  )
}

export function BentoGrid({ children, className = '' }: { children: React.ReactNode; className?: string }) {
  return (
    <>
      <style>{BENTO_CSS}</style>
      <div className={`bento-grid ${className}`}>{children}</div>
    </>
  )
}

const BENTO_CSS = `
  .bento-grid {
    display: grid;
    gap: 12px;
    grid-template-columns: repeat(3, 1fr);
    grid-auto-rows: minmax(0, 1fr);
    min-height: calc(100vh - 140px);
    padding-bottom: 72px; /* dock clearance */
  }

  @media (max-width: 768px) {
    .bento-grid {
      grid-template-columns: 1fr;
    }
  }

  .bento-card {
    position: relative;
    display: flex;
    flex-direction: column;
    padding: 20px;
    border-radius: 16px;
    border: 1px solid hsl(var(--border) / 0.5);
    background: hsl(var(--card));
    overflow: hidden;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
    transition: border-color 0.25s, transform 0.25s, box-shadow 0.25s;

    --glow-x: 50%;
    --glow-y: 50%;
    --glow-opacity: 0;
    --glow-rgb: 139, 92, 246;
  }

  .bento-card:hover {
    transform: translateY(-2px);
    border-color: rgba(var(--glow-rgb), 0.3);
    box-shadow: 0 8px 30px rgba(var(--glow-rgb), 0.06);
  }

  .bento-glow {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    pointer-events: none;
    opacity: var(--glow-opacity);
    transition: opacity 0.3s;
    background: radial-gradient(
      300px circle at var(--glow-x) var(--glow-y),
      rgba(var(--glow-rgb), 0.08) 0%,
      rgba(var(--glow-rgb), 0.03) 40%,
      transparent 70%
    );
    z-index: 0;
  }

  /* Border glow mask */
  .bento-card::after {
    content: '';
    position: absolute;
    inset: 0;
    padding: 1px;
    border-radius: inherit;
    background: radial-gradient(
      250px circle at var(--glow-x) var(--glow-y),
      rgba(var(--glow-rgb), calc(var(--glow-opacity) * 0.5)) 0%,
      transparent 60%
    );
    -webkit-mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    mask-composite: exclude;
    pointer-events: none;
    z-index: 1;
  }

  .bento-card > *:not(.bento-glow) {
    position: relative;
    z-index: 2;
  }
`

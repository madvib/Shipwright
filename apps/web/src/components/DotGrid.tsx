import { useRef, useEffect, useCallback } from 'react'

interface DotGridProps {
  className?: string
  dotSize?: number
  gap?: number
  color?: string
  radius?: number
}

export function DotGrid({
  className = '',
  dotSize = 1.5,
  gap = 28,
  color = 'hsl(var(--primary))',
  radius = 180,
}: DotGridProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const mouseRef = useRef({ x: -1000, y: -1000 })
  const rafRef = useRef<number>(0)
  const dotsRef = useRef<{ x: number; y: number }[]>([])

  const buildDots = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const { width, height } = canvas.getBoundingClientRect()
    canvas.width = width * devicePixelRatio
    canvas.height = height * devicePixelRatio

    const dots: { x: number; y: number }[] = []
    const cols = Math.ceil(width / gap) + 1
    const rows = Math.ceil(height / gap) + 1
    const offsetX = (width - (cols - 1) * gap) / 2
    const offsetY = (height - (rows - 1) * gap) / 2

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        dots.push({ x: offsetX + c * gap, y: offsetY + r * gap })
      }
    }
    dotsRef.current = dots
  }, [gap])

  const draw = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const dpr = devicePixelRatio

    ctx.clearRect(0, 0, canvas.width, canvas.height)
    ctx.scale(dpr, dpr)

    const mx = mouseRef.current.x
    const my = mouseRef.current.y

    // Parse color once — we'll vary opacity
    const baseAlpha = 0.15

    for (const dot of dotsRef.current) {
      const dx = dot.x - mx
      const dy = dot.y - my
      const dist = Math.sqrt(dx * dx + dy * dy)
      const proximity = Math.max(0, 1 - dist / radius)

      const alpha = baseAlpha + proximity * 0.7
      const size = dotSize + proximity * 2.5

      ctx.beginPath()
      ctx.arc(dot.x, dot.y, size, 0, Math.PI * 2)
      ctx.fillStyle = color.replace(')', ` / ${alpha})`)
      ctx.fill()
    }

    ctx.setTransform(1, 0, 0, 1, 0, 0)
    rafRef.current = requestAnimationFrame(draw)
  }, [color, dotSize, radius])

  useEffect(() => {
    buildDots()
    rafRef.current = requestAnimationFrame(draw)

    const handleResize = () => {
      buildDots()
    }
    window.addEventListener('resize', handleResize)

    return () => {
      cancelAnimationFrame(rafRef.current)
      window.removeEventListener('resize', handleResize)
    }
  }, [buildDots, draw])

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect()
    if (!rect) return
    mouseRef.current = { x: e.clientX - rect.left, y: e.clientY - rect.top }
  }, [])

  const handleMouseLeave = useCallback(() => {
    mouseRef.current = { x: -1000, y: -1000 }
  }, [])

  return (
    <canvas
      ref={canvasRef}
      className={`absolute inset-0 pointer-events-auto ${className}`}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      style={{ width: '100%', height: '100%' }}
    />
  )
}

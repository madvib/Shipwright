import { useRef, useEffect, useCallback } from 'react'

interface DotGridProps {
  className?: string
  dotSize?: number
  gap?: number
  radius?: number
}

/** Resolve a CSS custom property to an RGB triplet the canvas API can use.
 *  --primary uses oklch() which cannot be parsed via hsl() wrapping.
 *  Fall back to the brand amber/orange that matches the primary token. */
function resolveColor(_el: HTMLElement): [number, number, number] {
  // amber-500 equivalent: #f59e0b → rgb(245, 158, 11)
  return [245, 158, 11]
}

export function DotGrid({
  className = '',
  dotSize = 1.5,
  gap = 28,
  radius = 180,
}: DotGridProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const mouseRef = useRef({ x: -1000, y: -1000 })
  const rafRef = useRef<number>(0)
  const dotsRef = useRef<{ x: number; y: number }[]>([])
  const colorRef = useRef<[number, number, number]>([128, 128, 128])

  const buildDots = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas?.parentElement) return
    // Measure the parent — not the canvas — to avoid feedback loops
    const { width, height } = canvas.parentElement.getBoundingClientRect()
    canvas.width = width * devicePixelRatio
    canvas.height = height * devicePixelRatio

    // Resolve theme color
    colorRef.current = resolveColor(canvas)

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
    const [cr, cg, cb] = colorRef.current

    ctx.clearRect(0, 0, canvas.width, canvas.height)
    ctx.scale(dpr, dpr)

    const mx = mouseRef.current.x
    const my = mouseRef.current.y
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
      ctx.fillStyle = `rgba(${cr},${cg},${cb},${alpha})`
      ctx.fill()
    }

    ctx.setTransform(1, 0, 0, 1, 0, 0)
    rafRef.current = requestAnimationFrame(draw)
  }, [dotSize, radius])

  useEffect(() => {
    buildDots()
    rafRef.current = requestAnimationFrame(draw)

    // Track mouse at window level so the canvas works even behind other content
    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvasRef.current?.parentElement?.getBoundingClientRect()
      if (!rect) return
      mouseRef.current = { x: e.clientX - rect.left, y: e.clientY - rect.top }
    }

    const handleResize = () => buildDots()

    // Re-resolve color when theme changes (class mutation on <html>)
    const observer = new MutationObserver(() => {
      if (canvasRef.current) colorRef.current = resolveColor(canvasRef.current)
    })
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })

    window.addEventListener('mousemove', handleMouseMove, { passive: true })
    window.addEventListener('resize', handleResize)

    return () => {
      cancelAnimationFrame(rafRef.current)
      observer.disconnect()
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('resize', handleResize)
    }
  }, [buildDots, draw])

  return (
    <canvas
      ref={canvasRef}
      className={`absolute inset-0 size-full pointer-events-none ${className}`}
    />
  )
}

import { useState, useCallback, useRef, useEffect } from 'react'
import { SECTION_DEFS } from './AgentActivityBar'

export function useScrollspy() {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [activeSection, setActiveSection] = useState<string>(SECTION_DEFS[0].id)
  const isScrollingRef = useRef(false)

  useEffect(() => {
    const container = scrollRef.current
    if (!container) return

    const sectionIds = SECTION_DEFS.map((s) => `section-${s.id}`)
    const observer = new IntersectionObserver(
      (entries) => {
        if (isScrollingRef.current) return
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const sectionId = entry.target.id.replace('section-', '')
            setActiveSection(sectionId)
            break
          }
        }
      },
      { root: container, rootMargin: '-10% 0px -80% 0px', threshold: 0 },
    )

    for (const id of sectionIds) {
      const el = container.querySelector(`#${id}`)
      if (el) observer.observe(el)
    }

    return () => observer.disconnect()
  }, [])

  const handleSectionClick = useCallback((sectionId: string) => {
    const container = scrollRef.current
    if (!container) return
    const el = container.querySelector(`#section-${sectionId}`)
    if (!el) return

    isScrollingRef.current = true
    setActiveSection(sectionId)
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    setTimeout(() => { isScrollingRef.current = false }, 600)
  }, [])

  return { scrollRef, activeSection, handleSectionClick }
}

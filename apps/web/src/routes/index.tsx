import { createFileRoute } from '@tanstack/react-router'
import { lazy, Suspense } from 'react'
import { LandingHero } from '../components/landing/LandingHero'
import { ProductShowcase } from '../components/landing/ProductShowcase'
import { AnimatedShowcase } from '../components/landing/AnimatedShowcase'
import { FeatureGrid } from '../components/landing/FeatureGrid'
import { HowItWorks } from '../components/landing/HowItWorks'
import { LandingCta } from '../components/landing/LandingCta'
import { LandingFooter } from '../components/landing/LandingFooter'

const LandingNav = lazy(() => import('../components/landing/LandingNav'))

export const Route = createFileRoute('/')({ ssr: false, component: LandingPage })

function LandingPage() {
  return (
    <main className="min-h-screen">
      <Suspense fallback={<div className="h-14" />}>
        <LandingNav />
      </Suspense>
      <LandingHero />
      <AnimatedShowcase />
      <ProductShowcase />
      <FeatureGrid />
      <HowItWorks />
      <LandingCta />
      <LandingFooter />
    </main>
  )
}

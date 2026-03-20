import { createFileRoute } from '@tanstack/react-router'
import { LandingNav } from '../components/landing/LandingNav'
import { LandingHero } from '../components/landing/LandingHero'
import { ProductShowcase } from '../components/landing/ProductShowcase'
import { FeatureGrid } from '../components/landing/FeatureGrid'
import { HowItWorks } from '../components/landing/HowItWorks'
import { LandingCta } from '../components/landing/LandingCta'
import { LandingFooter } from '../components/landing/LandingFooter'

export const Route = createFileRoute('/')({ component: LandingPage })

function LandingPage() {
  return (
    <main className="min-h-screen">
      <LandingNav />
      <LandingHero />
      <ProductShowcase />
      <FeatureGrid />
      <HowItWorks />
      <LandingCta />
      <LandingFooter />
    </main>
  )
}

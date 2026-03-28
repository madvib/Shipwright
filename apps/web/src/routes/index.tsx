import { createFileRoute } from '@tanstack/react-router'
import LandingNav from '../components/landing/LandingNav'
import { LandingHero } from '../components/landing/LandingHero'
import { AnimatedShowcase } from '../components/landing/AnimatedShowcase'
import { FeatureGrid } from '../components/landing/FeatureGrid'
import { HowItWorks } from '../components/landing/HowItWorks'
import { LandingCta } from '../components/landing/LandingCta'
import { LandingFooter } from '../components/landing/LandingFooter'

export const Route = createFileRoute('/')({ ssr: false, component: LandingPage })

function LandingPage() {
  return (
    <main className="min-h-screen">
      <LandingNav />
      <LandingHero />
      <AnimatedShowcase />
      <FeatureGrid />
      <HowItWorks />
      <LandingCta />
      <LandingFooter />
    </main>
  )
}

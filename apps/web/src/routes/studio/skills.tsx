import { createFileRoute, Link } from '@tanstack/react-router'
import { BookOpen } from 'lucide-react'
import { useLibrary } from '#/features/compiler/useLibrary'
import { SkillsForm } from '#/features/compiler/sections/SkillsForm'
import { EmptyState } from '#/components/EmptyState'

export const Route = createFileRoute('/studio/skills')({ component: SkillsPage })

function SkillsPage() {
  const { library, updateLibrary } = useLibrary()

  const skills = library.skills ?? []

  if (skills.length === 0) {
    return (
      <div className="h-full flex flex-col">
        <EmptyState
          icon={<BookOpen className="size-5" />}
          title="No skills yet"
          description="Skills give your agent specific capabilities — commit conventions, code review, testing patterns."
          action={
            <Link
              to="/studio/registry"
              className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90 no-underline"
            >
              Browse the registry
            </Link>
          }
        />
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      <SkillsForm
        skills={skills}
        onChange={(updated) => updateLibrary({ skills: updated })}
      />
    </div>
  )
}

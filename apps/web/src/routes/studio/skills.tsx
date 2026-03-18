import { createFileRoute } from '@tanstack/react-router'
import { useLibrary } from '#/features/compiler/useLibrary'
import { SkillsForm } from '#/features/compiler/sections/SkillsForm'

export const Route = createFileRoute('/studio/skills')({ component: SkillsPage })

function SkillsPage() {
  const { library, updateLibrary } = useLibrary()

  return (
    <div className="h-full flex flex-col">
      <SkillsForm
        skills={library.skills}
        onChange={(skills) => updateLibrary({ skills })}
      />
    </div>
  )
}

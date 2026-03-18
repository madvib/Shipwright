import { X, Check } from 'lucide-react'

const DISMISSED_KEY = 'ship-tutorial-dismissed'

export function isTutorialDismissed() {
  try { return localStorage.getItem(DISMISSED_KEY) === '1' } catch { return false }
}

export function dismissTutorial() {
  try { localStorage.setItem(DISMISSED_KEY, '1') } catch { /* ignore */ }
}

const STEPS = [
  { label: 'Create profile' },
  { label: 'Add skills + MCP' },
  { label: 'Wire workflow' },
  { label: 'Export' },
]

interface TutorialStepperProps {
  currentStep: number // 0-based index of active step
  onDismiss: () => void
}

export function TutorialStepper({ currentStep, onDismiss }: TutorialStepperProps) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-border/40 bg-muted/20 px-4 py-2.5 mb-6">
      <div className="flex items-center gap-1">
        {STEPS.map((step, i) => {
          const done = i < currentStep
          const active = i === currentStep
          return (
            <div key={step.label} className="flex items-center gap-1">
              {i > 0 && <div className="w-6 h-px bg-border/60 mx-0.5" />}
              <div className={`flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[10px] font-medium transition ${
                active
                  ? 'bg-primary/10 text-primary'
                  : done
                  ? 'text-muted-foreground'
                  : 'text-muted-foreground/50'
              }`}>
                <span className={`flex size-3.5 items-center justify-center rounded-full text-[10px] font-bold shrink-0 ${
                  done
                    ? 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400'
                    : active
                    ? 'bg-primary/20 text-primary'
                    : 'bg-muted text-muted-foreground/50'
                }`}>
                  {done ? <Check className="size-2" /> : i + 1}
                </span>
                <span className={done ? 'line-through opacity-50' : ''}>{step.label}</span>
              </div>
            </div>
          )
        })}
      </div>
      <button
        onClick={onDismiss}
        className="ml-4 shrink-0 rounded p-0.5 text-muted-foreground/50 hover:text-muted-foreground transition"
        title="Dismiss tutorial"
      >
        <X className="size-3" />
      </button>
    </div>
  )
}

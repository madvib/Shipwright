import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { Spinner } from '@ship/primitives'
import { fetchApi } from '#/lib/api-errors'

interface PublishDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

interface PublishResponse {
  package_id: string
  version: string
  skills_indexed: number
}

export function PublishDialog({ open, onOpenChange }: PublishDialogProps) {
  const [repoUrl, setRepoUrl] = useState('')

  const mutation = useMutation({
    mutationFn: (repo_url: string) =>
      fetchApi<PublishResponse>('/api/registry/publish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ repo_url }),
      }),
    onSuccess: () => {
      // Keep dialog open to show success state
    },
  })

  const handlePublish = () => {
    if (!repoUrl.trim()) return
    mutation.mutate(repoUrl.trim())
  }

  const handleClose = (next: boolean) => {
    if (next) return
    onOpenChange(false)
    // Reset state after closing animation
    setTimeout(() => {
      setRepoUrl('')
      mutation.reset()
    }, 200)
  }

  const errorMessage = mutation.error
    ? ('message' in (mutation.error as object) ? (mutation.error as { message: string }).message : 'Publish failed')
    : null

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Publish to registry</DialogTitle>
          <DialogDescription>
            Publish a Ship package from a GitHub repository. The repo must contain a <code className="text-xs bg-muted px-1 py-0.5 rounded">.ship/ship.toml</code> with <code className="text-xs bg-muted px-1 py-0.5 rounded">[module]</code> and <code className="text-xs bg-muted px-1 py-0.5 rounded">[exports]</code> sections.
          </DialogDescription>
        </DialogHeader>

        {mutation.isSuccess ? (
          <SuccessView
            packageId={mutation.data.package_id}
            version={mutation.data.version}
            skillsIndexed={mutation.data.skills_indexed}
            onClose={() => handleClose(false)}
          />
        ) : (
          <>
            <div className="space-y-4 py-2">
              <div>
                <label className="text-xs font-medium text-foreground mb-1.5 block">
                  GitHub repository URL
                </label>
                <Input
                  value={repoUrl}
                  onChange={(e) => setRepoUrl(e.target.value)}
                  placeholder="https://github.com/user/repo"
                  autoFocus
                  disabled={mutation.isPending}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handlePublish()
                  }}
                />
              </div>

              {errorMessage && (
                <div className="rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2">
                  <p className="text-xs text-destructive">{errorMessage}</p>
                </div>
              )}
            </div>

            <DialogFooter>
              <Button variant="ghost" onClick={() => handleClose(false)} disabled={mutation.isPending}>
                Cancel
              </Button>
              <Button
                onClick={handlePublish}
                disabled={!repoUrl.trim() || mutation.isPending}
              >
                {mutation.isPending ? (
                  <span className="inline-flex items-center gap-2">
                    <Spinner size="sm" className="text-current" />
                    Publishing...
                  </span>
                ) : (
                  'Publish'
                )}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

function SuccessView({ packageId, version, skillsIndexed, onClose }: {
  packageId: string
  version: string
  skillsIndexed: number
  onClose: () => void
}) {
  return (
    <>
      <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-4 space-y-3">
        <div className="flex items-center gap-2">
          <span className="size-2 rounded-full bg-emerald-500" />
          <span className="text-sm font-medium text-emerald-600 dark:text-emerald-400">
            Published successfully
          </span>
        </div>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
          <dt className="text-muted-foreground">Package</dt>
          <dd className="font-mono text-foreground">{packageId}</dd>
          <dt className="text-muted-foreground">Version</dt>
          <dd className="font-mono text-foreground">{version}</dd>
          <dt className="text-muted-foreground">Skills indexed</dt>
          <dd className="font-mono text-foreground">{skillsIndexed}</dd>
        </dl>
      </div>
      <DialogFooter>
        <Button onClick={onClose}>Done</Button>
      </DialogFooter>
    </>
  )
}

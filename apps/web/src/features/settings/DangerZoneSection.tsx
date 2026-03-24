import { useState } from 'react'
import { AlertTriangle } from 'lucide-react'
import {
  Button,
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from '@ship/primitives'
import { toast } from 'sonner'
import { useNavigate } from '@tanstack/react-router'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { clearAllDrafts } from '#/features/agents/useAgentDrafts'
import { SettingsSection, SettingsRow } from './SettingsLayout'

const AGENT_STORAGE_KEY = 'ship-agents-v2'
const SETTINGS_STORAGE_KEY = 'ship-settings-v1'
const LIBRARY_STORAGE_KEY = 'ship-studio-v1'

export function DangerZoneSection() {
  const { agents, deleteAgent } = useAgentStore()
  const navigate = useNavigate()
  const [deletingAgents, setDeletingAgents] = useState(false)
  const [deletingAccount, setDeletingAccount] = useState(false)

  const handleDeleteAllAgents = () => {
    setDeletingAgents(true)
    try {
      for (const agent of agents) {
        deleteAgent(agent.profile.id)
      }
      clearAllDrafts()
      window.localStorage.removeItem(AGENT_STORAGE_KEY)
      toast.success('All agents deleted')
    } catch {
      toast.error('Failed to delete agents')
    } finally {
      setDeletingAgents(false)
    }
  }

  const handleDeleteAccount = async () => {
    setDeletingAccount(true)
    try {
      const res = await fetch('/api/auth/delete-user', {
        method: 'POST',
        credentials: 'include',
      })
      if (!res.ok) throw new Error('delete failed')
      window.localStorage.removeItem(AGENT_STORAGE_KEY)
      window.localStorage.removeItem(SETTINGS_STORAGE_KEY)
      window.localStorage.removeItem(LIBRARY_STORAGE_KEY)
      toast.success('Account deleted')
      void navigate({ to: '/' })
    } catch {
      toast.error('Failed to delete account')
    } finally {
      setDeletingAccount(false)
    }
  }

  return (
    <SettingsSection icon={<AlertTriangle className="size-[15px]" />} title="Danger Zone" danger>
      <SettingsRow label="Delete all agents" sublabel="Permanently remove all agent configurations">
        <AlertDialog>
          <AlertDialogTrigger render={<Button variant="destructive" size="xs" />}>
            Delete all
          </AlertDialogTrigger>
          <AlertDialogContent size="sm">
            <AlertDialogHeader>
              <AlertDialogTitle>Delete all agents?</AlertDialogTitle>
              <AlertDialogDescription>
                This will permanently remove all agent configurations. This action cannot be undone.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
              <AlertDialogAction
                variant="destructive"
                size="sm"
                disabled={deletingAgents}
                onClick={handleDeleteAllAgents}
              >
                {deletingAgents ? 'Deleting...' : 'Delete all agents'}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </SettingsRow>
      <SettingsRow label="Delete account" sublabel="Remove your account and all data from Ship">
        <AlertDialog>
          <AlertDialogTrigger render={<Button variant="destructive" size="xs" />}>
            Delete account
          </AlertDialogTrigger>
          <AlertDialogContent size="sm">
            <AlertDialogHeader>
              <AlertDialogTitle>Delete your account?</AlertDialogTitle>
              <AlertDialogDescription>
                This will permanently delete your account, all agents, and all associated data. This action cannot be undone.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
              <AlertDialogAction
                variant="destructive"
                size="sm"
                disabled={deletingAccount}
                onClick={() => void handleDeleteAccount()}
              >
                {deletingAccount ? 'Deleting...' : 'Delete account'}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </SettingsRow>
    </SettingsSection>
  )
}

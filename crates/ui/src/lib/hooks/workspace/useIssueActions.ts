import { Dispatch, SetStateAction } from 'react';
import { Issue, IssueEntry } from '@/bindings';
import {
  createNewIssueCmd,
  deleteIssueByPathCmd,
  moveIssueStatusCmd,
  updateIssueByPathCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseIssueActionsParams {
  issues: IssueEntry[];
  setIssues: Dispatch<SetStateAction<IssueEntry[]>>;
  setSelectedIssue: Dispatch<SetStateAction<IssueEntry | null>>;
  setShowNewIssue: Dispatch<SetStateAction<boolean>>;
  setError: Dispatch<SetStateAction<string | null>>;
  refreshActivity: () => Promise<void>;
  refreshProjectInfo: () => Promise<void>;
}

export function useIssueActions({
  issues,
  setIssues,
  setSelectedIssue,
  setShowNewIssue,
  setError,
  refreshActivity,
  refreshProjectInfo,
}: UseIssueActionsParams) {
  const handleCreateIssue = async (title: string, description: string, status: string) => {
    if (!isTauriRuntime()) {
      setError('Issue creation is only available in Tauri runtime.');
      return;
    }

    try {
      const entry = await createNewIssueCmd(title, description, status);
      setIssues((prev) => [...prev, entry]);
      setShowNewIssue(false);
      await refreshActivity();
      await refreshProjectInfo();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleStatusChange = async (
    file_name: string,
    from: string,
    to: string,
    options?: { selectMovedIssue?: boolean }
  ) => {
    if (!isTauriRuntime()) {
      setError('Issue status changes are only available in Tauri runtime.');
      return;
    }

    try {
      const updated = await moveIssueStatusCmd(file_name, from, to);
      setIssues((prev) =>
        prev.map((entry) =>
          entry.file_name === file_name && entry.status === from
            ? updated
            : entry
        )
      );
      if (options?.selectMovedIssue ?? true) {
        setSelectedIssue(updated);
      }
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleSaveIssue = async (path: string, issue: Issue) => {
    if (!isTauriRuntime()) {
      setError('Saving issues is only available in Tauri runtime.');
      return;
    }

    const existing = issues.find((entry) => entry.path === path);
    if (!existing) return;

    const updated: IssueEntry = {
      ...existing,
      issue,
    };

    try {
      await updateIssueByPathCmd(path, updated.issue);
      setIssues((prev) => prev.map((entry) => (entry.path === path ? updated : entry)));
      setSelectedIssue(updated);
    } catch (error) {
      setError(String(error));
    }
  };

  const handleDeleteIssue = async (path: string) => {
    if (!isTauriRuntime()) {
      setError('Deleting issues is only available in Tauri runtime.');
      return;
    }

    try {
      await deleteIssueByPathCmd(path);
      setIssues((prev) => prev.filter((entry) => entry.path !== path));
      setSelectedIssue(null);
      await refreshActivity();
      await refreshProjectInfo();
    } catch (error) {
      setError(String(error));
    }
  };

  return {
    handleCreateIssue,
    handleStatusChange,
    handleSaveIssue,
    handleDeleteIssue,
  };
}

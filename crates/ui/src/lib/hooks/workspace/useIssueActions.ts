import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
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
  const handleCreateIssue = useCallback(async (
    title: string,
    description: string,
    status: string,
    options?: {
      assignee?: string | null;
      tags?: string[];
      spec?: string | null;
    }
  ) => {
    if (!isTauriRuntime()) {
      setError('Issue creation is only available in Tauri runtime.');
      return;
    }

    try {
      const created = await createNewIssueCmd(
        title,
        description,
        status,
        options?.assignee ?? null,
        options?.tags ?? []
      );
      let entry = created;
      if (options?.spec?.trim()) {
        const nextIssue: Issue = {
          ...created.issue,
          spec_id: options.spec.trim(),
        };
        await updateIssueByPathCmd(created.path, nextIssue);
        entry = { ...created, issue: nextIssue };
      }
      setIssues((prev) => [...prev, entry]);
      setShowNewIssue(false);
      await refreshActivity();
      await refreshProjectInfo();
    } catch (error) {
      setError(String(error));
    }
  }, [setIssues, setShowNewIssue, setError, refreshActivity, refreshProjectInfo]);

  const handleStatusChange = useCallback(async (
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
  }, [setIssues, setSelectedIssue, setError, refreshActivity]);

  const handleSaveIssue = useCallback(async (path: string, issue: Issue) => {
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
  }, [issues, setIssues, setSelectedIssue, setError]);

  const handleDeleteIssue = useCallback(async (path: string) => {
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
  }, [setIssues, setSelectedIssue, setError, refreshActivity, refreshProjectInfo]);

  return useMemo(() => ({
    handleCreateIssue,
    handleStatusChange,
    handleSaveIssue,
    handleDeleteIssue,
  }), [handleCreateIssue, handleStatusChange, handleSaveIssue, handleDeleteIssue]);
}

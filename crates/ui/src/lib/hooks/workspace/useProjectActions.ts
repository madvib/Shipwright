import { Dispatch, SetStateAction } from 'react';
import {
  AdrEntry,
  FeatureDocument,
  IssueEntry,
  ProjectDiscovery as Project,
  ProjectConfig,
  ReleaseDocument,
  StatusConfig,
} from '@/bindings';
import { SpecDocument } from '@/lib/types/spec';
import { CreateProjectInput } from '@/features/planning/ProjectOnboarding';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import {
  CreateProjectPayload,
  createNewProjectCmd,
  createProjectWithOptionsCmd,
  pickAndOpenProject,
  pickProjectDirectoryCmd,
  setActiveProjectCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';
import { projectFromInfo } from './constants';

interface UseProjectActionsParams {
  setError: Dispatch<SetStateAction<string | null>>;
  setActiveProject: Dispatch<SetStateAction<Project | null>>;
  setDetectedProject: Dispatch<SetStateAction<Project | null>>;
  setSelectedIssue: Dispatch<SetStateAction<IssueEntry | null>>;
  setSelectedAdr: Dispatch<SetStateAction<AdrEntry | null>>;
  setSelectedSpec: Dispatch<SetStateAction<SpecDocument | null>>;
  setSelectedRelease: Dispatch<SetStateAction<ReleaseDocument | null>>;
  setSelectedFeature: Dispatch<SetStateAction<FeatureDocument | null>>;
  setCreatingProject: Dispatch<SetStateAction<boolean>>;
  loadProjectData: () => Promise<void>;
  loadProjectConfig: () => Promise<void>;
}

export function useProjectActions({
  setError,
  setActiveProject,
  setDetectedProject,
  setSelectedIssue,
  setSelectedAdr,
  setSelectedSpec,
  setSelectedRelease,
  setSelectedFeature,
  setCreatingProject,
  loadProjectData,
  loadProjectConfig,
}: UseProjectActionsParams) {
  const resetSelection = () => {
    setSelectedIssue(null);
    setSelectedAdr(null);
    setSelectedSpec(null);
    setSelectedRelease(null);
    setSelectedFeature(null);
  };

  const activateProjectFromInfo = async (info: { name: string; path: string; issue_count?: number }) => {
    setActiveProject(projectFromInfo(info));
    setDetectedProject(null);
    resetSelection();
    await loadProjectData();
    await loadProjectConfig();
  };

  const handleOpenProject = async () => {
    if (!isTauriRuntime()) {
      setError('Project picker is only available in Tauri runtime.');
      return;
    }

    try {
      const info = await pickAndOpenProject();
      await activateProjectFromInfo(info);
    } catch (error) {
      if (String(error).includes('No directory selected')) return;
      setError(String(error));
    }
  };

  const handleNewProject = async () => {
    if (!isTauriRuntime()) {
      setError('Project creation is only available in Tauri runtime.');
      return;
    }

    try {
      const info = await createNewProjectCmd();
      await activateProjectFromInfo(info);
    } catch (error) {
      if (String(error).includes('No directory selected')) return;
      setError(String(error));
    }
  };

  const handlePickProjectDirectory = async (): Promise<string | null> => {
    if (!isTauriRuntime()) {
      setError('Directory picker is only available in Tauri runtime.');
      return null;
    }

    try {
      return await pickProjectDirectoryCmd();
    } catch (error) {
      setError(String(error));
      return null;
    }
  };

  const handleCreateProjectFromForm = async (input: CreateProjectInput) => {
    if (!isTauriRuntime()) {
      setError('Project creation is only available in Tauri runtime.');
      return;
    }

    const payload: CreateProjectPayload = {
      directory: input.directory,
      name: input.name,
      description: input.description,
    };

    if (!input.useDefaults) {
      const statuses = DEFAULT_STATUSES.filter((status: StatusConfig) =>
        input.selectedStatuses.includes(status.id)
      );
      payload.config = {
        version: '1',
        name: input.name,
        description: input.description ?? null,
        statuses,
        git: {
          ignore: ['issues'],
          commit: ['releases', 'features', 'specs', 'adrs', 'ship.toml', 'templates'],
        },
      } as ProjectConfig;
    }

    setCreatingProject(true);
    try {
      const info = await createProjectWithOptionsCmd(payload);
      await activateProjectFromInfo(info);
    } catch (error) {
      setError(String(error));
      throw error;
    } finally {
      setCreatingProject(false);
    }
  };

  const handleSelectProject = async (project: Project): Promise<boolean> => {
    if (!isTauriRuntime()) {
      setError('Project switching is only available in Tauri runtime.');
      return false;
    }

    try {
      const info = await setActiveProjectCmd(project.path);
      await activateProjectFromInfo(info);
      return true;
    } catch (error) {
      setError(String(error));
      return false;
    }
  };

  return {
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
  };
}

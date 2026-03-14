import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import {
  AdrEntry,
  FeatureDocument,
  ModeConfig,
  ProjectDiscovery as Project,
  ProjectConfig,
  ReleaseDocument,
  StatusConfig,
} from '@/bindings';
import { CreateProjectInput } from '@/features/planning/common/ProjectOnboarding';
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
  setSelectedAdr: Dispatch<SetStateAction<AdrEntry | null>>;
  setSelectedRelease: Dispatch<SetStateAction<ReleaseDocument | null>>;
  setSelectedFeature: Dispatch<SetStateAction<FeatureDocument | null>>;
  setCreatingProject: Dispatch<SetStateAction<boolean>>;
  loadProjectData: () => Promise<void>;
  loadProjectConfig: () => Promise<void>;
  refreshRecentProjects: () => Promise<void>;
}

export function useProjectActions({
  setError,
  setActiveProject,
  setDetectedProject,
  setSelectedAdr,
  setSelectedRelease,
  setSelectedFeature,
  setCreatingProject,
  loadProjectData,
  loadProjectConfig,
  refreshRecentProjects,
}: UseProjectActionsParams) {
  const resetSelection = useCallback(() => {
    setSelectedAdr(null);
    setSelectedRelease(null);
    setSelectedFeature(null);
  }, [setSelectedAdr, setSelectedRelease, setSelectedFeature]);

  const activateProjectFromInfo = useCallback(async (info: { name: string; path: string }) => {
    setActiveProject(projectFromInfo(info));
    setDetectedProject(null);
    resetSelection();
    await loadProjectData();
    await loadProjectConfig();
  }, [setActiveProject, setDetectedProject, resetSelection, loadProjectData, loadProjectConfig]);

  const handleOpenProject = useCallback(async () => {
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
  }, [setError, activateProjectFromInfo]);

  const handleNewProject = useCallback(async () => {
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
  }, [setError, activateProjectFromInfo]);

  const handlePickProjectDirectory = useCallback(async (): Promise<string | null> => {
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
  }, [setError]);

  const handleCreateProjectFromForm = useCallback(async (input: CreateProjectInput) => {
    if (!isTauriRuntime()) {
      setError('Project creation is only available in Tauri runtime.');
      return null;
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
      
      // Ensure ship.toml is ALWAYS committed
      const commitCategories = [...input.gitCommitCategories];
      if (!commitCategories.includes('ship.toml')) {
        commitCategories.push('ship.toml');
      }

      payload.config = {
        version: '1',
        name: input.name,
        description: input.description ?? null,
        statuses,
        git: {
          ignore: [] as string[],
          commit: commitCategories,
        },
        providers: input.enabledAgents,
        // Set active_mode to the first selected mode
        active_mode: input.selectedModes[0] || 'frontend-react',
        // Map selected mode IDs to full ModeConfig objects (partial is fine for creation)
        modes: input.selectedModes.map(id => ({ id, name: id } as ModeConfig)),
      } as ProjectConfig;
    }

    setCreatingProject(true);
    try {
      const info = await createProjectWithOptionsCmd(payload);
      await activateProjectFromInfo(info);
      await refreshRecentProjects();
      return info;
    } catch (error) {
      setError(String(error));
      throw error;
    } finally {
      setCreatingProject(false);
    }
  }, [setError, setCreatingProject, activateProjectFromInfo, refreshRecentProjects]);

  const handleSelectProject = useCallback(async (project: Project): Promise<boolean> => {
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
  }, [setError, activateProjectFromInfo]);

  return useMemo(() => ({
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
  }), [
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
  ]);
}

import { invoke } from '@tauri-apps/api/core';
import {
  ADR,
  AdrEntry,
  AdrStatus,
  EventRecord,
  FeatureDocument as Feature,
  FeatureInfo as FeatureEntry,
  LogEntry,
  McpServerConfig,
  ModeConfig,
  NoteDocument,
  NoteInfo as NoteEntry,
  ProjectDiscovery,
  ProjectInfo,
  ProjectConfig,
  ReleaseDocument as Release,
  ReleaseInfo,
  SpecEntry as RawSpecInfo,
  Workspace,
  Result,
  commands as spectaCommands,
} from '@/bindings';
import {
  SpecInfo,
  toSpecInfo,
} from '@/lib/types/spec';

export interface CreateProjectPayload {
  directory: string;
  name?: string;
  description?: string;
  config?: ProjectConfig;
}

export interface WorkspaceEditorInfo {
  id: string;
  name: string;
  binary: string;
}

export interface WorkspaceFileChange {
  status: string;
  path: string;
}

export interface WorkspaceSessionInfo {
  id: string;
  workspace_id: string;
  workspace_branch: string;
  status: 'active' | 'ended';
  started_at: string;
  ended_at?: string | null;
  mode_id?: string | null;
  primary_provider?: string | null;
  goal?: string | null;
  summary?: string | null;
  updated_feature_ids: string[];
  updated_spec_ids: string[];
  compiled_at?: string | null;
  compile_error?: string | null;
  created_at: string;
  updated_at: string;
}

export type TemplateKind = 'issue' | 'adr' | 'spec' | 'release' | 'feature' | 'vision';
export type NotesScope = 'project' | 'global';

const unwrapResult = async <T>(promise: Promise<Result<T, string>>): Promise<T> => {
  const result = await promise;
  if (result.status === 'ok') {
    return result.data;
  }
  throw new Error(String(result.error));
};

export const listAdrs = (): Promise<AdrEntry[]> => invoke('list_adrs_cmd');
export const listSpecs = (): Promise<SpecInfo[]> =>
  invoke<RawSpecInfo[]>('list_specs_cmd').then((entries) => entries.map(toSpecInfo));
export const listReleases = (): Promise<ReleaseInfo[]> => invoke<ReleaseInfo[]>('list_releases_cmd');
export const listFeatures = (): Promise<FeatureEntry[]> => invoke('list_features_cmd');
export const listNotes = (scope: NotesScope = 'project'): Promise<NoteEntry[]> =>
  invoke('list_notes_cmd', { scope });
export const listLogEntries = (): Promise<LogEntry[]> => invoke('get_log');
export const listEventEntries = (
  since?: number,
  limit?: number
): Promise<EventRecord[]> => invoke('list_events_cmd', { since, limit });
export const ingestEventChanges = (): Promise<number> => invoke('ingest_events_cmd');
export const getWorkspaceCmd = (branch: string): Promise<Result<Workspace | null, string>> =>
  invoke('get_workspace_cmd', { branch }).then(data => ({ status: 'ok', data } as Result<Workspace | null, string>)).catch(error => ({ status: 'error', error }));
export const listWorkspaceEditorsCmd = (): Promise<Result<WorkspaceEditorInfo[], string>> =>
  invoke('list_workspace_editors_cmd').then(data => ({ status: 'ok', data } as Result<WorkspaceEditorInfo[], string>)).catch(error => ({ status: 'error', error }));
export const listWorkspacesCmd = (): Promise<Result<Workspace[], string>> =>
  invoke('list_workspaces_cmd').then(data => ({ status: 'ok', data } as Result<Workspace[], string>)).catch(error => ({ status: 'error', error }));
export const syncWorkspaceCmd = (branch?: string | null): Promise<Result<Workspace, string>> =>
  invoke('sync_workspace_cmd', { branch: branch ?? null }).then(data => ({ status: 'ok', data } as Result<Workspace, string>)).catch(error => ({ status: 'error', error }));
export const createWorkspaceCmd = (
  branch: string,
  options?: {
    workspaceType?: string | null;
    featureId?: string | null;
    specId?: string | null;
    releaseId?: string | null;
    activate?: boolean;
  }
): Promise<Result<Workspace, string>> =>
  invoke('create_workspace_cmd', {
    branch,
    workspaceType: options?.workspaceType ?? null,
    featureId: options?.featureId ?? null,
    specId: options?.specId ?? null,
    releaseId: options?.releaseId ?? null,
    activate: options?.activate ?? null,
  }).then(data => ({ status: 'ok', data } as Result<Workspace, string>)).catch(error => ({ status: 'error', error }));
export const activateWorkspaceCmd = (branch: string): Promise<Result<Workspace, string>> =>
  invoke('activate_workspace_cmd', { branch }).then(data => ({ status: 'ok', data } as Result<Workspace, string>)).catch(error => ({ status: 'error', error }));
export const deleteWorkspaceCmd = (branch: string): Promise<Result<null, string>> =>
  invoke('delete_workspace_cmd', { branch })
    .then(() => ({ status: 'ok', data: null } as Result<null, string>))
    .catch(error => ({ status: 'error', error: String(error) }));
export const getActiveWorkspaceSessionCmd = (
  branch: string
): Promise<Result<WorkspaceSessionInfo | null, string>> =>
  invoke('get_active_workspace_session_cmd', { branch })
    .then(data => ({ status: 'ok', data } as Result<WorkspaceSessionInfo | null, string>))
    .catch(error => ({ status: 'error', error: String(error) }));
export const listWorkspaceSessionsCmd = (
  branch?: string | null,
  limit?: number | null
): Promise<Result<WorkspaceSessionInfo[], string>> =>
  invoke('list_workspace_sessions_cmd', { branch: branch ?? null, limit: limit ?? null })
    .then(data => ({ status: 'ok', data } as Result<WorkspaceSessionInfo[], string>))
    .catch(error => ({ status: 'error', error: String(error) }));
export const listWorkspaceChangesCmd = (
  branch: string
): Promise<Result<WorkspaceFileChange[], string>> =>
  invoke('list_workspace_changes_cmd', { branch })
    .then(data => ({ status: 'ok', data } as Result<WorkspaceFileChange[], string>))
    .catch(error => ({ status: 'error', error: String(error) }));
export const openWorkspaceEditorCmd = (
  branch: string,
  editor: string
): Promise<Result<null, string>> =>
  invoke('open_workspace_editor_cmd', { branch, editor })
    .then(() => ({ status: 'ok', data: null } as Result<null, string>))
    .catch(error => ({ status: 'error', error: String(error) }));
export const transitionWorkspaceCmd = (
  branch: string,
  status: string
): Promise<Result<Workspace, string>> =>
  invoke('transition_workspace_cmd', { branch, status }).then(data => ({ status: 'ok', data } as Result<Workspace, string>)).catch(error => ({ status: 'error', error }));
export const getCurrentBranchCmd = (): Promise<string | null> =>
  invoke<string | null>('get_current_branch_cmd').catch(() => null);

export const getProjectConfigCmd = (): Promise<ProjectConfig> => invoke('get_project_config');
export const saveProjectConfigCmd = (config: ProjectConfig): Promise<void> =>
  invoke('save_project_config', { config });
export const getAppSettingsCmd = (): Promise<ProjectConfig> => invoke('get_app_settings');
export const saveAppSettingsCmd = (config: ProjectConfig): Promise<void> =>
  invoke('save_app_settings', { config });

export const listProjects = (): Promise<ProjectDiscovery[]> => invoke('list_projects');
export const detectCurrentProject = (): Promise<ProjectInfo | null> => invoke('detect_current_project');
export const getActiveProject = (): Promise<ProjectInfo | null> => invoke('get_active_project');
export const pickAndOpenProject = (): Promise<ProjectInfo> => invoke('pick_and_open_project');
export const createNewProjectCmd = (): Promise<ProjectInfo> => invoke('create_new_project');
export const pickProjectDirectoryCmd = (): Promise<string | null> => invoke('pick_project_directory');
export const createProjectWithOptionsCmd = (payload: CreateProjectPayload): Promise<ProjectInfo> =>
  invoke('create_project_with_options', { ...payload });
export const setActiveProjectCmd = (path: string): Promise<ProjectInfo> =>
  invoke('set_active_project', { path });
export const renameProjectCmd = (path: string, name: string): Promise<ProjectInfo> =>
  invoke('rename_project_cmd', { path, name });

export const createNewAdrCmd = (
  title: string,
  context: string,
  decision: string
): Promise<AdrEntry> => unwrapResult(spectaCommands.createNewAdr(title, context, decision));

export const getAdrCmd = (id: string): Promise<AdrEntry> =>
  unwrapResult(spectaCommands.getAdrCmd(id));

export const updateAdrCmd = (id: string, adr: ADR): Promise<AdrEntry> =>
  unwrapResult(spectaCommands.updateAdrCmd(id, adr));

export const moveAdrCmd = (id: string, newStatus: AdrStatus): Promise<AdrEntry> =>
  unwrapResult(spectaCommands.moveAdrCmd(id, newStatus));

export const deleteAdrCmd = (id: string): Promise<void> =>
  unwrapResult(spectaCommands.deleteAdrCmd(id)).then(() => undefined);

export const getSpecCmd = (id: string): Promise<Result<SpecInfo, string>> =>
  spectaCommands.getSpecCmd(id)
    .then((result) => {
      if (result.status === 'ok') {
        return { status: 'ok', data: toSpecInfo(result.data) } as Result<SpecInfo, string>;
      }
      return result as unknown as Result<SpecInfo, string>;
    });

export const createSpecCmd = (title: string, content: string): Promise<Result<SpecInfo, string>> =>
  invoke<RawSpecInfo>('create_spec_cmd', { title, content })
    .then((data) => ({ status: 'ok', data: toSpecInfo(data) } as Result<SpecInfo, string>))
    .catch((error) => ({ status: 'error', error: String(error) }));

export const updateSpecCmd = async (id: string, content: string): Promise<Result<SpecInfo, string>> => {
  const existing = await spectaCommands.getSpecCmd(id);
  if (existing.status === 'error') {
    return existing as unknown as Result<SpecInfo, string>;
  }
  return spectaCommands.updateSpecCmd(id, { ...existing.data.spec, body: content })
    .then((result) => {
      if (result.status === 'ok') {
        return { status: 'ok', data: toSpecInfo(result.data) } as Result<SpecInfo, string>;
      }
      return result as unknown as Result<SpecInfo, string>;
    });
};

export const deleteSpecCmd = (id: string): Promise<Result<null, string>> =>
  spectaCommands.deleteSpecCmd(id);

export const getReleaseCmd = (fileName: string): Promise<Result<Release, string>> =>
  invoke('get_release_cmd', { fileName }).then(data => ({ status: 'ok', data } as Result<Release, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const createReleaseCmd = (version: string, content: string): Promise<Result<Release, string>> =>
  invoke('create_release_cmd', { version, content }).then(data => ({ status: 'ok', data } as Result<Release, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const updateReleaseCmd = (fileName: string, content: string): Promise<Result<Release, string>> =>
  invoke('update_release_cmd', { fileName, content }).then(data => ({ status: 'ok', data } as Result<Release, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const getFeatureCmd = (fileName: string): Promise<Result<Feature, string>> =>
  invoke('get_feature_cmd', { fileName }).then(data => ({ status: 'ok', data } as Result<Feature, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const createFeatureCmd = (
  title: string,
  content: string,
  release?: string | null,
  spec?: string | null
): Promise<Result<Feature, string>> => invoke('create_feature_cmd', { title, content, release, spec }).then(data => ({ status: 'ok', data } as Result<Feature, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const updateFeatureCmd = (fileName: string, content: string): Promise<Result<Feature, string>> =>
  invoke('update_feature_cmd', { fileName, content }).then(data => ({ status: 'ok', data } as Result<Feature, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const getNoteCmd = (id: string, scope: NotesScope = 'project'): Promise<NoteDocument> =>
  invoke('get_note_cmd', { id, scope });

export const createNoteCmd = (
  title: string,
  content: string,
  scope: NotesScope = 'project'
): Promise<NoteDocument> =>
  invoke('create_note_cmd', { title, content, scope });

export const updateNoteCmd = (
  id: string,
  content: string,
  scope: NotesScope = 'project'
): Promise<NoteDocument> =>
  invoke('update_note_cmd', { id, content, scope });

export const deleteNoteCmd = (
  id: string,
  scope: NotesScope = 'project'
): Promise<void> =>
  invoke('delete_note_cmd', { id, scope });

export const getTemplateCmd = (kind: TemplateKind): Promise<string> =>
  invoke('get_template_cmd', { kind });
export const saveTemplateCmd = (kind: TemplateKind, content: string): Promise<void> =>
  invoke('save_template_cmd', { kind, content });

// Modes
export const listModesCmd = (): Promise<ModeConfig[]> =>
  invoke('list_modes_cmd');
export const addModeCmd = (mode: ModeConfig): Promise<void> =>
  invoke('add_mode_cmd', { mode });
export const removeModeCmd = (id: string): Promise<void> =>
  invoke('remove_mode_cmd', { id });
export const setActiveModeCmd = (id: string | null): Promise<void> =>
  invoke('set_active_mode_cmd', { id });
export const getActiveModeCmd = (): Promise<ModeConfig | null> =>
  invoke('get_active_mode_cmd');

// MCP servers
export const listMcpServersCmd = (): Promise<McpServerConfig[]> =>
  invoke('list_mcp_servers_cmd');
export const addMcpServerCmd = (server: McpServerConfig): Promise<void> =>
  invoke('add_mcp_server_cmd', { server });
export const removeMcpServerCmd = (id: string): Promise<void> =>
  invoke('remove_mcp_server_cmd', { id });

// Agent export
export const exportAgentConfigCmd = (
  target: 'claude' | 'codex' | 'gemini'
): Promise<void> =>
  invoke('export_agent_config_cmd', { target });

// AI
export const generateAdrCmd = (title: string, context: string): Promise<string> =>
  invoke('generate_adr_cmd', { title, context });
export const transformTextCmd = (instruction: string, text: string): Promise<string> =>
  invoke('transform_text_cmd', { instruction, text });

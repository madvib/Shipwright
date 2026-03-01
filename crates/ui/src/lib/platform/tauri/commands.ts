import { invoke } from '@tauri-apps/api/core';
import {
  AdrEntry,
  EventRecord,
  FeatureDocument,
  FeatureInfo as FeatureEntry,
  Issue,
  IssueEntry,
  LogEntry,
  McpServerConfig,
  ModeConfig,
  NoteDocument,
  NoteInfo as NoteEntry,
  ProjectDiscovery,
  ProjectInfo,
  ProjectConfig,
  ReleaseDocument,
  ReleaseInfo as ReleaseEntry,
  SpecDocument,
  SpecInfo as SpecEntry,
  Workspace,
  Result,
} from '@/bindings';

export interface CreateProjectPayload {
  directory: string;
  name?: string;
  description?: string;
  config?: ProjectConfig;
}

export type TemplateKind = 'issue' | 'adr' | 'spec' | 'release' | 'feature' | 'vision';

export const listIssues = (): Promise<IssueEntry[]> => invoke('list_items');
export const listAdrs = (): Promise<AdrEntry[]> => invoke('list_adrs_cmd');
export const listSpecs = (): Promise<SpecEntry[]> => invoke('list_specs_cmd');
export const listReleases = (): Promise<ReleaseEntry[]> => invoke('list_releases_cmd');
export const listFeatures = (): Promise<FeatureEntry[]> => invoke('list_features_cmd');
export const listNotes = (): Promise<NoteEntry[]> => invoke('list_notes_cmd');
export const listLogEntries = (): Promise<LogEntry[]> => invoke('get_log');
export const listEventEntries = (
  since?: number,
  limit?: number
): Promise<EventRecord[]> => invoke('list_events_cmd', { since, limit });
export const ingestEventChanges = (): Promise<number> => invoke('ingest_events_cmd');
export const getWorkspaceCmd = (branch: string): Promise<Result<Workspace | null, string>> =>
  invoke('get_workspace_cmd', { branch }).then(data => ({ status: 'ok', data } as Result<Workspace | null, string>)).catch(error => ({ status: 'error', error }));
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

export const createNewIssueCmd = (
  title: string,
  description: string,
  status: string,
  assignee?: string | null,
  tags?: string[] | null
): Promise<IssueEntry> => invoke('create_new_issue', { title, description, status, assignee, tags });

export const moveIssueStatusCmd = (
  fileName: string,
  fromStatus: string,
  toStatus: string
): Promise<IssueEntry> => invoke('move_issue_status', { fileName, fromStatus, toStatus });

export const updateIssueByPathCmd = (path: string, issue: Issue): Promise<void> =>
  invoke('update_issue_by_path', { path, issue });

export const deleteIssueByPathCmd = (path: string): Promise<void> => invoke('delete_issue_by_path', { path });

export const createNewAdrCmd = (title: string, decision: string): Promise<AdrEntry> =>
  invoke('create_new_adr', { title, decision });

export const getAdrCmd = (fileName: string): Promise<AdrEntry> => invoke('get_adr_cmd', { fileName });

export const updateAdrCmd = (fileName: string, adr: AdrEntry['adr']): Promise<AdrEntry> =>
  invoke('update_adr_cmd', { fileName, adr });

export const deleteAdrCmd = (fileName: string): Promise<void> => invoke('delete_adr_cmd', { fileName });

export const getSpecCmd = (fileName: string): Promise<Result<SpecDocument, string>> =>
  invoke('get_spec_cmd', { fileName }).then(data => ({ status: 'ok', data } as Result<SpecDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const createSpecCmd = (title: string, content: string): Promise<Result<SpecDocument, string>> =>
  invoke('create_spec_cmd', { title, content }).then(data => ({ status: 'ok', data } as Result<SpecDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const updateSpecCmd = (fileName: string, content: string): Promise<Result<SpecDocument, string>> =>
  invoke('update_spec_cmd', { fileName, content }).then(data => ({ status: 'ok', data } as Result<SpecDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const deleteSpecCmd = (fileName: string): Promise<Result<null, string>> =>
  invoke('delete_spec_cmd', { fileName }).then(() => ({ status: 'ok' as const, data: null })).catch(error => ({ status: 'error', error: String(error) }));

export const getReleaseCmd = (fileName: string): Promise<Result<ReleaseDocument, string>> =>
  invoke('get_release_cmd', { fileName }).then(data => ({ status: 'ok', data } as Result<ReleaseDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const createReleaseCmd = (version: string, content: string): Promise<Result<ReleaseDocument, string>> =>
  invoke('create_release_cmd', { version, content }).then(data => ({ status: 'ok', data } as Result<ReleaseDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const updateReleaseCmd = (fileName: string, content: string): Promise<Result<ReleaseDocument, string>> =>
  invoke('update_release_cmd', { fileName, content }).then(data => ({ status: 'ok', data } as Result<ReleaseDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const getFeatureCmd = (fileName: string): Promise<Result<FeatureDocument, string>> =>
  invoke('get_feature_cmd', { fileName }).then(data => ({ status: 'ok', data } as Result<FeatureDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const createFeatureCmd = (
  title: string,
  content: string,
  release?: string | null,
  spec?: string | null
): Promise<Result<FeatureDocument, string>> => invoke('create_feature_cmd', { title, content, release, spec }).then(data => ({ status: 'ok', data } as Result<FeatureDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const updateFeatureCmd = (fileName: string, content: string): Promise<Result<FeatureDocument, string>> =>
  invoke('update_feature_cmd', { fileName, content }).then(data => ({ status: 'ok', data } as Result<FeatureDocument, string>)).catch(error => ({ status: 'error', error: String(error) }));

export const getNoteCmd = (fileName: string): Promise<NoteDocument> =>
  invoke('get_note_cmd', { fileName });

export const createNoteCmd = (title: string, content: string): Promise<NoteDocument> =>
  invoke('create_note_cmd', { title, content });

export const updateNoteCmd = (fileName: string, content: string): Promise<NoteDocument> =>
  invoke('update_note_cmd', { fileName, content });

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
export const generateIssueDescriptionCmd = (title: string): Promise<string> =>
  invoke('generate_issue_description_cmd', { title });

export const generateAdrCmd = (title: string, context: string): Promise<string> =>
  invoke('generate_adr_cmd', { title, context });

export const brainstormIssuesCmd = (topic: string): Promise<string[]> =>
  invoke('brainstorm_issues_cmd', { topic });
export const transformTextCmd = (instruction: string, text: string): Promise<string> =>
  invoke('transform_text_cmd', { instruction, text });

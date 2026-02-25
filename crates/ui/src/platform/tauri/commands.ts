import { invoke } from '@tauri-apps/api/core';
import { AdrEntry, Issue, IssueEntry, LogEntry, McpServerConfig, ModeConfig, ProjectConfig, SpecDocument, SpecEntry } from '../../types';

export interface ProjectInfo {
  name: string;
  path: string;
  issue_count: number;
}

export interface ProjectDiscovery {
  name: string;
  path: string;
  issue_count?: number;
}

export interface CreateProjectPayload {
  directory: string;
  name?: string;
  description?: string;
  config?: ProjectConfig;
}

export const listIssues = (): Promise<IssueEntry[]> => invoke('list_items');
export const listAdrs = (): Promise<AdrEntry[]> => invoke('list_adrs_cmd');
export const listSpecs = (): Promise<SpecEntry[]> => invoke('list_specs_cmd');
export const listLogEntries = (): Promise<LogEntry[]> => invoke('get_log');

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
  status: string
): Promise<IssueEntry> => invoke('create_new_issue', { title, description, status });

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

export const getSpecCmd = (fileName: string): Promise<SpecDocument> =>
  invoke('get_spec_cmd', { fileName });

export const createSpecCmd = (title: string, content: string): Promise<SpecDocument> =>
  invoke('create_spec_cmd', { title, content });

export const updateSpecCmd = (fileName: string, content: string): Promise<SpecDocument> =>
  invoke('update_spec_cmd', { fileName, content });

export const deleteSpecCmd = (fileName: string): Promise<void> =>
  invoke('delete_spec_cmd', { fileName });

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

export type ComposerArtifactKind = 'skill' | 'mcp' | 'rule';

export interface ComposerTemplate {
  id: string;
  name: string;
  description: string;
  targetAgents: string[];
  recommendedSkills: string[];
  recommendedMcpServers: string[];
  toolAllow: string[];
  toolDeny: string[];
}

export interface ComposerArtifact {
  id: string;
  name: string;
  kind: ComposerArtifactKind;
  scope: 'project' | 'global';
  description?: string;
}

export interface ComposerSelection {
  templateId: string | null;
  skillIds: string[];
  mcpServerIds: string[];
  ruleIds: string[];
}

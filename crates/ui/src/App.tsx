import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  AdrEntry,
  Config,
  IssueEntry,
  IssueStatus,
  LogEntry,
  NavSection,
  Project
} from './types';
import Sidebar from './components/Sidebar';
import IssueList from './components/IssueList';
import IssueDetail from './components/IssueDetail';
import NewIssueModal from './components/NewIssueModal';
import LogPanel from './components/LogPanel';
import SettingsPanel from './components/SettingsPanel';
import AdrList from './components/AdrList';
import NewAdrModal from './components/NewAdrModal';
import './App.css';

interface ProjectInfo {
  name: string;
  path: string;
  issue_count: number;
}

interface ProjectDiscovery {
  name: string;
  path: string;
}

export default function App() {
  // State
  const [activeSection, setActiveSection] = useState<NavSection>('issues');
  const [activeProject, setActiveProject] = useState<Project | null>(null);
  const [recentProjects, setRecentProjects] = useState<Project[]>([]);
  const [issues, setIssues] = useState<IssueEntry[]>([]);
  const [adrs, setAdrs] = useState<AdrEntry[]>([]);
  const [logEntries, setLogEntries] = useState<LogEntry[]>([]);
  const [config, setConfig] = useState<Config>({});
  const [selectedIssue, setSelectedIssue] = useState<IssueEntry | null>(null);
  const [showNewIssue, setShowNewIssue] = useState(false);
  const [showNewAdr, setShowNewAdr] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // ─── Initial Load ───────────────────────────────────────────────────────────

  const loadProjectData = useCallback(async () => {
    try {
      const [issueList, adrList, logList] = await Promise.all([
        invoke<IssueEntry[]>('list_items').catch(() => []),
        invoke<AdrEntry[]>('list_adrs_cmd').catch(() => []),
        invoke<LogEntry[]>('get_log').catch(() => []),
      ]);
      setIssues(issueList);
      setAdrs(adrList);
      setLogEntries(logList);
    } catch (e) {
      console.error('Failed to load project data', e);
    }
  }, []);

  useEffect(() => {
    (async () => {
      try {
        // Load nearby projects
        const projects = await invoke<ProjectDiscovery[]>('list_projects').catch(() => []);
        setRecentProjects(projects.map((p) => ({ name: p.name, path: p.path.toString() })));

        // Auto-detect current project (for dogfooding)
        const detected = await invoke<ProjectInfo | null>('detect_current_project').catch(() => null);
        if (detected) {
          setActiveProject({
            name: detected.name,
            path: detected.path,
            issue_count: detected.issue_count,
          });
          await loadProjectData();
        }

        // Load settings
        const cfg = await invoke<Config>('get_app_settings').catch(() => ({}));
        setConfig(cfg);
      } catch (e) {
        console.error('Init error', e);
      } finally {
        setLoading(false);
      }
    })();
  }, [loadProjectData]);

  // ─── Project Actions ────────────────────────────────────────────────────────

  const handleOpenProject = async () => {
    try {
      const info = await invoke<ProjectInfo>('pick_and_open_project');
      setActiveProject({ name: info.name, path: info.path, issue_count: info.issue_count });
      setActiveSection('issues');
      await loadProjectData();
    } catch (e) {
      if (String(e).includes('No directory selected')) return;
      setError(String(e));
    }
  };

  const handleSelectProject = async (p: Project) => {
    try {
      const info = await invoke<ProjectInfo>('set_active_project', { path: p.path });
      setActiveProject({ name: info.name, path: info.path, issue_count: info.issue_count });
      setActiveSection('issues');
      await loadProjectData();
    } catch (e) {
      setError(String(e));
    }
  };

  // ─── Issue Actions ──────────────────────────────────────────────────────────

  const handleCreateIssue = async (title: string, description: string, status: IssueStatus) => {
    try {
      const entry = await invoke<IssueEntry>('create_new_issue', { title, description, status });
      setIssues((prev) => [...prev, entry]);
      setShowNewIssue(false);
      refreshLog();
      refreshProjectInfo();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleStatusChange = async (file_name: string, from: string, to: string) => {
    try {
      const updated = await invoke<IssueEntry>('move_issue_status', {
        fileName: file_name,
        fromStatus: from,
        toStatus: to,
      });
      setIssues((prev) => prev.map((i) => (i.file_name === file_name ? updated : i)));
      setSelectedIssue(updated);
      refreshLog();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSaveIssue = async (path: string, title: string, description: string) => {
    const existing = issues.find((i) => i.path === path);
    if (!existing) return;
    const updated: IssueEntry = {
      ...existing,
      issue: { ...existing.issue, title, description },
    };
    try {
      await invoke('update_issue_by_path', { path, issue: updated.issue });
      setIssues((prev) => prev.map((i) => (i.path === path ? updated : i)));
      setSelectedIssue(updated);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDeleteIssue = async (path: string) => {
    try {
      await invoke('delete_issue_by_path', { path });
      setIssues((prev) => prev.filter((i) => i.path !== path));
      setSelectedIssue(null);
      refreshLog();
      refreshProjectInfo();
    } catch (e) {
      setError(String(e));
    }
  };

  // ─── ADR Actions ────────────────────────────────────────────────────────────

  const handleCreateAdr = async (title: string, decision: string) => {
    try {
      const entry = await invoke<AdrEntry>('create_new_adr', { title, decision });
      setAdrs((prev) => [...prev, entry]);
      setShowNewAdr(false);
      refreshLog();
    } catch (e) {
      setError(String(e));
    }
  };

  // ─── Helpers ────────────────────────────────────────────────────────────────

  const refreshLog = async () => {
    const logList = await invoke<LogEntry[]>('get_log').catch(() => []);
    setLogEntries(logList);
  };

  const refreshProjectInfo = async () => {
    const info = await invoke<ProjectInfo | null>('get_active_project').catch(() => null);
    if (info) setActiveProject({ name: info.name, path: info.path, issue_count: info.issue_count });
  };

  const handleSaveSettings = async (newConfig: Config) => {
    try {
      await invoke('save_app_settings', { config: newConfig });
      setConfig(newConfig);
      setActiveSection('issues');
    } catch (e) {
      setError(String(e));
    }
  };

  // ─── Render ─────────────────────────────────────────────────────────────────

  const noProject = !activeProject;

  return (
    <div className="app-shell">
      <Sidebar
        activeSection={activeSection}
        onSectionChange={setActiveSection}
        activeProject={activeProject}
        recentProjects={recentProjects}
        onOpenProject={handleOpenProject}
        onSelectProject={handleSelectProject}
      />

      <main className="main-content">
        {/* Error Banner */}
        {error && (
          <div className="error-banner">
            <span>{error}</span>
            <button onClick={() => setError(null)}>✕</button>
          </div>
        )}

        {/* No Project Welcome */}
        {noProject && !loading && (
          <div className="welcome-screen">
            <div className="welcome-logo-ring">
              <div className="welcome-logo-glow" />
              <div className="welcome-logo-box">
                <img src="/ship-logo.png" alt="Ship" className="welcome-logo-img" />
              </div>
            </div>
            <h1 className="welcome-title">Ready for&nbsp;Liftoff?</h1>
            <p className="welcome-desc">
              Open a project folder to start tracking issues and architecture decisions.
            </p>
            <button className="btn-primary btn-lg" onClick={handleOpenProject}>
              Open Project
            </button>
          </div>
        )}

        {/* Main sections */}
        {!noProject && (
          <>
            {activeSection === 'issues' && (
              <div className="section-wrap">
                <div className="section-header">
                  <div>
                    <h1 className="page-title">{activeProject.name}</h1>
                    <p className="page-subtitle">{issues.length} issue{issues.length !== 1 ? 's' : ''}</p>
                  </div>
                  <button className="btn-primary" onClick={() => setShowNewIssue(true)}>
                    ＋ New Issue
                  </button>
                </div>
                {issues.length === 0 ? (
                  <div className="empty-state">
                    <div className="empty-icon">⚡</div>
                    <h3 className="empty-title">No issues yet</h3>
                    <p className="empty-desc">Create your first issue to start tracking work.</p>
                    <button className="btn-primary" onClick={() => setShowNewIssue(true)}>
                      Create First Issue
                    </button>
                  </div>
                ) : (
                  <IssueList
                    issues={issues}
                    onSelect={setSelectedIssue}
                    onNewIssue={() => setShowNewIssue(true)}
                  />
                )}
                <LogPanel entries={logEntries} onRefresh={refreshLog} />
              </div>
            )}

            {activeSection === 'adrs' && (
              <div className="section-wrap">
                <AdrList adrs={adrs} onNewAdr={() => setShowNewAdr(true)} />
                <LogPanel entries={logEntries} onRefresh={refreshLog} />
              </div>
            )}

            {activeSection === 'log' && (
              <div className="section-wrap">
                <div className="page-header">
                  <div>
                    <h1 className="page-title">Activity Log</h1>
                    <p className="page-subtitle">Full history for {activeProject.name}</p>
                  </div>
                  <button className="btn-ghost" onClick={refreshLog}>↺ Refresh</button>
                </div>
                <div className="log-full">
                  {logEntries.length === 0 ? (
                    <div className="empty-state">
                      <div className="empty-icon">📋</div>
                      <h3 className="empty-title">No activity yet</h3>
                      <p className="empty-desc">Start working on issues to see activity here.</p>
                    </div>
                  ) : (
                    logEntries.map((entry, i) => (
                      <div key={i} className="log-entry-full">
                        <div className="log-entry-action">{entry.action}</div>
                        <div className="log-entry-details">{entry.details}</div>
                        <div className="log-entry-time">
                          {new Date(entry.timestamp).toLocaleString()}
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}
          </>
        )}

        {activeSection === 'settings' && (
          <SettingsPanel
            config={config}
            onSave={handleSaveSettings}
            onBack={() => setActiveSection('issues')}
          />
        )}
      </main>

      {/* Modals / Overlays */}
      {selectedIssue && (
        <IssueDetail
          entry={selectedIssue}
          onClose={() => setSelectedIssue(null)}
          onStatusChange={handleStatusChange}
          onDelete={handleDeleteIssue}
          onSave={handleSaveIssue}
        />
      )}
      {showNewIssue && (
        <NewIssueModal
          onClose={() => setShowNewIssue(false)}
          onSubmit={handleCreateIssue}
          defaultStatus={(config.default_status as IssueStatus) ?? 'backlog'}
        />
      )}
      {showNewAdr && (
        <NewAdrModal
          onClose={() => setShowNewAdr(false)}
          onSubmit={handleCreateAdr}
        />
      )}
    </div>
  );
}

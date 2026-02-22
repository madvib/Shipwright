import { NavSection, Project } from '../types';

interface SidebarProps {
    activeSection: NavSection;
    onSectionChange: (s: NavSection) => void;
    activeProject: Project | null;
    recentProjects: Project[];
    onOpenProject: () => void;
    onSelectProject: (p: Project) => void;
}

const NAV_ITEMS: { id: NavSection; label: string; icon: string }[] = [
    { id: 'issues', label: 'Issues', icon: '⚡' },
    { id: 'adrs', label: 'Decisions', icon: '📐' },
    { id: 'log', label: 'Activity', icon: '📋' },
];

export default function Sidebar({
    activeSection,
    onSectionChange,
    activeProject,
    recentProjects,
    onOpenProject,
    onSelectProject,
}: SidebarProps) {
    return (
        <aside className="sidebar">
            {/* Logo */}
            <div className="sidebar-logo">
                <div className="logo-mark">
                    <img src="/ship-logo.png" alt="Ship" className="logo-img" />
                </div>
                <span className="logo-text">Ship</span>
            </div>

            {/* Project Switcher */}
            <div className="sidebar-section">
                <p className="sidebar-label">Project</p>
                {activeProject ? (
                    <div className="active-project-card">
                        <div className="active-project-dot" />
                        <div className="active-project-info">
                            <span className="active-project-name">{activeProject.name}</span>
                            <span className="active-project-count">
                                {activeProject.issue_count ?? 0} issues
                            </span>
                        </div>
                    </div>
                ) : (
                    <div className="no-project-hint">No project selected</div>
                )}

                {recentProjects.length > 0 && (
                    <div className="recent-projects">
                        {recentProjects
                            .filter((p) => p.name !== activeProject?.name)
                            .slice(0, 4)
                            .map((p) => (
                                <button
                                    key={p.path}
                                    className="recent-project-btn"
                                    onClick={() => onSelectProject(p)}
                                    title={p.path}
                                >
                                    <span className="recent-project-dot" />
                                    {p.name}
                                </button>
                            ))}
                    </div>
                )}

                <button className="open-project-btn" onClick={onOpenProject}>
                    <span>＋</span> Open Project
                </button>
            </div>

            <div className="sidebar-divider" />

            {/* Navigation */}
            <nav className="sidebar-nav">
                {NAV_ITEMS.map((item) => (
                    <button
                        key={item.id}
                        className={`nav-item ${activeSection === item.id ? 'nav-item-active' : ''}`}
                        onClick={() => onSectionChange(item.id)}
                    >
                        <span className="nav-item-icon">{item.icon}</span>
                        {item.label}
                    </button>
                ))}
            </nav>

            <div className="sidebar-spacer" />

            {/* Settings at bottom */}
            <button
                className={`nav-item nav-item-settings ${activeSection === 'settings' ? 'nav-item-active' : ''}`}
                onClick={() => onSectionChange('settings')}
            >
                <span className="nav-item-icon">⚙</span>
                Settings
            </button>
        </aside>
    );
}

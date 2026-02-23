import { useState } from 'react';
import { LogEntry } from '../types';

interface LogPanelProps {
    entries: LogEntry[];
    onRefresh: () => void;
}

const ACTION_COLORS: Record<string, string> = {
    'init': 'text-purple-400',
    'issue create': 'text-emerald-400',
    'issue move': 'text-blue-400',
    'issue delete': 'text-red-400',
    'adr create': 'text-amber-400',
};

function relativeTime(timestamp: string): string {
    try {
        const date = new Date(timestamp);
        const diff = Date.now() - date.getTime();
        const mins = Math.floor(diff / 60000);
        const hours = Math.floor(mins / 60);
        const days = Math.floor(hours / 24);
        if (days > 0) return `${days}d ago`;
        if (hours > 0) return `${hours}h ago`;
        if (mins > 0) return `${mins}m ago`;
        return 'just now';
    } catch {
        return timestamp.slice(0, 10);
    }
}

export default function LogPanel({ entries, onRefresh }: LogPanelProps) {
    const [collapsed, setCollapsed] = useState(false);

    return (
        <div className={`log-panel ${collapsed ? 'log-panel-collapsed' : ''}`}>
            <div className="log-header">
                <button className="log-toggle" onClick={() => setCollapsed((c) => !c)}>
                    <span className="log-toggle-icon">{collapsed ? '▲' : '▼'}</span>
                    Activity Log
                    {entries.length > 0 && !collapsed && (
                        <span className="log-count">{entries.length}</span>
                    )}
                </button>
                {!collapsed && (
                    <button className="log-refresh" onClick={onRefresh} title="Refresh log">
                        ↺
                    </button>
                )}
            </div>

            {!collapsed && (
                <div className="log-entries">
                    {entries.length === 0 ? (
                        <div className="log-empty">No activity yet. Start working on issues!</div>
                    ) : (
                        entries.map((entry, i) => (
                            <div key={i} className="log-entry">
                                <span className="log-actor text-zinc-600">[{entry.actor}]</span>
                                <span className={`log-action ${ACTION_COLORS[entry.action] ?? 'text-zinc-400'}`}>
                                    {entry.action}
                                </span>
                                <span className="log-details">{entry.details}</span>
                                <span className="log-time">{relativeTime(entry.timestamp)}</span>
                            </div>
                        ))
                    )}
                </div>
            )}
        </div>
    );
}

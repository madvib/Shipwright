import { IssueEntry, STATUS_CONFIG } from '../types';

interface IssueListProps {
    issues: IssueEntry[];
    onSelect: (entry: IssueEntry) => void;
    onNewIssue: () => void;
}

const COLUMNS = ['backlog', 'in-progress', 'blocked', 'done'];

export default function IssueList({ issues, onSelect, onNewIssue }: IssueListProps) {
    const byStatus = (status: string) => issues.filter((i) => i.status === status);

    return (
        <div className="issue-board">
            {COLUMNS.map((status) => {
                const cfg = STATUS_CONFIG[status];
                const col = byStatus(status);
                return (
                    <div key={status} className="issue-column">
                        <div className="column-header">
                            <span className={`column-status-dot ${cfg.color}`}>●</span>
                            <span className="column-title">{cfg.label}</span>
                            <span className="column-count">{col.length}</span>
                        </div>
                        <div className="column-cards">
                            {col.map((entry) => (
                                <button
                                    key={entry.path}
                                    className="issue-card"
                                    onClick={() => onSelect(entry)}
                                >
                                    <span className="issue-card-title">{entry.issue.title}</span>
                                    {entry.issue.description && (
                                        <span className="issue-card-desc">
                                            {entry.issue.description.slice(0, 100)}
                                            {entry.issue.description.length > 100 ? '…' : ''}
                                        </span>
                                    )}
                                    <span className="issue-card-date">
                                        {new Date(entry.issue.created_at).toLocaleDateString('en-US', {
                                            month: 'short', day: 'numeric',
                                        })}
                                    </span>
                                </button>
                            ))}
                            {status === 'backlog' && (
                                <button className="issue-card-new" onClick={onNewIssue}>
                                    <span className="issue-card-new-icon">＋</span>
                                    New Issue
                                </button>
                            )}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

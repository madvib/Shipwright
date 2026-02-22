import { useState } from 'react';
import { IssueEntry, STATUS_CONFIG } from '../types';

interface IssueDetailProps {
    entry: IssueEntry;
    onClose: () => void;
    onStatusChange: (file_name: string, from: string, to: string) => void;
    onDelete: (path: string) => void;
    onSave: (path: string, title: string, description: string) => void;
}

const STATUSES = ['backlog', 'in-progress', 'blocked', 'done'];

export default function IssueDetail({
    entry,
    onClose,
    onStatusChange,
    onDelete,
    onSave,
}: IssueDetailProps) {
    const [editTitle, setEditTitle] = useState(entry.issue.title);
    const [editDesc, setEditDesc] = useState(entry.issue.description);
    const [dirty, setDirty] = useState(false);
    const [confirmDelete, setConfirmDelete] = useState(false);
    const cfg = STATUS_CONFIG[entry.status] ?? STATUS_CONFIG['backlog'];

    const handleTitleChange = (v: string) => { setEditTitle(v); setDirty(true); };
    const handleDescChange = (v: string) => { setEditDesc(v); setDirty(true); };

    const handleSave = () => {
        onSave(entry.path, editTitle, editDesc);
        setDirty(false);
    };

    const createdDate = new Date(entry.issue.created_at).toLocaleDateString('en-US', {
        year: 'numeric', month: 'short', day: 'numeric',
    });

    return (
        <div className="detail-overlay" onClick={onClose}>
            <div className="detail-panel" onClick={(e) => e.stopPropagation()}>
                {/* Header */}
                <div className="detail-header">
                    <span className={`status-badge ${cfg.bg} ${cfg.color} ${cfg.border}`}>
                        {cfg.label}
                    </span>
                    <button className="detail-close-btn" onClick={onClose}>✕</button>
                </div>

                {/* Title */}
                <div className="detail-title-wrap">
                    <textarea
                        className="detail-title-input"
                        value={editTitle}
                        onChange={(e) => handleTitleChange(e.target.value)}
                        rows={2}
                    />
                </div>

                {/* Meta */}
                <div className="detail-meta">
                    <span>Created {createdDate}</span>
                    <span>·</span>
                    <span>{entry.file_name}</span>
                </div>

                {/* Description */}
                <div className="detail-section">
                    <label className="detail-label">Description</label>
                    <textarea
                        className="detail-textarea"
                        value={editDesc}
                        onChange={(e) => handleDescChange(e.target.value)}
                        rows={8}
                        placeholder="Describe this issue..."
                    />
                </div>

                {/* Status change */}
                <div className="detail-section">
                    <label className="detail-label">Status</label>
                    <div className="status-row">
                        {STATUSES.map((s) => {
                            const c = STATUS_CONFIG[s];
                            return (
                                <button
                                    key={s}
                                    className={`status-chip ${entry.status === s ? `${c.bg} ${c.color} ${c.border} ring-1 ring-current` : 'bg-zinc-900 text-zinc-500 border border-zinc-800'}`}
                                    onClick={() => {
                                        if (s !== entry.status) onStatusChange(entry.file_name, entry.status, s);
                                    }}
                                >
                                    {c.label}
                                </button>
                            );
                        })}
                    </div>
                </div>

                {/* Actions */}
                <div className="detail-actions">
                    {dirty && (
                        <button className="btn-primary" onClick={handleSave}>
                            Save Changes
                        </button>
                    )}
                    {!confirmDelete ? (
                        <button className="btn-danger-ghost" onClick={() => setConfirmDelete(true)}>
                            Delete
                        </button>
                    ) : (
                        <div className="confirm-delete">
                            <span>Are you sure?</span>
                            <button className="btn-danger" onClick={() => onDelete(entry.path)}>Yes, delete</button>
                            <button className="btn-ghost" onClick={() => setConfirmDelete(false)}>Cancel</button>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

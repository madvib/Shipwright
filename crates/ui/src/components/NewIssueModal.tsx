import { useState } from 'react';
import { IssueStatus, STATUS_CONFIG } from '../types';

interface NewIssueModalProps {
    onClose: () => void;
    onSubmit: (title: string, description: string, status: IssueStatus) => void;
    defaultStatus?: IssueStatus;
}

const STATUSES: IssueStatus[] = ['backlog', 'in-progress', 'blocked', 'done'];

export default function NewIssueModal({ onClose, onSubmit, defaultStatus = 'backlog' }: NewIssueModalProps) {
    const [title, setTitle] = useState('');
    const [description, setDescription] = useState('');
    const [status, setStatus] = useState<IssueStatus>(defaultStatus);
    const [error, setError] = useState('');

    const handleSubmit = (e: React.ChangeEvent) => {
        e.preventDefault();
        if (!title.trim()) {
            setError('Title is required');
            return;
        }
        onSubmit(title.trim(), description.trim(), status);
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
                <div className="modal-header">
                    <h2 className="modal-title">New Issue</h2>
                    <button className="modal-close" onClick={onClose}>✕</button>
                </div>

                <form onSubmit={handleSubmit} className="modal-form">
                    {error && <div className="form-error">{error}</div>}

                    <div className="form-group">
                        <label className="form-label">Title <span className="required">*</span></label>
                        <input
                            autoFocus
                            type="text"
                            className="form-input"
                            placeholder="Short, descriptive title"
                            value={title}
                            onChange={(e) => { setTitle(e.target.value); setError(''); }}
                        />
                    </div>

                    <div className="form-group">
                        <label className="form-label">Description</label>
                        <textarea
                            className="form-textarea"
                            placeholder="Steps to reproduce, context, links..."
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            rows={5}
                        />
                    </div>

                    <div className="form-group">
                        <label className="form-label">Initial Status</label>
                        <div className="status-row">
                            {STATUSES.map((s) => {
                                const c = STATUS_CONFIG[s];
                                return (
                                    <button
                                        type="button"
                                        key={s}
                                        className={`status-chip ${status === s
                                            ? `${c.bg} ${c.color} ${c.border} ring-1 ring-current`
                                            : 'bg-zinc-900 text-zinc-500 border border-zinc-800'
                                            }`}
                                        onClick={() => setStatus(s)}
                                    >
                                        {c.label}
                                    </button>
                                );
                            })}
                        </div>
                    </div>

                    <div className="modal-actions">
                        <button type="button" className="btn-ghost" onClick={onClose}>Cancel</button>
                        <button type="submit" className="btn-primary">Create Issue</button>
                    </div>
                </form>
            </div>
        </div>
    );
}

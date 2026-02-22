import { useState } from 'react';

interface NewAdrModalProps {
    onClose: () => void;
    onSubmit: (title: string, decision: string) => void;
}

export default function NewAdrModal({ onClose, onSubmit }: NewAdrModalProps) {
    const [title, setTitle] = useState('');
    const [decision, setDecision] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!title.trim()) { setError('Title is required'); return; }
        if (!decision.trim()) { setError('Decision text is required'); return; }
        onSubmit(title.trim(), decision.trim());
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
                <div className="modal-header">
                    <h2 className="modal-title">Record Decision</h2>
                    <button className="modal-close" onClick={onClose}>✕</button>
                </div>

                <form onSubmit={handleSubmit} className="modal-form">
                    {error && <div className="form-error">{error}</div>}

                    <div className="form-group">
                        <label className="form-label">Decision Title <span className="required">*</span></label>
                        <input
                            autoFocus
                            type="text"
                            className="form-input"
                            placeholder="e.g. Use PostgreSQL for persistence"
                            value={title}
                            onChange={(e) => { setTitle(e.target.value); setError(''); }}
                        />
                    </div>

                    <div className="form-group">
                        <label className="form-label">Decision Details <span className="required">*</span></label>
                        <textarea
                            className="form-textarea"
                            placeholder="Why this decision? What are the trade-offs?"
                            value={decision}
                            onChange={(e) => { setDecision(e.target.value); setError(''); }}
                            rows={7}
                        />
                    </div>

                    <div className="modal-actions">
                        <button type="button" className="btn-ghost" onClick={onClose}>Cancel</button>
                        <button type="submit" className="btn-primary">Record Decision</button>
                    </div>
                </form>
            </div>
        </div>
    );
}

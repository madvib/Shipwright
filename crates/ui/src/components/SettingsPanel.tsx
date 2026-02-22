import { useState } from 'react';
import { Config } from '../types';

interface SettingsPanelProps {
    config: Config;
    onSave: (config: Config) => void;
    onBack: () => void;
}

export default function SettingsPanel({ config, onSave, onBack }: SettingsPanelProps) {
    const [local, setLocal] = useState<Config>(config);

    return (
        <div className="settings-panel">
            <div className="page-header">
                <button className="back-btn" onClick={onBack}>← Back</button>
                <div>
                    <h1 className="page-title">Settings</h1>
                    <p className="page-subtitle">Global preferences for Ship</p>
                </div>
            </div>

            <div className="settings-card">
                <div className="settings-row">
                    <label className="form-label">Author Name</label>
                    <input
                        type="text"
                        className="form-input"
                        value={local.author ?? ''}
                        onChange={(e) => setLocal({ ...local, author: e.target.value })}
                        placeholder="e.g. Captain Nemo"
                    />
                </div>

                <div className="settings-row">
                    <label className="form-label">Default Issue Status</label>
                    <select
                        className="form-select"
                        value={local.default_status ?? 'backlog'}
                        onChange={(e) => setLocal({ ...local, default_status: e.target.value })}
                    >
                        <option value="backlog">Backlog</option>
                        <option value="in-progress">In Progress</option>
                        <option value="blocked">Blocked</option>
                        <option value="done">Done</option>
                    </select>
                </div>

                <div className="settings-toggle-row">
                    <div>
                        <div className="settings-toggle-label">Enable Notifications</div>
                        <div className="settings-toggle-hint">Get alerted on issue updates</div>
                    </div>
                    <button
                        className={`toggle-btn ${local.notifications_enabled ? 'toggle-on' : 'toggle-off'}`}
                        onClick={() => setLocal({ ...local, notifications_enabled: !local.notifications_enabled })}
                        aria-label="toggle notifications"
                    >
                        <span className="toggle-knob" />
                    </button>
                </div>
            </div>

            <div className="settings-actions">
                <button className="btn-ghost" onClick={onBack}>Cancel</button>
                <button className="btn-primary" onClick={() => onSave(local)}>Save Changes</button>
            </div>
        </div>
    );
}

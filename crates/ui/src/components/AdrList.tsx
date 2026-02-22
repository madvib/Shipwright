import { AdrEntry } from '../types';

interface AdrListProps {
    adrs: AdrEntry[];
    onNewAdr: () => void;
}

const STATUS_COLORS: Record<string, string> = {
    accepted: 'text-emerald-400',
    rejected: 'text-red-400',
    superseded: 'text-amber-400',
    proposed: 'text-blue-400',
};

export default function AdrList({ adrs, onNewAdr }: AdrListProps) {
    return (
        <div className="adr-container">
            <div className="adr-header">
                <div>
                    <h2 className="page-title">Architecture Decisions</h2>
                    <p className="page-subtitle">{adrs.length} recorded decision{adrs.length !== 1 ? 's' : ''}</p>
                </div>
                <button className="btn-primary" onClick={onNewAdr}>＋ New Decision</button>
            </div>

            {adrs.length === 0 ? (
                <div className="empty-state">
                    <div className="empty-icon">📐</div>
                    <h3 className="empty-title">No decisions yet</h3>
                    <p className="empty-desc">Document your architecture decisions to keep the team aligned.</p>
                    <button className="btn-primary" onClick={onNewAdr}>Record First Decision</button>
                </div>
            ) : (
                <div className="adr-list">
                    {adrs.map((entry) => (
                        <div key={entry.path} className="adr-card">
                            <div className="adr-card-header">
                                <span className={`adr-status ${STATUS_COLORS[entry.adr.status] ?? 'text-zinc-400'}`}>
                                    {entry.adr.status}
                                </span>
                                <span className="adr-date">
                                    {new Date(entry.adr.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                                </span>
                            </div>
                            <h3 className="adr-title">{entry.adr.title}</h3>
                            <p className="adr-decision">{entry.adr.decision.slice(0, 200)}{entry.adr.decision.length > 200 ? '…' : ''}</p>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}

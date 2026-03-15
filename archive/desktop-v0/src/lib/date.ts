export function relativeDate(iso: string): string {
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    const diff = Date.now() - date.getTime();
    const mins = Math.floor(diff / 60_000);
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    if (days < 30) return `${days}d ago`;
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

export function formatDate(dateStr: string): string {
    try {
        const date = new Date(dateStr);
        if (Number.isNaN(date.getTime())) return dateStr;
        return date.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
        });
    } catch {
        return dateStr;
    }
}

export function formatShortDate(dateStr: string): string {
    try {
        const date = new Date(dateStr);
        if (Number.isNaN(date.getTime())) return '';
        return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    } catch {
        return '';
    }
}

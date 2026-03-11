import { useNavigate } from '@tanstack/react-router';
import { useShip } from './hooks/workspace/WorkspaceContext';
import { Badge, Button } from '@ship/ui';
import { ExternalLink } from 'lucide-react';

export type EntityType = 'feature' | 'release' | 'adr' | 'note';

export interface EntityLinkInfo {
    type: string;
    target: string;
}

export function useEntityLink() {
    const navigate = useNavigate();
    const ship = useShip();

    const resolve = (link: EntityLinkInfo) => {
        const { type, target } = link;
        const lowerType = type.toLowerCase();

        if (lowerType === 'feature') {
            const entry = ship.features.find(f => f.file_name === target || f.id === target);
            if (entry) void ship.handleSelectFeature(entry);
            navigate({ to: '/project/features' });
        } else if (lowerType === 'release') {
            const entry = ship.releases.find(r => r.file_name === target || r.id === target);
            if (entry) void ship.handleSelectRelease(entry);
            navigate({ to: '/project/releases' });
        } else if (lowerType === 'adr') {
            const entry = ship.adrs.find(a => a.file_name === target || a.id === target);
            if (entry) ship.setSelectedAdr(entry);
            navigate({ to: '/project/adrs' });
        } else if (lowerType === 'note') {
            const entry = ship.notes.find(n => n.id === target);
            if (entry) void ship.handleSelectNote(entry);
            navigate({ to: '/project/notes' });
        }
    };

    return { resolve };
}

interface EntityLinkProps {
    link: EntityLinkInfo;
    onRemove?: () => void;
    className?: string;
}

export function EntityLink({ link, onRemove, className = "" }: EntityLinkProps) {
    const { resolve } = useEntityLink();

    return (
        <div className={`group flex items-center gap-2 rounded-md border bg-card/50 px-2 py-1 transition-colors hover:bg-accent/10 ${className}`}>
            <Badge variant="outline" className="h-4 px-1 text-[9px] uppercase tracking-wider opacity-70">
                {link.type}
            </Badge>
            <button
                type="button"
                onClick={() => resolve(link)}
                className="flex-1 truncate text-left text-xs font-medium hover:underline hover:text-primary"
            >
                {link.target}
            </button>
            <div className="flex items-center gap-1">
                <button
                    type="button"
                    onClick={() => resolve(link)}
                    className="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-accent rounded transition-all"
                >
                    <ExternalLink className="size-3 text-muted-foreground" />
                </button>
                {onRemove && (
                    <Button
                        variant="ghost"
                        size="xs"
                        className="h-5 w-5 p-0 opacity-0 group-hover:opacity-100 hover:text-destructive"
                        onClick={onRemove}
                    >
                        ×
                    </Button>
                )}
            </div>
        </div>
    );
}

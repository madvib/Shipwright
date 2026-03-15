import { Card, CardContent } from '@ui/components/ui/card';
import { Badge } from '@ui/components/ui/badge';

export function OutputMock() {
    return (
        <Card className="bg-ship-bg/80 border-ship-border/30 font-mono text-[11px] overflow-hidden relative group">
            <CardContent className="p-5 font-mono leading-relaxed relative">
                <div className="absolute top-0 right-0 p-2 opacity-20 group-hover:opacity-100 transition-opacity">
                    <Badge variant="secondary" className="bg-ship-green/10 text-ship-green text-[9px] border-ship-green/20 uppercase">Synced</Badge>
                </div>
                <div className="space-y-1">
                    <div className="text-ship-blue">const <span className="text-ship-text">analytics</span> = &#123;</div>
                    <div className="pl-8 text-ship-dim"> bus.<span className="text-ship-text">emit</span>(event, &#123;</div>
                    <div className="pl-12 text-ship-dim">timestamp: <span className="text-ship-green">Date</span>.now(),</div>
                    <div className="pl-12 text-ship-dim">source: <span className="text-ship-green">'ship_client'</span></div>
                    <div className="pl-8 text-ship-dim">&#125;);</div>
                    <div className="pl-4 text-ship-dim">&#125;</div>
                    <div className="text-ship-blue">&#125;;</div>
                </div>
            </CardContent>
        </Card>
    );
}

import { Card, CardHeader, CardTitle, CardContent } from '@ui/components/ui/card';
import { Badge } from '@ui/components/ui/badge';

export function IntentMock() {
    return (
        <div className="space-y-4">
            <Card size="sm" className="bg-ship-bg/50 border-ship-border/30 backdrop-blur-md">
                <CardHeader>
                    <div className="flex items-center gap-2">
                        <Badge variant="outline" className="border-ship-accent text-ship-accent text-[9px] h-4 px-1.5">FEATURE</Badge>
                        <CardTitle className="text-sm font-bold tracking-tighter uppercase text-ship-text">User Analytics</CardTitle>
                    </div>
                </CardHeader>
                <CardContent>
                    <p className="text-[11px] text-ship-dim font-light leading-snug">Track per-session event streams and synthesize into daily reports...</p>
                </CardContent>
            </Card>

            <Card size="sm" className="bg-ship-bg/50 border-ship-border/30 backdrop-blur-md">
                <CardHeader>
                    <div className="flex items-center gap-2">
                        <Badge variant="outline" className="border-ship-blue text-ship-blue text-[9px] h-4 px-1.5">ARCHITECTURE</Badge>
                        <CardTitle className="text-sm font-bold tracking-tighter uppercase text-ship-text">Event Bus</CardTitle>
                    </div>
                </CardHeader>
                <CardContent>
                    <p className="text-[11px] text-ship-dim font-light leading-snug">Implement a non-blocking message queue for real-time ingestion...</p>
                </CardContent>
            </Card>
        </div>
    );
}

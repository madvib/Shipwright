import { RefObject } from 'react';
import { TerminalSquare, RefreshCw, Maximize2, Minimize2, X, Info, Play, Square } from 'lucide-react';
import { Badge, Button, Select, SelectTrigger, SelectValue, SelectContent, SelectItem, Tooltip, TooltipTrigger, TooltipContent } from '@ship/ui';
import { cn } from '@/lib/utils';
import { type RuntimePerfSnapshot, type WorkspaceTerminalSessionInfo } from '@/lib/platform/tauri/commands';

interface WorkspaceTerminalTrayProps {
    terminalSession: WorkspaceTerminalSessionInfo | null;
    terminalProvider: 'codex' | 'claude' | 'gemini' | 'shell';
    onProviderChange: (val: 'codex' | 'claude' | 'gemini' | 'shell') => void;
    startingTerminal: boolean;
    stoppingTerminal: boolean;
    onStart: () => void;
    onStop: () => void;
    onMaximizedChange: (max: boolean) => void;
    maximized: boolean;
    height: number;
    onResizerMouseDown: (e: React.MouseEvent) => void;
    terminalContainerRef: RefObject<HTMLDivElement | null>;
    onSendSigInt: () => void;
    activationError?: string | null;
    runtimeError?: string | null;
    hasActiveSession: boolean;
    runtimePerf?: RuntimePerfSnapshot | null;
}

export function WorkspaceTerminalTray({
    terminalSession,
    terminalProvider,
    onProviderChange,
    startingTerminal,
    stoppingTerminal,
    onStart,
    onStop,
    onMaximizedChange,
    maximized,
    height,
    onResizerMouseDown,
    terminalContainerRef,
    onSendSigInt,
    activationError,
    runtimeError,
    hasActiveSession,
    runtimePerf,
}: WorkspaceTerminalTrayProps) {
    const resolvedHeight = Math.max(height, 140);
    return (
        <div
            className={cn(
                "absolute bottom-0 left-0 right-0 z-40 flex flex-col border-t border-border bg-card shadow-[0_-10px_40px_rgba(0,0,0,0.1)] dark:shadow-[0_-10px_40px_rgba(0,0,0,0.5)] backdrop-blur-xl",
                maximized ? "top-0 h-auto" : "transition-all duration-300 ease-in-out"
            )}
            style={!maximized ? { height: resolvedHeight } : undefined}
        >
            {/* Resize Handle */}
            {!maximized && (
                <div
                    className="absolute -top-1 left-0 right-0 z-50 h-2 cursor-ns-resize transition-colors hover:bg-primary/40"
                    onMouseDown={onResizerMouseDown}
                />
            )}

            {/* Tray Header */}
            <div className="flex h-11 shrink-0 items-center justify-between border-b border-border px-4 select-none">
                <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                        <TerminalSquare className="size-3.5 text-primary" />
                        <span className="text-[10px] font-semibold uppercase tracking-[0.12em] text-foreground">Runtime Console</span>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Info className="size-3 text-muted-foreground/30 hover:text-muted-foreground cursor-help transition-colors" />
                            </TooltipTrigger>
                            <TooltipContent side="top">A live terminal environment. AI agent sessions stream here, and you can also use it for manual shell tasks.</TooltipContent>
                        </Tooltip>
                    </div>
                    {terminalSession && (
                        <Badge variant="outline" className="h-4 border-status-green/30 bg-status-green/10 px-1.5 text-[8px] font-black text-status-green">
                            {terminalSession.provider.toUpperCase()} · READY
                        </Badge>
                    )}
                    {hasActiveSession && (
                        <Badge variant="outline" className="h-4 px-1.5 text-[8px] font-black uppercase tracking-wide">
                            session tracked
                        </Badge>
                    )}
                </div>

                <div className="flex items-center gap-2">
                    <div className="mr-2 flex items-center gap-1.5 rounded-md bg-muted p-1">
                        <Select
                            value={terminalProvider}
                            onValueChange={(value) => onProviderChange(value as any)}
                            disabled={Boolean(terminalSession)}
                        >
                            <SelectTrigger size="sm" className="h-6 w-28 border-none bg-transparent text-[9px] font-black uppercase text-muted-foreground hover:text-foreground transition-colors">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="codex">Codex-1</SelectItem>
                                <SelectItem value="claude">Claude-3</SelectItem>
                                <SelectItem value="gemini">Gemini-2</SelectItem>
                                <SelectItem value="shell">System Shell</SelectItem>
                            </SelectContent>
                        </Select>
                        {terminalSession ? (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button size="sm" variant="outline" className="h-6 gap-1 px-2 text-[10px]" onClick={onStop}>
                                        {stoppingTerminal ? (
                                            <RefreshCw className="size-3 animate-spin" />
                                        ) : (
                                            <Square className="size-3" />
                                        )}
                                        Stop Console
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent side="top">
                                    Stop terminal process only. Use End Session to close tracked workspace session.
                                </TooltipContent>
                            </Tooltip>
                        ) : (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button size="sm" className="h-6 gap-1 px-2 text-[10px]" onClick={onStart}>
                                        {startingTerminal ? (
                                            <RefreshCw className="size-3 animate-spin" />
                                        ) : (
                                            <Play className="size-3 fill-current" />
                                        )}
                                        Start
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent side="top">Start console (auto-starts session if needed)</TooltipContent>
                            </Tooltip>
                        )}
                    </div>
                    {activationError && (
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Badge
                                    variant="outline"
                                    className="h-5 border-amber-500/40 bg-amber-500/5 px-1.5 text-[8px] font-black uppercase tracking-wider text-amber-700"
                                >
                                    activation warning
                                </Badge>
                            </TooltipTrigger>
                            <TooltipContent side="top" className="max-w-sm">
                                {activationError}
                            </TooltipContent>
                        </Tooltip>
                    )}
                    {runtimeError && (
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Badge
                                    variant="outline"
                                    className="h-5 border-status-red/40 bg-status-red/5 px-1.5 text-[8px] font-semibold uppercase tracking-wide text-status-red"
                                >
                                    provider error
                                </Badge>
                            </TooltipTrigger>
                            <TooltipContent side="top" className="max-w-sm">
                                {runtimeError}
                            </TooltipContent>
                        </Tooltip>
                    )}

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button size="icon-xs" variant="ghost" className="size-6 text-muted-foreground hover:text-foreground" onClick={() => onMaximizedChange(!maximized)}>
                                {maximized ? <Minimize2 className="size-3" /> : <Maximize2 className="size-3" />}
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="top">{maximized ? 'Minimize' : 'Maximize'} Console</TooltipContent>
                    </Tooltip>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button size="icon-xs" variant="ghost" className="size-6 text-muted-foreground hover:text-foreground" onClick={onSendSigInt}>
                                <X className="size-3.5" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="top">Send SIGINT (Ctrl+C)</TooltipContent>
                    </Tooltip>
                </div>
            </div>

            {runtimePerf && (
                <div className="border-b border-border bg-muted/20 px-4 py-1 text-[10px] text-muted-foreground">
                    perf: start {runtimePerf.terminal_start_last_micros}us · read {runtimePerf.terminal_last_read_micros}us · write {runtimePerf.terminal_write_last_micros}us · resize {runtimePerf.terminal_resize_last_micros}us · watcher events {runtimePerf.watcher_fs_events} / flushes {runtimePerf.watcher_flushes}
                </div>
            )}

            {(activationError || runtimeError) && (
                <div className="border-b border-border bg-muted/20 px-4 py-2">
                    {activationError && (
                        <p className="text-[10px] text-amber-700">
                            activation warning: {activationError}
                        </p>
                    )}
                    {runtimeError && (
                        <p className="text-[10px] text-status-red">
                            provider status: {runtimeError}
                        </p>
                    )}
                </div>
            )}

            {/* Output Console */}
            <div className="relative flex-1 overflow-hidden">
                <div
                    ref={terminalContainerRef}
                    className="h-full w-full overflow-hidden bg-muted/10 dark:bg-black/20"
                />
                {!terminalSession && (
                    <div className="pointer-events-none absolute inset-0 grid place-items-center">
                        <span className="text-xs text-muted-foreground/60">
                            No runtime session. Choose a provider and click start.
                        </span>
                    </div>
                )}
            </div>
        </div>
    );
}

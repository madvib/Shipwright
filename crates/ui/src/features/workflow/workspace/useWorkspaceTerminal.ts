import { useCallback, useEffect, useRef, useState } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import 'xterm/css/xterm.css';
import {
    readWorkspaceTerminalCmd,
    resizeWorkspaceTerminalCmd,
    writeWorkspaceTerminalCmd,
    startWorkspaceTerminalCmd,
    stopWorkspaceTerminalCmd,
    type WorkspaceTerminalSessionInfo
} from '@/lib/platform/tauri/commands';

type TerminalProvider = 'codex' | 'claude' | 'gemini' | 'shell';

function isTerminalClosedError(error: string | null | undefined): boolean {
    const message = (error ?? '').toLowerCase();
    return message.includes('closed') || message.includes('not found');
}

export function useWorkspaceTerminal(
    branch: string | undefined,
    _activeModeId: string | null | undefined,
    _viewMode: 'command' | 'board'
) {
    const [terminalSession, setTerminalSession] = useState<WorkspaceTerminalSessionInfo | null>(null);
    const [terminalProvider, setTerminalProvider] = useState<TerminalProvider>('codex');
    const [startingTerminal, setStartingTerminal] = useState(false);
    const [stoppingTerminal, setStoppingTerminal] = useState(false);
    const [terminalInput, setTerminalInput] = useState('');
    const [runtimeError, setRuntimeError] = useState<string | null>(null);

    const xtermRef = useRef<Terminal | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const terminalBacklogRef = useRef('');
    const terminalContainerRef = useRef<HTMLDivElement | null>(null);

    const writeTerminalInput = useCallback(async (text: string) => {
        if (!terminalSession || !text) return;
        const result = await writeWorkspaceTerminalCmd(terminalSession.session_id, text);
        if (result.status === 'error') {
            if (isTerminalClosedError(result.error)) {
                setTerminalSession(null);
                return;
            }
            setRuntimeError(result.error || 'Failed to write to terminal session.');
        }
    }, [terminalSession]);

    const resizeActiveTerminal = useCallback(async () => {
        if (!terminalSession || !xtermRef.current || !fitAddonRef.current) return;
        fitAddonRef.current.fit();
        const cols = Math.max(xtermRef.current.cols, 20);
        const rows = Math.max(xtermRef.current.rows, 5);
        const result = await resizeWorkspaceTerminalCmd(terminalSession.session_id, cols, rows);
        if (result.status === 'error' && isTerminalClosedError(result.error)) {
            setTerminalSession(null);
        }
    }, [terminalSession]);

    const startWorkspaceTerminal = async (
        customProvider?: TerminalProvider
    ) => {
        if (!branch) return;
        setStartingTerminal(true);
        try {
            setRuntimeError(null);
            terminalBacklogRef.current = '';
            xtermRef.current?.reset();
            const result = await startWorkspaceTerminalCmd(
                branch,
                customProvider ?? terminalProvider,
                140,
                36
            );
            if (result.status === 'ok') {
                setTerminalSession(result.data);
                setTimeout(() => {
                    void resizeActiveTerminal();
                }, 0);
            } else {
                setRuntimeError(result.error || `Failed to start ${customProvider ?? terminalProvider} terminal.`);
            }
        } finally {
            setStartingTerminal(false);
        }
    };

    const stopWorkspaceTerminal = async () => {
        if (!terminalSession) return;
        setStoppingTerminal(true);
        try {
            await stopWorkspaceTerminalCmd(terminalSession.session_id);
            setTerminalSession(null);
            setRuntimeError(null);
        } finally {
            setStoppingTerminal(false);
        }
    };

    const sendTerminalInput = async (payload?: string) => {
        const text = payload ?? terminalInput;
        await writeTerminalInput(text);
    };

    // Create xterm once per mounted container
    useEffect(() => {
        if (!terminalContainerRef.current || xtermRef.current) return;
        const terminal = new Terminal({
            cursorBlink: true,
            convertEol: true,
            fontSize: 12,
            fontFamily:
                'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
            theme: {
                background: 'transparent',
            },
            scrollback: 5000,
        });
        const fitAddon = new FitAddon();
        terminal.loadAddon(fitAddon);
        terminal.open(terminalContainerRef.current);
        fitAddon.fit();
        if (terminalBacklogRef.current.length > 0) {
            terminal.write(terminalBacklogRef.current);
            terminalBacklogRef.current = '';
        }
        const disposable = terminal.onData((data) => {
            void writeTerminalInput(data);
        });
        const resizeObserver = new ResizeObserver(() => {
            void resizeActiveTerminal();
        });
        resizeObserver.observe(terminalContainerRef.current);
        xtermRef.current = terminal;
        fitAddonRef.current = fitAddon;

        return () => {
            resizeObserver.disconnect();
            disposable.dispose();
            terminal.dispose();
            xtermRef.current = null;
            fitAddonRef.current = null;
        };
    }, [resizeActiveTerminal, writeTerminalInput]);

    // Stream reader
    useEffect(() => {
        if (!terminalSession) return;
        let cancelled = false;
        const interval = window.setInterval(async () => {
            const result = await readWorkspaceTerminalCmd(terminalSession.session_id, 65_536);
            if (cancelled) return;
            if (result.status === 'ok' && result.data) {
                if (xtermRef.current) {
                    xtermRef.current.write(result.data);
                } else {
                    terminalBacklogRef.current += result.data;
                }
            } else if (result.status === 'error') {
                if (isTerminalClosedError(result.error)) {
                    setTerminalSession(null);
                } else {
                    setRuntimeError(result.error || 'Terminal stream failed.');
                }
            }
        }, 200);
        return () => {
            cancelled = true;
            window.clearInterval(interval);
        };
    }, [terminalSession?.session_id]);

    useEffect(() => {
        if (!terminalSession) return;
        if (branch && terminalSession.branch === branch) return;
        void stopWorkspaceTerminal();
    }, [branch, terminalSession?.branch]);

    useEffect(() => {
        if (!terminalSession) return;
        void resizeActiveTerminal();
    }, [terminalSession?.session_id, resizeActiveTerminal]);

    return {
        terminalSession,
        setTerminalSession,
        terminalProvider,
        setTerminalProvider,
        startingTerminal,
        stoppingTerminal,
        terminalInput,
        setTerminalInput,
        runtimeError,
        setRuntimeError,
        xtermRef,
        fitAddonRef,
        terminalBacklogRef,
        terminalContainerRef,
        startWorkspaceTerminal,
        stopWorkspaceTerminal,
        sendTerminalInput,
    };
}

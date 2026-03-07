import { type MouseEvent as ReactMouseEvent, useEffect, useMemo, useState } from 'react';
import { TerminalSquare, ChevronDown } from 'lucide-react';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '../dropdown-menu';

export interface WorkspaceEditorInfo {
    id: string;
    name: string;
    binary: string;
}

export function editorLogoCandidates(editorId: string, isDarkTheme: boolean): string[] {
    const byTheme = (dark: string, light: string) => (isDarkTheme ? dark : light);
    const fallbackTheme = (dark: string, light: string) => (isDarkTheme ? light : dark);
    switch (editorId) {
        case 'vscode':
            return ['/ide-logos/vscode.svg', '/ide-logos/vscode-alt.svg'];
        case 'cursor':
            return [
                byTheme('/ide-logos/CUBE_2D_DARK.svg', '/ide-logos/CUBE_2D_LIGHT.svg'),
                fallbackTheme('/ide-logos/CUBE_2D_DARK.svg', '/ide-logos/CUBE_2D_LIGHT.svg'),
            ];
        case 'windsurf':
            return [
                byTheme(
                    '/ide-logos/windsurf-white-symbol.svg',
                    '/ide-logos/windsurf-black-symbol.svg',
                ),
                fallbackTheme(
                    '/ide-logos/windsurf-white-symbol.svg',
                    '/ide-logos/windsurf-black-symbol.svg',
                ),
            ];
        case 'antigravity':
            return ['/ide-logos/Google-Antigravity-Icon-Full-Color.png'];
        case 'zed':
            return [
                byTheme('/ide-logos/zed-light.svg', '/ide-logos/zed-dark.svg'),
                fallbackTheme('/ide-logos/zed-light.svg', '/ide-logos/zed-dark.svg'),
            ];
        case 'intellij':
            return ['/ide-logos/IntelliJ_icon.svg'];
        case 'webstorm':
            return ['/ide-logos/WebStorm_icon.svg'];
        case 'pycharm':
            return ['/ide-logos/PyCharm_icon.svg'];
        case 'clion':
            return ['/ide-logos/CLion_icon.svg'];
        case 'goland':
            return ['/ide-logos/GoLand_icon.svg'];
        case 'rustrover':
            return ['/ide-logos/RustRover_icon.svg'];
        default:
            return [];
    }
}

export function EditorLogo({
    editorId,
    isDarkTheme,
    className,
}: {
    editorId: string;
    isDarkTheme: boolean;
    className?: string;
}) {
    const candidates = useMemo(
        () => editorLogoCandidates(editorId, isDarkTheme),
        [editorId, isDarkTheme],
    );
    const [logoIndex, setLogoIndex] = useState(0);

    useEffect(() => {
        setLogoIndex(0);
    }, [editorId, isDarkTheme]);

    if (candidates.length === 0 || logoIndex >= candidates.length) {
        return <TerminalSquare className={className} />;
    }

    return (
        <img
            src={candidates[logoIndex]}
            alt=""
            className={className}
            onError={() => setLogoIndex((current) => current + 1)}
        />
    );
}

export function EditorQuickOpenMenu({
    branch,
    editors,
    isDarkTheme,
    onOpenEditor,
}: {
    branch: string;
    editors: WorkspaceEditorInfo[];
    isDarkTheme: boolean;
    onOpenEditor: (branch: string, editorId: string) => Promise<void> | void;
}) {
    if (editors.length === 0) return null;

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                className="inline-flex h-5 items-center gap-0.5 rounded-sm px-1 text-muted-foreground hover:bg-muted/70 hover:text-foreground"
                onClick={(event: ReactMouseEvent<HTMLElement>) => event.stopPropagation()}
                title="Open in IDE"
            >
                <TerminalSquare className="size-3" />
                <ChevronDown className="size-2.5" />
            </DropdownMenuTrigger>
            <DropdownMenuContent
                align="end"
                onClick={(event: ReactMouseEvent<HTMLElement>) => event.stopPropagation()}
            >
                {editors.map((editor) => (
                    <DropdownMenuItem
                        key={editor.id}
                        className="gap-2"
                        onClick={(event: ReactMouseEvent<HTMLElement>) => {
                            event.stopPropagation();
                            void onOpenEditor(branch, editor.id);
                        }}
                    >
                        <EditorLogo
                            editorId={editor.id}
                            isDarkTheme={isDarkTheme}
                            className="size-3.5 rounded-sm"
                        />
                        {editor.name}
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

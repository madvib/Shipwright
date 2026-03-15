import * as React from "react";
import { cn } from "@/lib/utils";

// ─────────────────────────────────────────────────────────────────────────────
// SideBySide Layout — a v0-style full-screen two-panel layout with a
// chat interface on the left and a preview/canvas panel on the right.
// ─────────────────────────────────────────────────────────────────────────────

export interface SideBySideProps extends React.HTMLAttributes<HTMLDivElement> {
    /** Left panel content (e.g. chat messages + prompt input) */
    chat?: React.ReactNode;
    /** Right panel content (e.g. a web preview or code sandbox) */
    preview?: React.ReactNode;
    /** Width of the left chat panel. Defaults to 38% */
    chatWidth?: string;
    /** Header content shown across the top */
    header?: React.ReactNode;
}

export function SideBySide({
    chat,
    preview,
    chatWidth = "38%",
    header,
    className,
    ...props
}: SideBySideProps) {
    return (
        <div className={cn("flex h-full w-full flex-col overflow-hidden", className)} {...props}>
            {header && (
                <div className="flex shrink-0 items-center justify-between border-b bg-card/80 px-4 py-2 backdrop-blur-sm">
                    {header}
                </div>
            )}
            <div className="flex min-h-0 flex-1 overflow-hidden">
                {/* Chat panel */}
                <div
                    className="flex flex-col border-r bg-background"
                    style={{ width: chatWidth, minWidth: "300px", maxWidth: "50%" }}
                >
                    {chat}
                </div>
                {/* Preview panel */}
                <div className="flex flex-1 flex-col overflow-hidden bg-muted/20">
                    {preview}
                </div>
            </div>
        </div>
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Sub-components for the chat panel
// ─────────────────────────────────────────────────────────────────────────────

export interface SideBySideChatPanelProps extends React.HTMLAttributes<HTMLDivElement> { }

export function SideBySideChatPanel({ className, children, ...props }: SideBySideChatPanelProps) {
    return (
        <div className={cn("flex min-h-0 flex-1 flex-col", className)} {...props}>
            {children}
        </div>
    );
}

export interface SideBySideMessagesProps extends React.HTMLAttributes<HTMLDivElement> { }

export function SideBySideMessages({ className, children, ...props }: SideBySideMessagesProps) {
    return (
        <div
            className={cn(
                "flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4 scroll-smooth",
                className
            )}
            {...props}
        >
            {children}
        </div>
    );
}

export interface SideBySideInputBarProps extends React.HTMLAttributes<HTMLDivElement> { }

export function SideBySideInputBar({ className, children, ...props }: SideBySideInputBarProps) {
    return (
        <div
            className={cn(
                "flex shrink-0 flex-col gap-2 border-t bg-card/60 p-3 backdrop-blur-sm",
                className
            )}
            {...props}
        >
            {children}
        </div>
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Sub-components for the preview panel
// ─────────────────────────────────────────────────────────────────────────────

export interface SideBySidePreviewPanelProps extends React.HTMLAttributes<HTMLDivElement> { }

export function SideBySidePreviewPanel({
    className,
    children,
    ...props
}: SideBySidePreviewPanelProps) {
    return (
        <div className={cn("relative flex min-h-0 flex-1 flex-col", className)} {...props}>
            {children}
        </div>
    );
}

export interface SideBySidePreviewHeaderProps extends React.HTMLAttributes<HTMLDivElement> { }

export function SideBySidePreviewHeader({
    className,
    children,
    ...props
}: SideBySidePreviewHeaderProps) {
    return (
        <div
            className={cn(
                "flex shrink-0 items-center justify-between border-b bg-card/60 px-3 py-2 backdrop-blur-sm",
                className
            )}
            {...props}
        >
            {children}
        </div>
    );
}

export interface SideBySidePreviewContentProps extends React.HTMLAttributes<HTMLDivElement> { }

export function SideBySidePreviewContent({
    className,
    children,
    ...props
}: SideBySidePreviewContentProps) {
    return (
        <div className={cn("min-h-0 flex-1 overflow-auto", className)} {...props}>
            {children}
        </div>
    );
}

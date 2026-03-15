import { createFileRoute } from '@tanstack/react-router';
import { useState, useRef } from 'react';
import { Button } from '@ship/primitives';
import { SideBySide, SideBySideChatPanel, SideBySideMessages, SideBySideInputBar, SideBySidePreviewPanel, SideBySidePreviewHeader, SideBySidePreviewContent } from '@ship/primitives';
import { Textarea } from '@ship/primitives';
import { Bot, Send, RefreshCw, Code2, Eye } from 'lucide-react';

interface ChatMessage {
    id: string;
    role: 'user' | 'assistant';
    content: string;
}

function AgentV0RouteComponent() {
    const [messages, setMessages] = useState<ChatMessage[]>([
        {
            id: '1',
            role: 'assistant',
            content: 'Hello! I\'m your Ship AI assistant. Describe what you\'d like to build and I\'ll help you generate it.',
        },
    ]);
    const [input, setInput] = useState('');
    const [activeTab, setActiveTab] = useState<'preview' | 'code'>('preview');
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const handleSubmit = () => {
        if (!input.trim()) return;
        const userMsg: ChatMessage = { id: Date.now().toString(), role: 'user', content: input };
        const assistantMsg: ChatMessage = {
            id: (Date.now() + 1).toString(),
            role: 'assistant',
            content: `I received your request: "${input}". This is a demo — connect an AI provider to enable real responses.`,
        };
        setMessages((prev) => [...prev, userMsg, assistantMsg]);
        setInput('');
        setTimeout(() => messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' }), 50);
    };

    return (
        <SideBySide
            className="h-full"
            header={
                <>
                    <div className="flex items-center gap-2">
                        <div className="flex size-6 items-center justify-center rounded-md bg-primary/15">
                            <Bot className="size-3.5 text-primary" />
                        </div>
                        <span className="text-sm font-semibold">Ship AI Studio</span>
                        <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary">
                            v0 Preview
                        </span>
                    </div>
                    <div className="flex items-center gap-2">
                        <Button variant="ghost" size="xs">
                            <RefreshCw className="mr-1.5 size-3" />
                            Reset
                        </Button>
                    </div>
                </>
            }
            chat={
                <SideBySideChatPanel>
                    <SideBySideMessages>
                        {messages.map((msg) => (
                            <div
                                key={msg.id}
                                className={`flex gap-3 ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
                            >
                                {msg.role === 'assistant' && (
                                    <div className="flex size-7 shrink-0 items-center justify-center rounded-full bg-primary/15 text-primary">
                                        <Bot className="size-4" />
                                    </div>
                                )}
                                <div
                                    className={`max-w-[80%] rounded-xl px-3.5 py-2.5 text-sm leading-relaxed ${msg.role === 'user'
                                            ? 'bg-primary text-primary-foreground'
                                            : 'bg-muted/60 text-foreground'
                                        }`}
                                >
                                    {msg.content}
                                </div>
                            </div>
                        ))}
                        <div ref={messagesEndRef} />
                    </SideBySideMessages>

                    <SideBySideInputBar>
                        <Textarea
                            placeholder="Describe what you want to build..."
                            className="min-h-[80px] resize-none border-0 bg-transparent text-sm focus-visible:ring-0"
                            value={input}
                            onChange={(e) => setInput(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter' && !e.shiftKey) {
                                    e.preventDefault();
                                    handleSubmit();
                                }
                            }}
                        />
                        <div className="flex items-center justify-between">
                            <p className="text-[11px] text-muted-foreground">⏎ to send · Shift+⏎ for newline</p>
                            <Button size="sm" onClick={handleSubmit} disabled={!input.trim()}>
                                <Send className="mr-1.5 size-3.5" />
                                Send
                            </Button>
                        </div>
                    </SideBySideInputBar>
                </SideBySideChatPanel>
            }
            preview={
                <SideBySidePreviewPanel>
                    <SideBySidePreviewHeader>
                        <div className="flex items-center gap-1">
                            <button
                                onClick={() => setActiveTab('preview')}
                                className={`flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${activeTab === 'preview'
                                        ? 'bg-primary/10 text-primary'
                                        : 'text-muted-foreground hover:text-foreground'
                                    }`}
                            >
                                <Eye className="size-3.5" />
                                Preview
                            </button>
                            <button
                                onClick={() => setActiveTab('code')}
                                className={`flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${activeTab === 'code'
                                        ? 'bg-primary/10 text-primary'
                                        : 'text-muted-foreground hover:text-foreground'
                                    }`}
                            >
                                <Code2 className="size-3.5" />
                                Code
                            </button>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="h-2 w-2 rounded-full bg-status-green animate-pulse" />
                            <span className="text-[11px] text-muted-foreground">Ready</span>
                        </div>
                    </SideBySidePreviewHeader>

                    <SideBySidePreviewContent>
                        {activeTab === 'preview' ? (
                            <div className="flex h-full items-center justify-center">
                                <div className="text-center">
                                    <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-2xl border bg-card shadow-sm">
                                        <Eye className="size-6 text-muted-foreground" />
                                    </div>
                                    <p className="text-sm font-medium">No preview yet</p>
                                    <p className="mt-1 text-xs text-muted-foreground">
                                        Ask the AI to generate something and it'll appear here.
                                    </p>
                                </div>
                            </div>
                        ) : (
                            <div className="h-full bg-[oklch(0.14_0.004_49)] p-4">
                                <pre className="text-xs leading-relaxed text-[oklch(0.9_0.02_100)]">
                                    {`// Generated code will appear here\n// Ask the AI to build something above`}
                                </pre>
                            </div>
                        )}
                    </SideBySidePreviewContent>
                </SideBySidePreviewPanel>
            }
        />
    );
}

export const Route = createFileRoute('/project/agents/v0')({
    component: AgentV0RouteComponent,
});

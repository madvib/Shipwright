import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    Textarea,
    Button,
} from '@ship/primitives';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface AdrContextDialogProps {
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
    context: string;
    onContextChange?: (next: string) => void;
    isEditing: boolean;
}

export function AdrContextDialog({
    isOpen,
    onOpenChange,
    context,
    onContextChange,
    isEditing,
}: AdrContextDialogProps) {
    return (
        <Dialog open={isOpen} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle>Decision Context</DialogTitle>
                    <DialogDescription>
                        Background, constraints, and drivers for this architectural decision.
                    </DialogDescription>
                </DialogHeader>
                <div className="mt-4 min-h-[300px]">
                    {isEditing && onContextChange ? (
                        <Textarea
                            value={context}
                            onChange={(e) => onContextChange(e.target.value)}
                            placeholder="Capture constraints, drivers, and background..."
                            className="min-h-[300px] text-sm"
                        />
                    ) : (
                        <div className="ship-markdown-preview rounded-md border bg-muted/20 p-4 overflow-auto max-h-[60vh]">
                            {context.trim() ? (
                                <ReactMarkdown remarkPlugins={[remarkGfm]}>{context}</ReactMarkdown>
                            ) : (
                                <p className="text-muted-foreground text-sm italic">No context documented yet.</p>
                            )}
                        </div>
                    )}
                </div>
                <div className="mt-4 flex justify-end">
                    <Button onClick={() => onOpenChange(false)}>Close</Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}

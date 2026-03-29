import { Sparkles, Wand2, Type, AlignLeft, CheckCircle } from 'lucide-react';
import { Button } from '../button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '../dropdown-menu';
import { Tooltip, TooltipContent, TooltipTrigger } from '../tooltip';

type AiAction = 'polish' | 'shorten' | 'expand' | 'fix_grammar';

const INSTRUCTION_MAP: Record<AiAction, string> = {
    polish: 'Polish the writing to be more professional and clear',
    shorten: 'Make the text more concise and remove jargon',
    expand: 'Add more relevant details and context',
    fix_grammar: 'Fix any grammar or spelling issues',
};

interface AiActionsMenuProps {
    disabled: boolean;
    onAction: (action: AiAction) => void;
}

export { INSTRUCTION_MAP };
export type { AiAction };

export function AiActionsMenu({ disabled, onAction }: AiActionsMenuProps) {
    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <DropdownMenu>
                    <DropdownMenuTrigger render={
                        <Button variant="outline" size="xs" disabled={disabled}>
                            <Wand2 className="size-3.5" />
                            Create with AI
                        </Button>
                    } />
                    <DropdownMenuContent align="end" className="w-56 p-1.5 shadow-xl">
                        <DropdownMenuGroup>
                            <DropdownMenuLabel className="px-2 pb-2 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
                                Transform Text
                            </DropdownMenuLabel>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator className="opacity-50" />
                        <div className="space-y-0.5">
                            <DropdownMenuItem onClick={() => onAction('polish')} className="flex items-center gap-2 rounded-md">
                                <Sparkles className="size-3.5 text-amber-500" />
                                <span className="text-sm">Polish Writing</span>
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => onAction('shorten')} className="flex items-center gap-2 rounded-md">
                                <AlignLeft className="size-3.5 text-blue-500" />
                                <span className="text-sm">Make Concise</span>
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => onAction('expand')} className="flex items-center gap-2 rounded-md">
                                <Type className="size-3.5 text-indigo-500" />
                                <span className="text-sm">Expand Details</span>
                            </DropdownMenuItem>
                            <DropdownMenuSeparator className="opacity-50" />
                            <DropdownMenuItem onClick={() => onAction('fix_grammar')} className="flex items-center gap-2 rounded-md">
                                <CheckCircle className="size-3.5 text-emerald-500" />
                                <span className="text-sm">Fix Grammar</span>
                            </DropdownMenuItem>
                        </div>
                    </DropdownMenuContent>
                </DropdownMenu>
            </TooltipTrigger>
            <TooltipContent>Refine, polish, or transform your text using AI.</TooltipContent>
        </Tooltip>
    );
}

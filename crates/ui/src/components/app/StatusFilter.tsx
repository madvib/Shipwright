import { Check, Filter } from 'lucide-react';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Popover, PopoverContent, PopoverTrigger } from '@ship/ui';
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator } from '@ship/ui';
import { Separator } from '@ship/ui';
import { cn } from '@/lib/utils';

interface StatusOption {
    value: string;
    label: string;
    icon?: React.ComponentType<{ className?: string }>;
}

interface StatusFilterProps {
    options: StatusOption[];
    selectedValues: Set<string>;
    onSelect: (values: Set<string>) => void;
    label?: string;
    className?: string;
}

export function StatusFilter({
    options,
    selectedValues,
    onSelect,
    label = 'Status',
    className,
}: StatusFilterProps) {
    return (
        <Popover>
            <PopoverTrigger>
                <Button variant="outline" size="sm" className={cn('h-8 border-dashed', className)}>
                    <Filter className="mr-2 size-4" />
                    {label}
                    {selectedValues.size > 0 && (
                        <>
                            <Separator orientation="vertical" className="mx-2 h-4" />
                            <Badge variant="secondary" className="rounded-sm px-1 font-normal lg:hidden">
                                {selectedValues.size}
                            </Badge>
                            <div className="hidden space-x-1 lg:flex">
                                {selectedValues.size > 2 ? (
                                    <Badge variant="secondary" className="rounded-sm px-1 font-normal">
                                        {selectedValues.size} selected
                                    </Badge>
                                ) : (
                                    options
                                        .filter((option) => selectedValues.has(option.value))
                                        .map((option) => (
                                            <Badge variant="secondary" key={option.value} className="rounded-sm px-1 font-normal">
                                                {option.label}
                                            </Badge>
                                        ))
                                )}
                            </div>
                        </>
                    )}
                </Button>
            </PopoverTrigger>
            <PopoverContent className="w-[200px] p-0" align="start">
                <Command>
                    <CommandInput placeholder={`Filter ${label.toLowerCase()}...`} />
                    <CommandList>
                        <CommandEmpty>No results found.</CommandEmpty>
                        <CommandGroup>
                            {options.map((option) => {
                                const isSelected = selectedValues.has(option.value);
                                return (
                                    <CommandItem
                                        key={option.value}
                                        onSelect={() => {
                                            const next = new Set(selectedValues);
                                            if (isSelected) {
                                                next.delete(option.value);
                                            } else {
                                                next.add(option.value);
                                            }
                                            onSelect(next);
                                        }}
                                    >
                                        <div
                                            className={cn(
                                                'mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary',
                                                isSelected ? 'bg-primary text-primary-foreground' : 'opacity-50 [&_svg]:invisible'
                                            )}
                                        >
                                            <Check className="size-4" />
                                        </div>
                                        {option.icon && <option.icon className="mr-2 size-4 text-muted-foreground" />}
                                        <span>{option.label}</span>
                                    </CommandItem>
                                );
                            })}
                        </CommandGroup>
                        {selectedValues.size > 0 && (
                            <>
                                <CommandSeparator />
                                <CommandGroup>
                                    <CommandItem onSelect={() => onSelect(new Set())} className="justify-center text-center">
                                        Clear filters
                                    </CommandItem>
                                </CommandGroup>
                            </>
                        )}
                    </CommandList>
                </Command>
            </PopoverContent>
        </Popover>
    );
}

import * as React from "react"
import { Check, PlusCircle } from "lucide-react"

import { cn } from "@/lib/utils"
import { Badge } from './badge'
import { Button } from './button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from './command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from './popover'
import { Separator } from './separator'

interface FilterOption {
  label: string
  value: string
  icon?: React.ComponentType<{ className?: string }>
}

interface FacetedFilterProps {
  title: string
  options: FilterOption[]
  selectedValues: string[]
  onSelectionChange: (values: string[]) => void
  facets?: Map<string, number>
  maxDisplay?: number
  allowNew?: boolean
  onAddNew?: (value: string) => void
}

export function FacetedFilter({
  title,
  options,
  selectedValues,
  onSelectionChange,
  facets,
  maxDisplay = 2,
  allowNew = false,
  onAddNew,
}: FacetedFilterProps) {
  const selectedSet = React.useMemo(
    () => new Set(selectedValues),
    [selectedValues]
  )
  const [inputValue, setInputValue] = React.useState('')

  const handleSelect = (value: string) => {
    const newSelected = new Set(selectedSet)
    if (newSelected.has(value)) {
      newSelected.delete(value)
    } else {
      newSelected.add(value)
    }
    onSelectionChange(Array.from(newSelected))
  }

  const handleClear = () => {
    onSelectionChange([])
  }

  const handleAddNew = () => {
    const clean = inputValue.trim()
    if (!clean || selectedSet.has(clean)) return
    if (allowNew && onAddNew) {
      onAddNew(clean)
      setInputValue('')
    }
  }

  const filteredOptions = options.filter(
    (option) =>
      option.label.toLowerCase().includes(inputValue.toLowerCase()) &&
      !selectedSet.has(option.value)
  )

  const showAddNew = allowNew && inputValue.trim() && !options.some((o) => o.value === inputValue.trim())

  return (
    <Popover>
      <PopoverTrigger render={
        <Button variant="outline" size="sm" className="h-8 border-dashed">
          <PlusCircle className="mr-2 h-4 w-4" />
          {title}
          {selectedSet.size > 0 && (
            <>
              <Separator orientation="vertical" className="mx-2 h-4" />
              <Badge
                variant="secondary"
                className="rounded-sm px-1 font-normal lg:hidden"
              >
                {selectedSet.size}
              </Badge>
              <div className="hidden space-x-1 lg:flex">
                {selectedSet.size > maxDisplay ? (
                  <Badge
                    variant="secondary"
                    className="rounded-sm px-1 font-normal"
                  >
                    {selectedSet.size} selected
                  </Badge>
                ) : (
                  options
                    .filter((option) => selectedSet.has(option.value))
                    .map((option) => (
                      <Badge
                        variant="secondary"
                        key={option.value}
                        className="rounded-sm px-1 font-normal"
                      >
                        {option.label}
                      </Badge>
                    ))
                )}
              </div>
            </>
          )}
        </Button>
      } />
      <PopoverContent className="w-[200px] p-0" align="start">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={title}
            value={inputValue}
            onValueChange={setInputValue}
          />
          <CommandList>
            <CommandEmpty>
              {showAddNew ? null : 'No results found.'}
            </CommandEmpty>
            <CommandGroup>
              {showAddNew && (
                <CommandItem onSelect={handleAddNew}>
                  <div className="mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary bg-primary text-primary-foreground">
                    <PlusCircle className="h-3 w-3" />
                  </div>
                  <span>Add &quot;{inputValue.trim()}&quot;</span>
                </CommandItem>
              )}
              {filteredOptions.map((option) => {
                const isSelected = selectedSet.has(option.value)
                return (
                  <CommandItem
                    key={option.value}
                    onSelect={() => handleSelect(option.value)}
                  >
                    <div
                      className={cn(
                        "mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary",
                        isSelected
                          ? "bg-primary text-primary-foreground"
                          : "opacity-50 [&_svg]:invisible"
                      )}
                    >
                      <Check className="h-4 w-4" />
                    </div>
                    {option.icon && (
                      <option.icon className="mr-2 h-4 w-4 text-muted-foreground" />
                    )}
                    <span>{option.label}</span>
                    {facets?.get(option.value) && (
                      <span className="ml-auto flex h-4 w-4 items-center justify-center font-mono text-xs">
                        {facets.get(option.value)}
                      </span>
                    )}
                  </CommandItem>
                )
              })}
            </CommandGroup>
            {selectedSet.size > 0 && (
              <>
                <CommandSeparator />
                <CommandGroup>
                  <CommandItem
                    onSelect={handleClear}
                    className="justify-center text-center"
                  >
                    Clear filters
                  </CommandItem>
                </CommandGroup>
              </>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

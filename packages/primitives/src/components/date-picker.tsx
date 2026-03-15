import { format } from "date-fns"
import { CalendarIcon } from "lucide-react"

import { cn } from "@/lib/utils"
import { Button } from './button'
import { Calendar } from './calendar'
import { Popover, PopoverContent, PopoverTrigger } from './popover'

interface DatePickerProps {
  value: string
  onValueChange: (next: string) => void
  className?: string
}

function parseIsoDate(value: string): Date | undefined {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})$/)
  if (!match) return undefined

  const year = Number(match[1])
  const month = Number(match[2])
  const day = Number(match[3])
  const parsed = new Date(year, month - 1, day)
  if (Number.isNaN(parsed.getTime())) return undefined
  if (parsed.getFullYear() !== year || parsed.getMonth() !== month - 1 || parsed.getDate() !== day) {
    return undefined
  }
  return parsed
}

export function DatePicker({ value, onValueChange, className }: DatePickerProps) {
  const selected = parseIsoDate(value)

  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            size="xs"
            variant="outline"
            className={cn(
              "h-7 w-[168px] justify-start px-2 text-left text-xs font-normal",
              !selected && "text-muted-foreground",
              className
            )}
          />
        }
      >
        <CalendarIcon className="size-3.5 shrink-0" />
        {selected ? format(selected, "PPP") : <span>Pick a date</span>}
      </PopoverTrigger>
      <PopoverContent align="start" className="w-auto p-0">
        <Calendar
          mode="single"
          selected={selected}
          captionLayout="dropdown"
          onSelect={(next) => {
            if (!next) return
            onValueChange(format(next, "yyyy-MM-dd"))
          }}
        />
      </PopoverContent>
    </Popover>
  )
}

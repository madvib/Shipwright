// Editable TODO checklist backed by .ship-session/todo.md.
// Each line starting with "- [ ]" or "- [x]" is a checklist item.

import { useState, useCallback } from 'react'
import { Plus, ListTodo } from 'lucide-react'
import { useSessionTodo } from './useSessionFiles'

interface TodoItem {
  text: string
  checked: boolean
}

function parseTodoItems(content: string): TodoItem[] {
  return content
    .split('\n')
    .filter((line) => /^- \[[ x]\] /.test(line))
    .map((line) => ({
      checked: line.startsWith('- [x] '),
      text: line.replace(/^- \[[ x]\] /, ''),
    }))
}

function serializeTodoItems(items: TodoItem[]): string {
  return items.map((item) => `- [${item.checked ? 'x' : ' '}] ${item.text}`).join('\n') + '\n'
}

export function SessionTodo() {
  const { content, exists, isLoading, writeTodo, isSaving } = useSessionTodo()
  const [newItem, setNewItem] = useState('')

  const items = content ? parseTodoItems(content) : []

  const handleToggle = useCallback(
    (index: number) => {
      const updated = items.map((item, i) =>
        i === index ? { ...item, checked: !item.checked } : item,
      )
      writeTodo(serializeTodoItems(updated))
    },
    [items, writeTodo],
  )

  const handleAdd = useCallback(() => {
    const text = newItem.trim()
    if (!text) return
    const updated = [...items, { text, checked: false }]
    writeTodo(serializeTodoItems(updated))
    setNewItem('')
  }, [items, newItem, writeTodo])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault()
        handleAdd()
      }
    },
    [handleAdd],
  )

  if (isLoading) {
    return (
      <div className="px-1 py-2 space-y-1.5">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-6 rounded bg-muted animate-pulse" />
        ))}
      </div>
    )
  }

  if (!exists) {
    return (
      <button
        onClick={() => writeTodo('- [ ] \n')}
        disabled={isSaving}
        className="flex items-center gap-1.5 px-2 py-1.5 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded-md transition w-full"
      >
        <ListTodo className="size-3" />
        Add a TODO
      </button>
    )
  }

  return (
    <div className="space-y-1">
      {items.map((item, i) => (
        <label
          key={i}
          className="flex items-start gap-2 px-1.5 py-1 rounded hover:bg-muted/30 cursor-pointer group"
        >
          <input
            type="checkbox"
            checked={item.checked}
            onChange={() => handleToggle(i)}
            className="mt-0.5 size-3.5 rounded border-border accent-primary cursor-pointer"
          />
          <span
            className={`text-[11px] leading-tight ${
              item.checked
                ? 'line-through text-muted-foreground/60'
                : 'text-foreground'
            }`}
          >
            {item.text}
          </span>
        </label>
      ))}

      <div className="flex items-center gap-1 mt-1">
        <input
          type="text"
          value={newItem}
          onChange={(e) => setNewItem(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Add item..."
          className="flex-1 text-[11px] bg-transparent border-0 outline-none placeholder:text-muted-foreground/50 text-foreground px-1.5 py-1"
        />
        <button
          onClick={handleAdd}
          disabled={!newItem.trim() || isSaving}
          className="shrink-0 p-0.5 rounded text-muted-foreground hover:text-foreground disabled:opacity-30 transition"
          aria-label="Add TODO item"
        >
          <Plus className="size-3" />
        </button>
      </div>
    </div>
  )
}

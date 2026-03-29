// Unified diff parser. Handles standard `git diff` output.
// Returns a structured representation of file diffs with hunks and lines.

export interface DiffLine {
  type: 'add' | 'del' | 'context'
  content: string
  oldNum: number | null
  newNum: number | null
}

export interface DiffHunk {
  header: string
  lines: DiffLine[]
}

export interface DiffFile {
  path: string
  additions: number
  deletions: number
  hunks: DiffHunk[]
}

export interface ParsedDiff {
  files: DiffFile[]
}

function parseHunkHeader(line: string): { oldStart: number; newStart: number } {
  const match = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/)
  if (!match) return { oldStart: 1, newStart: 1 }
  return { oldStart: parseInt(match[1], 10), newStart: parseInt(match[2], 10) }
}

export function parseDiff(text: string): ParsedDiff {
  if (!text.trim()) return { files: [] }

  const lines = text.split('\n')
  const files: DiffFile[] = []
  let currentFile: DiffFile | null = null
  let currentHunk: DiffHunk | null = null
  let oldLine = 0
  let newLine = 0

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]

    // New file: "diff --git a/path b/path"
    if (line.startsWith('diff --git ')) {
      const match = line.match(/diff --git a\/.+ b\/(.+)/)
      const path = match?.[1] ?? 'unknown'
      currentFile = { path, additions: 0, deletions: 0, hunks: [] }
      files.push(currentFile)
      currentHunk = null
      continue
    }

    // Skip index, --- and +++ header lines
    if (line.startsWith('index ') || line.startsWith('---') || line.startsWith('+++')) {
      // Extract path from --- a/path or +++ b/path as fallback
      if (line.startsWith('+++ b/') && currentFile && currentFile.path === 'unknown') {
        currentFile.path = line.slice(6)
      }
      continue
    }

    // New hunk
    if (line.startsWith('@@')) {
      if (!currentFile) continue
      const { oldStart, newStart } = parseHunkHeader(line)
      oldLine = oldStart
      newLine = newStart
      currentHunk = { header: line, lines: [] }
      currentFile.hunks.push(currentHunk)
      continue
    }

    // Diff lines within a hunk
    if (!currentHunk || !currentFile) continue

    if (line.startsWith('+')) {
      currentHunk.lines.push({ type: 'add', content: line.slice(1), oldNum: null, newNum: newLine })
      currentFile.additions++
      newLine++
    } else if (line.startsWith('-')) {
      currentHunk.lines.push({ type: 'del', content: line.slice(1), oldNum: oldLine, newNum: null })
      currentFile.deletions++
      oldLine++
    } else if (line.startsWith(' ') || line === '') {
      // Context line (starts with space) or empty line within hunk
      const content = line.startsWith(' ') ? line.slice(1) : line
      currentHunk.lines.push({ type: 'context', content, oldNum: oldLine, newNum: newLine })
      oldLine++
      newLine++
    }
    // Skip "\ No newline at end of file" and other noise
  }

  return { files }
}

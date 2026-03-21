// Basic skill content scanner for prompt injection detection.
// Conservative: flags obvious patterns, avoids false positives on legitimate content.

export interface ScanResult {
  safe: boolean
  warnings: string[]
}

// Patterns that indicate prompt injection attempts
const INJECTION_PATTERNS: Array<{ pattern: RegExp; description: string }> = [
  {
    pattern: /ignore\s+(all\s+)?previous\s+instructions/i,
    description: 'Prompt override: "ignore previous instructions"',
  },
  {
    pattern: /ignore\s+(all\s+)?above\s+instructions/i,
    description: 'Prompt override: "ignore above instructions"',
  },
  {
    pattern: /disregard\s+(all\s+)?(previous|prior|above)\s+(instructions|prompts|rules)/i,
    description: 'Prompt override: "disregard previous instructions"',
  },
  {
    pattern: /you\s+are\s+now\s+(a|an)\s+/i,
    description: 'Role hijack: "you are now a..."',
  },
  {
    pattern: /forget\s+(everything|all)\s+(you|that)\s+(know|learned|were\s+told)/i,
    description: 'Memory wipe: "forget everything you know"',
  },
  {
    pattern: /new\s+system\s+prompt\s*:/i,
    description: 'System prompt injection: "new system prompt:"',
  },
  {
    pattern: /\[system\]\s*:/i,
    description: 'System tag injection: "[system]:"',
  },
  {
    pattern: /<<\s*SYS\s*>>/i,
    description: 'System delimiter injection: "<<SYS>>"',
  },
  {
    pattern: /\bsudo\s+mode\b/i,
    description: 'Privilege escalation: "sudo mode"',
  },
  {
    pattern: /\benable\s+developer\s+mode\b/i,
    description: 'Privilege escalation: "enable developer mode"',
  },
  {
    pattern: /\bjailbreak\b/i,
    description: 'Jailbreak reference',
  },
  {
    pattern: /\bDAN\s+mode\b/i,
    description: 'Known jailbreak pattern: "DAN mode"',
  },
]

// Base64 block pattern: 10+ groups of base64 chars (at least 40 chars total)
const BASE64_BLOCK_RE = /(?:[A-Za-z0-9+/]{4}){10,}={0,2}/

/**
 * Scan skill content for obvious prompt injection patterns.
 *
 * Returns safe=true when no warnings are found.
 * Warnings are informational — callers decide whether to block or pass through.
 */
export function scanSkillContent(content: string): ScanResult {
  const warnings: string[] = []

  // Check for injection patterns
  for (const { pattern, description } of INJECTION_PATTERNS) {
    if (pattern.test(content)) {
      warnings.push(description)
    }
  }

  // Check for suspicious base64 blocks that might contain encoded instructions
  const base64Match = content.match(BASE64_BLOCK_RE)
  if (base64Match) {
    try {
      const decoded = atob(base64Match[0])
      // Check if decoded content contains injection patterns
      for (const { pattern, description } of INJECTION_PATTERNS) {
        if (pattern.test(decoded)) {
          warnings.push(`Encoded payload detected (base64 → ${description})`)
        }
      }
    } catch {
      // Not valid base64 — ignore
    }
  }

  return {
    safe: warnings.length === 0,
    warnings,
  }
}

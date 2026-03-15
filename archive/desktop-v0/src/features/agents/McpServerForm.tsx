import { Info, Plus, Trash2 } from 'lucide-react';
import type { McpServerConfig, McpServerType } from '@/bindings';
import { Button } from '@ship/primitives';
import { Input } from '@ship/primitives';
import { Label } from '@ship/primitives';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { AutocompleteInput } from '@ship/primitives';
import { cn } from '@/lib/utils';
import { getMcpTemplateValidation, MCP_STDIO_ONLY_ALPHA, slugifyId, splitShellArgs } from './agents.types';

// ── McpServerForm ────────────────────────────────────────────────────────────

export function McpServerForm({
  draft,
  onChange,
  onSave,
  onCancel,
  idOptions,
  commandOptions,
  envKeyOptions,
  isNew,
}: {
  draft: McpServerConfig;
  onChange: (server: McpServerConfig) => void;
  onSave: () => void;
  onCancel: () => void;
  idOptions: Array<{ value: string; label?: string; keywords?: string[] }>;
  commandOptions: Array<{ value: string; label?: string; keywords?: string[] }>;
  envKeyOptions: Array<{ value: string; label?: string; keywords?: string[] }>;
  isNew?: boolean;
}) {
  const transport = draft.server_type ?? 'stdio';
  const argsStr = (draft.args ?? []).join(' ');
  const validations = getMcpTemplateValidation(draft);
  const setField = <K extends keyof McpServerConfig>(key: K, value: McpServerConfig[K]) =>
    onChange({ ...draft, [key]: value });

  return (
    <div className="border-t bg-muted/20 px-4 py-4 space-y-3">
      <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
        {isNew ? 'New MCP Server' : 'Edit Server'}
      </p>

      <div className="grid gap-3 sm:grid-cols-[1fr_1fr_auto]">
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Name</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Display name shown in UI and provider config exports.</TooltipContent>
            </Tooltip>
          </div>
          <Input
            value={draft.name}
            onChange={(e) => setField('name', e.target.value)}
            placeholder="e.g. ship"
            className="h-8 text-xs"
            autoFocus={isNew}
          />
        </div>
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Server ID</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Stable slug used in permissions and provider exports.</TooltipContent>
            </Tooltip>
          </div>
          <AutocompleteInput
            value={draft.id ?? ''}
            options={idOptions}
            placeholder={slugifyId(draft.name || 'server-id') || 'server-id'}
            onValueChange={(value) => setField('id', value)}
            className="h-8 text-xs font-mono"
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
          />
        </div>
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Transport</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>
                {MCP_STDIO_ONLY_ALPHA
                  ? 'Alpha currently supports stdio MCP servers only.'
                  : 'How Ship connects to this MCP server: local process, SSE, or HTTP.'}
              </TooltipContent>
            </Tooltip>
          </div>
          <Select value={transport} onValueChange={(v) => setField('server_type', v as McpServerType)}>
            <SelectTrigger size="sm" className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="stdio">stdio</SelectItem>
              <SelectItem value="sse" disabled={MCP_STDIO_ONLY_ALPHA}>SSE</SelectItem>
              <SelectItem value="http" disabled={MCP_STDIO_ONLY_ALPHA}>HTTP</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {transport === 'stdio' ? (
        <div className="grid gap-3 sm:grid-cols-[1fr_1fr]">
          <div className="space-y-1.5">
            <div className="flex items-center gap-1.5">
              <Label className="text-xs">Command</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="size-3 cursor-default text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>Executable launched for stdio servers (resolved from PATH if not absolute).</TooltipContent>
              </Tooltip>
            </div>
            <AutocompleteInput
              value={draft.command}
              options={commandOptions}
              onValueChange={(value) => setField('command', value)}
              placeholder="e.g. ship-mcp"
              className="h-8 text-xs font-mono"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
            />
          </div>
          <div className="space-y-1.5">
            <div className="flex items-center gap-1.5">
              <Label className="text-xs">Arguments</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="size-3 cursor-default text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>Space-separated args passed to the command.</TooltipContent>
              </Tooltip>
            </div>
            <Input
              value={argsStr}
              onChange={(e) => setField('args', splitShellArgs(e.target.value))}
              placeholder="--port 3000"
              className="h-8 text-xs font-mono"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
            />
          </div>
        </div>
      ) : (
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">URL</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Endpoint for HTTP/SSE transport, including protocol and port.</TooltipContent>
            </Tooltip>
          </div>
          <Input
            value={draft.url ?? ''}
            onChange={(e) => setField('url', e.target.value || null)}
            placeholder="https://my-mcp-server.example.com"
            className="h-8 text-xs font-mono"
          />
        </div>
      )}

      {/* Env vars */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Environment Variables</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Injected into the server process. Use for API keys and secrets.</TooltipContent>
            </Tooltip>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="xs"
            className="h-5 px-1.5 text-[10px]"
            onClick={() => {
              const envCopy = { ...(draft.env ?? {}) };
              envCopy['NEW_KEY'] = '';
              setField('env', envCopy);
            }}
          >
            <Plus className="mr-0.5 size-3" />Add
          </Button>
        </div>
        {draft.env && Object.entries(draft.env).length > 0 && (
          <div className="space-y-2">
            {Object.entries(draft.env).map(([key, val], envIdx) => (
              <div key={envIdx} className="flex items-center gap-2">
                <AutocompleteInput
                  value={key}
                  options={envKeyOptions}
                  onValueChange={(value) => {
                    const entries = Object.entries(draft.env ?? {});
                    entries[envIdx] = [value, val ?? ''];
                    setField('env', Object.fromEntries(entries));
                  }}
                  placeholder="KEY"
                  className="h-7 w-32 shrink-0 text-xs font-mono"
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
                />
                <span className="text-xs text-muted-foreground">=</span>
                <Input
                  value={val ?? ''}
                  onChange={(e) => {
                    const envCopy = { ...(draft.env ?? {}) };
                    envCopy[key] = e.target.value;
                    setField('env', envCopy);
                  }}
                  placeholder="value"
                  className="h-7 flex-1 text-xs font-mono"
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="xs"
                  className="h-7 w-7 shrink-0 p-0"
                  onClick={() => {
                    const envCopy = { ...(draft.env ?? {}) };
                    delete envCopy[key];
                    setField('env', envCopy);
                  }}
                >
                  <Trash2 className="size-3" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      {validations.length > 0 && (
        <div className="space-y-1.5 rounded-md border bg-background/50 px-2.5 py-2">
          {validations.map((check, idx) => (
            <p
              key={`${check.message}-${idx}`}
              className={cn(
                "text-[11px]",
                check.level === 'warning' ? 'text-amber-600' : 'text-muted-foreground'
              )}
            >
              {check.level === 'warning' ? 'Warning' : 'Hint'}: {check.message}
            </p>
          ))}
        </div>
      )}

      <div className="flex items-center gap-2 pt-1">
        <Button
          size="sm"
          onClick={onSave}
          disabled={
            !draft.name.trim()
            || (transport === 'stdio' && !draft.command.trim())
            || (transport !== 'stdio' && !draft.url?.trim())
            || (MCP_STDIO_ONLY_ALPHA && transport !== 'stdio')
          }
        >
          Save
        </Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
      </div>
    </div>
  );
}

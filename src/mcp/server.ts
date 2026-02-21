import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import * as project from '../core/project';

export function createServer() {
  const server = new Server(
    {
      name: 'Ship Project Tracker',
      version: '1.0.0',
    },
    {
      capabilities: {
        tools: {},
      },
    }
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
      tools: [
        {
          name: 'init_project',
          description: 'Initialize the project tracking directory structure',
          inputSchema: { type: 'object', properties: {} },
        },
        {
          name: 'create_issue',
          description: 'Create a new project issue',
          inputSchema: {
            type: 'object',
            properties: {
              title: { type: 'string', description: 'Title of the issue' },
              description: { type: 'string', description: 'Detailed description of the issue' },
              status: { type: 'string', enum: project.ISSUE_STATUSES, default: 'backlog' },
            },
            required: ['title', 'description'],
          },
        },
        {
          name: 'update_issue_status',
          description: 'Update the status of an existing issue',
          inputSchema: {
            type: 'object',
            properties: {
              issueFileName: { type: 'string', description: 'The filename of the issue (e.g., my-issue.md)' },
              currentStatus: { type: 'string', enum: project.ISSUE_STATUSES },
              newStatus: { type: 'string', enum: project.ISSUE_STATUSES },
            },
            required: ['issueFileName', 'currentStatus', 'newStatus'],
          },
        },
        {
          name: 'list_issues',
          description: 'List all issues and their current status',
          inputSchema: { type: 'object', properties: {} },
        },
        {
          name: 'create_adr',
          description: 'Create a new Architecture Decision Record (ADR)',
          inputSchema: {
            type: 'object',
            properties: {
              title: { type: 'string', description: 'Title of the ADR' },
              decision: { type: 'string', description: 'The decision made' },
              status: { type: 'string', default: 'proposed' },
            },
            required: ['title', 'decision'],
          },
        },
        {
          name: 'add_link',
          description: 'Link two project items (Issue or ADR) together',
          inputSchema: {
            type: 'object',
            properties: {
              sourcePath: { type: 'string', description: 'Path to the source file' },
              targetPath: { type: 'string', description: 'Path to the target file to link to' },
            },
            required: ['sourcePath', 'targetPath'],
          },
        },
        {
          name: 'log_action',
          description: 'Log an agent action to the project log',
          inputSchema: {
            type: 'object',
            properties: {
              agent: { type: 'string', description: 'Name of the agent' },
              action: { type: 'string', description: 'Action performed' },
              details: { type: 'string', description: 'Details of the action' },
            },
            required: ['agent', 'action', 'details'],
          },
        },
      ],
    };
  });

  server.setRequestHandler(CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;
    try {
      switch (name) {
        case 'init_project':
          await project.initProject();
          await project.logAction('MCP Server', 'init_project', 'Initialized project structure');
          return { content: [{ type: 'text', text: 'Project structure initialized successfully.' }] };
        case 'create_issue': {
          const status = (args?.status as string) || 'backlog';
          const issuePath = await project.createIssue(args?.title as string, args?.description as string, status);
          await project.logAction('MCP Server', 'create_issue', `Created issue: ${args?.title} (${status})`);
          return { content: [{ type: 'text', text: `Issue created at ${issuePath}` }] };
        }
        case 'update_issue_status': {
          const newPath = await project.moveIssue(args?.issueFileName as string, args?.currentStatus as string, args?.newStatus as string);
          await project.logAction('MCP Server', 'update_issue_status', `Moved ${args?.issueFileName} from ${args?.currentStatus} to ${args?.newStatus}`);
          return { content: [{ type: 'text', text: `Issue moved to ${newPath}` }] };
        }
        case 'list_issues': {
          const issues = await project.listIssues();
          return { content: [{ type: 'text', text: JSON.stringify(issues, null, 2) }] };
        }
        case 'create_adr': {
          const adrStatus = (args?.status as string) || 'proposed';
          const adrPath = await project.createADR(args?.title as string, args?.decision as string, adrStatus);
          await project.logAction('MCP Server', 'create_adr', `Created ADR: ${args?.title} (${adrStatus})`);
          return { content: [{ type: 'text', text: `ADR created at ${adrPath}` }] };
        }
        case 'add_link': {
          await project.addLink(args?.sourcePath as string, args?.targetPath as string);
          await project.logAction('MCP Server', 'add_link', `Linked ${args?.sourcePath} to ${args?.targetPath}`);
          return { content: [{ type: 'text', text: 'Link added successfully.' }] };
        }
        case 'log_action': {
          await project.logAction(args?.agent as string, args?.action as string, args?.details as string);
          return { content: [{ type: 'text', text: 'Action logged successfully.' }] };
        }
        default:
          throw new Error(`Unknown tool: ${name}`);
      }
    } catch (error: any) {
      return { content: [{ type: 'text', text: `Error: ${error.message}` }], isError: true };
    }
  });

  return server;
}

export const server = createServer();

export async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('Vibe Project Tracker MCP server running on stdio');
}

if (require.main === module) {
  main().catch((error) => {
    console.error('Server error:', error);
    process.exit(1);
  });
}

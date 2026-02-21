import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { InMemoryTransport } from '@modelcontextprotocol/sdk/inMemory.js';
import { createServer } from './server';
import * as project from '../core/project';

// Mock the project core
vi.mock('../core/project', () => ({
    ISSUE_STATUSES: ['backlog', 'blocked', 'done', 'in-progress'],
    initProject: vi.fn(),
    createIssue: vi.fn(),
    moveIssue: vi.fn(),
    listIssues: vi.fn(),
    createADR: vi.fn(),
    addLink: vi.fn(),
    logAction: vi.fn(),
}));

describe('mcp/server', () => {
    let client: Client;
    // clientTransport and serverTransport are now declared as const inside beforeEach
    // so they don't need to be declared here with `let`
    // let serverTransport: InMemoryTransport;
    // let clientTransport: InMemoryTransport;

    beforeEach(async () => {
        vi.clearAllMocks();

        // Create in-memory transport for testing
        const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();

        // Create a fresh server instance for each test
        const server = createServer();
        client = new Client({ name: 'test-client', version: '1.0.0' }, { capabilities: {} });

        // Connect both simultaneously to avoid blocking
        await Promise.all([
            client.connect(clientTransport),
            server.connect(serverTransport)
        ]);
    });

    it('lists available tools', async () => {
        const response = await client.listTools();
        expect(response.tools).toBeDefined();
        expect(response.tools.length).toBeGreaterThan(0);
        const toolNames = response.tools.map((t: any) => t.name);
        expect(toolNames).toContain('create_issue');
        expect(toolNames).toContain('list_issues');
        expect(toolNames).toContain('add_link');
    });

    describe('tools', () => {
        it('executes create_issue successfully', async () => {
            vi.mocked(project.createIssue).mockResolvedValue('/path/to/issue.md');

            const response = await client.callTool({
                name: 'create_issue',
                arguments: {
                    title: 'Test Issue',
                    description: 'Test Description',
                    status: 'backlog'
                }
            });

            expect(project.createIssue).toHaveBeenCalledWith('Test Issue', 'Test Description', 'backlog');
            const text = (response.content[0] as { type: 'text'; text: string }).text;
            expect(text).toContain('Issue created at /path/to/issue.md');
        });

        it('executes list_issues successfully', async () => {
            const mockIssues = [{ file: 'issue1.md', status: 'backlog' }];
            vi.mocked(project.listIssues).mockResolvedValue(mockIssues);

            const response = await client.callTool({
                name: 'list_issues',
                arguments: {}
            });

            expect(project.listIssues).toHaveBeenCalled();
            const text = (response.content[0] as { type: 'text'; text: string }).text;
            expect(JSON.parse(text)).toEqual(mockIssues);
        });

        it('handles errors gracefully', async () => {
            vi.mocked(project.createIssue).mockRejectedValue(new Error('Test Error'));

            const response = await client.callTool({
                name: 'create_issue',
                arguments: {
                    title: 'Test Issue',
                    description: 'Test Description'
                }
            });

            // MCP errors are returned in the content or as a failure depending on how you structure it
            // In this server's implementation, it catches and returns isError: true
            const text = (response.content[0] as { type: 'text'; text: string }).text;
            expect(text).toContain('Error: Test Error');
        });

        it('handles unknown tool gracefully', async () => {
            const response = await client.callTool({
                name: 'unknown_tool',
                arguments: {}
            });
            expect(response.isError).toBe(true);
            const text = (response.content[0] as { type: 'text'; text: string }).text;
            expect(text).toContain('Unknown tool: unknown_tool');
        });
    });
});

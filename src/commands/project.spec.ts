import { describe, it, expect, vi, beforeEach } from 'vitest';
import inquirer from 'inquirer';
import commands from './project';
import * as project from '../core/project';

// Mock dependencies
vi.mock('inquirer', () => ({
    default: {
        prompt: vi.fn(),
    },
}));

vi.mock('../core/project', () => ({
    ISSUE_STATUSES: ['backlog', 'blocked', 'done', 'in-progress'],
    initProject: vi.fn(),
    createIssue: vi.fn(),
    moveIssue: vi.fn(),
    listIssues: vi.fn(),
    createADR: vi.fn(),
}));

vi.mock('../mcp/server', () => ({
    main: vi.fn(),
}));

vi.mock('../ui/server', () => ({
    startUiServer: vi.fn(),
}));

describe('commands/project', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('exports the expected command groups', () => {
        expect(commands).toHaveLength(3);
        const commandNames = commands.map(c => c.command);
        expect(commandNames).toContain('issue');
        expect(commandNames).toContain('adr');
        expect(commandNames).toContain('project');
    });

    describe('issue create', () => {
        it('prompts the user and calls project.createIssue', async () => {
            const issueCommand = commands.find(c => c.command === 'issue');
            const createSubcommand = issueCommand?.subcommands?.find(s => s.command.startsWith('create'));

            vi.mocked(inquirer.prompt).mockResolvedValue({
                description: 'New Description',
                status: 'backlog'
            });
            vi.mocked(project.createIssue).mockResolvedValue('test.md');

            if (createSubcommand?.action) {
                await (createSubcommand.action as any)('New Title', 'New Description', 'backlog');
            }

            expect(inquirer.prompt).toHaveBeenCalled();
            expect(project.createIssue).toHaveBeenCalledWith('New Title', 'New Description', 'backlog');
        });
    });

    describe('project init', () => {
        it('calls project.initProject', async () => {
            const projectCommand = commands.find(c => c.command === 'project');
            const initSubcommand = projectCommand?.subcommands?.find(s => s.command === 'init');

            if (initSubcommand?.action) {
                await (initSubcommand.action as any)();
            }

            expect(project.initProject).toHaveBeenCalled();
        });
    });
});

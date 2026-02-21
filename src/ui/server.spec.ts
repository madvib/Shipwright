import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import { app } from './server';
import * as project from '../core/project';
import fs from 'fs-extra';
import path from 'path';

// Mock the project core and fs-extra
vi.mock('../core/project', () => ({
    ISSUE_STATUSES: ['backlog', 'blocked', 'done', 'in-progress'],
    listIssues: vi.fn(),
    getProjectDir: vi.fn(),
}));

vi.mock('fs-extra', () => ({
    default: {
        pathExists: vi.fn(),
        readFile: vi.fn(),
        readdir: vi.fn(),
    },
    pathExists: vi.fn(),
    readFile: vi.fn(),
    readdir: vi.fn(),
}));

describe('ui/server', () => {
    const mockProjectDir = '/test/project';

    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(project.getProjectDir).mockResolvedValue(mockProjectDir);
    });

    describe('GET /', () => {
        it('renders the dashboard with issues and ADRs', async () => {
            vi.mocked(project.listIssues).mockResolvedValue([
                { file: 'test-issue.md', status: 'backlog' }
            ] as any);
            vi.mocked(fs.pathExists).mockResolvedValue(true as never);
            vi.mocked(fs.readdir).mockResolvedValue(['test-adr.md'] as never);
            vi.mocked(fs.readFile).mockResolvedValue('# Project Log' as never);

            const response = await request(app).get('/');

            expect(response.status).toBe(200);
            expect(response.text).toContain('Project Dashboard');
            expect(response.text).toContain('test-issue.md');
            expect(response.text).toContain('test-adr.md');
            expect(response.text).toContain('Project Log');
        });

        it('handles errors gracefully', async () => {
            vi.mocked(project.getProjectDir).mockRejectedValue(new Error('Project not found'));

            const response = await request(app).get('/');

            expect(response.status).toBe(500);
            expect(response.text).toBe('Project not found');
        });
    });

    describe('GET /view', () => {
        it('renders a markdown file securely', async () => {
            const targetPath = path.join(mockProjectDir, 'Issues', 'backlog', 'test-issue.md');
            vi.mocked(fs.readFile).mockResolvedValue('---\ntitle: Test\n---\n# Content' as never);

            const response = await request(app)
                .get('/view')
                .query({ path: targetPath });

            expect(response.status).toBe(200);
            expect(response.text).toContain('Content');
            expect(response.text).not.toContain('title: Test'); // Frontmatter should be stripped
        });

        it('denies access to files outside the project directory', async () => {
            const response = await request(app)
                .get('/view')
                .query({ path: '/etc/passwd' });

            expect(response.status).toBe(403);
            expect(response.text).toBe('Access denied');
        });

        it('requires a path parameter', async () => {
            const response = await request(app).get('/view');
            expect(response.status).toBe(400);
        });

        it('returns 404 for non-existent files', async () => {
            vi.mocked(fs.readFile).mockRejectedValue(new Error('ENOENT'));
            const response = await request(app)
                .get('/view')
                .query({ path: path.join(mockProjectDir, 'missing.md') });

            expect(response.status).toBe(404);
        });
    });
});

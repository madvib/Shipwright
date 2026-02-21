import express from 'express';
import path from 'path';
import fs from 'fs-extra';
import MarkdownIt from 'markdown-it';
import * as project from '../core/project';

const md = new MarkdownIt();
export const app = express();
const PORT = process.env.PORT || 4747;

// Simple HTML escaping
const escapeHtml = (unsafe: string): string => {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
};

const stripFrontmatter = (content: string): string => {
  if (content.startsWith('---')) {
    const endOfFrontmatter = content.indexOf('---', 3);
    if (endOfFrontmatter !== -1) {
      return content.substring(endOfFrontmatter + 3).trim();
    }
  }
  return content;
};

const getLinksFromFrontmatter = (content: string): string[] => {
  const linksMatch = content.match(/links: \[(.*)\]/);
  if (linksMatch) {
    return linksMatch[1].split(',').map(l => l.trim().replace(/"/g, '')).filter(l => l);
  }
  return [];
};

const layout = (title: string, body: string): string => `
  <html>
    <head>
      <title>${escapeHtml(title)}</title>
      <style>
        :root { --primary: #2563eb; --bg: #f8fafc; --text: #1e293b; --border: #e2e8f0; }
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; line-height: 1.5; color: var(--text); background: var(--bg); max-width: 900px; margin: 0 auto; padding: 40px 20px; }
        h1, h2, h3 { color: #0f172a; margin-top: 2rem; }
        h1 { border-bottom: 2px solid var(--border); padding-bottom: 0.5rem; }
        a { color: var(--primary); text-decoration: none; }
        a:hover { text-decoration: underline; }
        .card { background: white; border: 1px solid var(--border); border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        .status-badge { display: inline-block; padding: 2px 8px; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase; }
        .backlog { background: #e2e8f0; color: #475569; }
        .in-progress { background: #dbeafe; color: #1e40af; }
        .done { background: #dcfce7; color: #166534; }
        .blocked { background: #fee2e2; color: #991b1b; }
        .links-section { margin-top: 20px; padding-top: 10px; border-top: 1px dashed var(--border); }
        .nav-link { display: inline-block; margin-bottom: 20px; font-weight: 500; }
        table { width: 100%; border-collapse: collapse; margin: 1rem 0; }
        th, td { text-align: left; padding: 12px; border-bottom: 1px solid var(--border); }
        th { background: #f1f5f9; }
        .log-container { overflow-x: auto; }
      </style>
    </head>
    <body>
      ${body}
    </body>
  </html>
`;

app.get('/', async (req: express.Request, res: express.Response) => {
  try {
    const projectDir = await project.getProjectDir();
    const adrDir = path.join(projectDir, 'ADR');
    const issuesDir = path.join(projectDir, 'Issues');

    const issues = await project.listIssues();
    const adrFiles = await fs.pathExists(adrDir) ? await fs.readdir(adrDir) : [];
    const logPath = path.join(projectDir, 'log.md');
    const logContent = await fs.pathExists(logPath)
      ? await fs.readFile(logPath, 'utf8')
      : '';

    let issuesHtml = '';
    for (const status of project.ISSUE_STATUSES) {
      const statusIssues = issues.filter((i: project.Issue) => i.status === status);
      issuesHtml += `
        <div class="card">
          <h3 style="margin-top: 0"><span class="status-badge ${status}">${status}</span></h3>
          <ul style="list-style: none; padding: 0">
            ${statusIssues.map((i: project.Issue) => `<li><a href="/view?path=${encodeURIComponent(path.join(issuesDir, status, i.file))}">${escapeHtml(i.file)}</a></li>`).join('') || '<li>No issues</li>'}
          </ul>
        </div>
      `;
    }

    const body = `
      <h1>Ship Project Dashboard</h1>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
        <div>
          <h2>Issues</h2>
          ${issuesHtml}
        </div>
        <div>
          <h2>ADRs</h2>
          <div class="card">
            <ul style="list-style: none; padding: 0">
              ${adrFiles.filter(f => f.endsWith('.md')).map(f => `<li><a href="/view?path=${encodeURIComponent(path.join(adrDir, f))}">${escapeHtml(f)}</a></li>`).join('') || '<li>No ADRs</li>'}
            </ul>
          </div>

          <h2>Log</h2>
          <div class="card log-container">
            ${md.render(logContent)}
          </div>
        </div>
      </div>
    `;

    res.send(layout('Project Dashboard', body));
  } catch (error: any) {
    res.status(500).send(error.message);
  }
});

app.get('/view', async (req: express.Request, res: express.Response): Promise<any> => {
  try {
    const queryPath = req.query.path as string;
    if (!queryPath) return res.status(400).send('Path is required');

    const projectDir = await project.getProjectDir();
    const resolvedBase = path.resolve(projectDir);
    const resolvedPath = path.resolve(queryPath);

    // Security check: is the resolvedPath inside resolvedBase?
    const relative = path.relative(resolvedBase, resolvedPath);
    const isSafe = relative && !relative.startsWith('..') && !path.isAbsolute(relative);

    if (!isSafe && resolvedBase !== resolvedPath) {
      return res.status(403).send('Access denied');
    }

    const content = await fs.readFile(resolvedPath, 'utf8');
    const stripped = stripFrontmatter(content);
    const links = getLinksFromFrontmatter(content);

    const body = `
            <a href="/" class="nav-link">← Back to Dashboard</a>
            <div class="card">
                ${md.render(stripped)}

                ${links.length > 0 ? `
                    <div class="links-section">
                        <h4>Links</h4>
                        <ul>
                            ${links.map(l => `<li><a href="/view?path=${encodeURIComponent(l)}">${escapeHtml(l)}</a></li>`).join('')}
                        </ul>
                    </div>
                ` : ''}
            </div>
        `;

    res.send(layout(path.basename(resolvedPath), body));
  } catch (error: any) {
    res.status(404).send('File not found');
  }
});

export const startUiServer = async (): Promise<void> => {
  app.listen(PORT, () => {
    console.log(`✓ Web UI server running at http://localhost:${PORT}`);
  });
};

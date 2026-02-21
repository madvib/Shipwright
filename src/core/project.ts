import fs from 'fs-extra';
import path from 'path';

export const PROJECT_DIR_NAME = '.ship';
export const ISSUE_STATUSES = ['backlog', 'blocked', 'done', 'in-progress'];

/**
 * Traverses upwards from the given directory to find the nearest .project folder.
 */
export const getProjectDir = async (startDir: string = process.cwd()): Promise<string> => {
  let currentDir = startDir;
  while (true) {
    const projectPath = path.join(currentDir, PROJECT_DIR_NAME);
    if (await fs.pathExists(projectPath)) {
      return projectPath;
    }
    const parentDir = path.dirname(currentDir);
    if (parentDir === currentDir) {
      // Reached the root of the file system
      throw new Error('Project tracking not initialized. Run `ship project init` to create a .ship directory.');
    }
    currentDir = parentDir;
  }
};
const DEFAULT_TEMPLATES: Record<string, string> = {
  'issue.md': `---
title: {{title}}
status: {{status}}
created: {{createdAt}}
links: []
---

# {{title}}

## Description
{{description}}

## Tasks
- [ ] Initial task

## Links
-
`,
  'adr.md': `---
title: {{title}}
status: {{status}}
date: {{date}}
links: []
---

# ADR: {{title}}

## Context
What is the problem we are solving?

## Decision
What is the decision we made?

## Status
{{status}}

## Consequences
What are the consequences of this decision?
`,
  'log_header.md': `# Project Log

| Date | Agent | Action | Details |
|------|-------|--------|---------|
`
};

export const sanitizeFileName = (name: string): string => {
  return name.replace(/[\\/]/g, '_');
};

const getTemplate = async (projectDir: string, name: keyof typeof DEFAULT_TEMPLATES): Promise<string> => {
  const customPath = path.join(projectDir, 'templates', name);
  if (await fs.pathExists(customPath)) {
    return await fs.readFile(customPath, 'utf8');
  }
  return DEFAULT_TEMPLATES[name];
};

export const initProject = async (): Promise<void> => {
  try {
    const existing = await getProjectDir();
    console.log(`Project tracking already exists at: ${existing}`);
    return;
  } catch (e) {
    // Expected if not found
  }

  const newProjectDir = path.join(process.cwd(), PROJECT_DIR_NAME);
  await fs.ensureDir(newProjectDir);
  await fs.ensureDir(path.join(newProjectDir, 'ADR'));
  for (const status of ISSUE_STATUSES) {
    await fs.ensureDir(path.join(newProjectDir, 'Issues', status));
  }

  const readmePath = path.join(newProjectDir, 'README.md');
  if (!(await fs.pathExists(readmePath))) {
    await fs.writeFile(readmePath, '# Project Tracking\n\nManaged by vibe-cli.');
  }

  const logPath = path.join(newProjectDir, 'log.md');
  if (!(await fs.pathExists(logPath))) {
    const header = await getTemplate(newProjectDir, 'log_header.md');
    await fs.writeFile(logPath, header);
  }
};

export const ejectTemplates = async (): Promise<string[]> => {
  const projectDir = await getProjectDir();
  const templatesDir = path.join(projectDir, 'templates');
  await fs.ensureDir(templatesDir);
  const ejected: string[] = [];
  for (const [name, content] of Object.entries(DEFAULT_TEMPLATES)) {
    const dest = path.join(templatesDir, name);
    if (!(await fs.pathExists(dest))) {
      await fs.writeFile(dest, content);
      ejected.push(dest);
    }
  }
  return ejected;
};

export const createIssue = async (title: string, description: string, status: string = 'backlog'): Promise<string> => {
  const projectDir = await getProjectDir();
  if (!ISSUE_STATUSES.includes(status)) {
    throw new Error(`Invalid status: ${status}`);
  }

  const fileName = sanitizeFileName(`${title.toLowerCase().replace(/\s+/g, '-')}.md`);
  const filePath = path.join(projectDir, 'Issues', status, fileName);

  const template = await getTemplate(projectDir, 'issue.md');
  let content = template
    .replace(/{{title}}/g, title)
    .replace(/{{description}}/g, description)
    .replace(/{{status}}/g, status)
    .replace(/{{createdAt}}/g, new Date().toISOString());

  await fs.writeFile(filePath, content);
  return filePath;
};

export const moveIssue = async (issueFileName: string, currentStatus: string, newStatus: string): Promise<string> => {
  const projectDir = await getProjectDir();
  if (!ISSUE_STATUSES.includes(newStatus)) {
    throw new Error(`Invalid status: ${newStatus}`);
  }

  const fileName = sanitizeFileName(issueFileName);
  const oldPath = path.join(projectDir, 'Issues', currentStatus, fileName);
  const newPath = path.join(projectDir, 'Issues', newStatus, fileName);

  if (!(await fs.pathExists(oldPath))) {
    throw new Error(`Issue not found: ${oldPath}`);
  }

  let content = await fs.readFile(oldPath, 'utf8');
  content = content.replace(/status: .*/, `status: ${newStatus}`);

  await fs.move(oldPath, newPath);
  await fs.writeFile(newPath, content);
  return newPath;
};

export const createADR = async (title: string, decision: string, status: string = 'proposed'): Promise<string> => {
  const projectDir = await getProjectDir();
  const fileName = sanitizeFileName(`${title.toLowerCase().replace(/\s+/g, '-')}.md`);
  const filePath = path.join(projectDir, 'ADR', fileName);

  const template = await getTemplate(projectDir, 'adr.md');
  let content = template
    .replace(/{{title}}/g, title)
    .replace(/{{status}}/g, status)
    .replace(/{{date}}/g, new Date().toISOString());

  await fs.writeFile(filePath, content);
  return filePath;
};

export const addLink = async (filePath: string, targetPath: string): Promise<void> => {
  if (!(await fs.pathExists(filePath))) throw new Error(`File not found: ${filePath}`);

  let content = await fs.readFile(filePath, 'utf8');
  // Simple regex to find links array in frontmatter
  const linksMatch = content.match(/links: \[(.*)\]/);
  if (linksMatch) {
    const currentLinks = linksMatch[1].split(',').map(l => l.trim()).filter(l => l);
    if (!currentLinks.includes(`"${targetPath}"`)) {
      currentLinks.push(`"${targetPath}"`);
      content = content.replace(/links: \[.*\]/, `links: [${currentLinks.join(', ')}]`);
      await fs.writeFile(filePath, content);
    }
  }
};

export const logAction = async (agent: string, action: string, details: string): Promise<void> => {
  let projectDir: string;
  try {
    projectDir = await getProjectDir();
  } catch (e) {
    await initProject();
    projectDir = await getProjectDir();
  }
  const logPath = path.join(projectDir, 'log.md');
  const date = new Date().toISOString();

  if (!(await fs.pathExists(logPath))) {
    const header = await getTemplate(projectDir, 'log_header.md');
    await fs.writeFile(logPath, header);
  }
  const entry = `| ${date} | ${agent} | ${action} | ${details} |\n`;
  await fs.appendFile(logPath, entry);
};

export interface Issue {
  file: string;
  status: string;
}

export const listIssues = async (): Promise<Issue[]> => {
  const projectDir = await getProjectDir();
  const issues: Issue[] = [];
  for (const status of ISSUE_STATUSES) {
    const statusDir = path.join(projectDir, 'Issues', status);
    if (await fs.pathExists(statusDir)) {
      const files = await fs.readdir(statusDir);
      for (const file of files) {
        if (file.endsWith('.md')) {
          issues.push({ file, status });
        }
      }
    }
  }
  return issues;
};

import inquirer from 'inquirer';
import chalk from 'chalk';
import * as project from '../core/project';
import { main as startMcpServer } from '../mcp/server';
import { startUiServer } from '../ui/server';

const issueCommands = {
  command: 'issue',
  description: 'Manage project issues',
  subcommands: [
    {
      command: 'create <title>',
      description: 'Create a new issue',
      action: async (title: string) => {
        const answers = await inquirer.prompt([
          { type: 'input', name: 'description', message: 'Issue description:' },
          { type: 'list', name: 'status', message: 'Initial status:', choices: project.ISSUE_STATUSES, default: 'backlog' }
        ]);
        const filePath = await project.createIssue(title, answers.description, answers.status);
        console.log(chalk.green(`✓ Issue created: ${filePath}`));
      }
    },
    {
      command: 'move <fileName> <currentStatus> <newStatus>',
      description: 'Move an issue to a new status',
      action: async (fileName: string, currentStatus: string, newStatus: string) => {
        try {
          const filePath = await project.moveIssue(fileName, currentStatus, newStatus);
          console.log(chalk.green(`✓ Issue moved to: ${filePath}`));
        } catch (error: any) {
          console.error(chalk.red(`Error: ${error.message}`));
        }
      }
    },
    {
      command: 'list',
      description: 'List all issues',
      action: async () => {
        const issues = await project.listIssues();
        console.log(chalk.blue('Project Issues:'));
        issues.forEach((i: project.Issue) => {
          console.log(chalk.gray(`- [${i.status}] ${i.file}`));
        });
      }
    }
  ]
};

const adrCommands = {
  command: 'adr',
  description: 'Manage Architecture Decision Records (ADRs)',
  subcommands: [
    {
      command: 'create <title>',
      description: 'Create a new ADR',
      action: async (title: string) => {
        const answers = await inquirer.prompt([
          { type: 'input', name: 'decision', message: 'Decision:' },
          { type: 'input', name: 'status', message: 'Status:', default: 'proposed' }
        ]);
        const filePath = await project.createADR(title, answers.decision, answers.status);
        console.log(chalk.green(`✓ ADR created: ${filePath}`));
      }
    }
  ]
};

const projectManagementCommands = {
  command: 'project',
  description: 'Project-level management',
  subcommands: [
    {
      command: 'init',
      description: 'Initialize project tracking structure',
      action: async () => {
        await project.initProject();
        console.log(chalk.green('✓ Project tracking structure initialized in .ship/'));
      }
    },
    {
      command: 'mcp',
      description: 'Start the Project Tracking MCP server (STDIO)',
      action: async () => {
        await startMcpServer();
      }
    },
    {
      command: 'ui',
      description: 'Start the Project Tracking Web UI',
      action: async () => {
        await startUiServer();
      }
    }
    // TODO: ejectTemplates and link have optional types, omitted temporarily 
  ]
};

export default [
  issueCommands,
  adrCommands,
  projectManagementCommands
];

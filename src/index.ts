#!/usr/bin/env node
import { Command } from 'commander';
import initCommand from './commands/init';
import newCommand from './commands/new';
import listCommand from './commands/list';
import showCommand from './commands/show';
import configCommand from './commands/config';
import deleteCommand from './commands/delete';
import projectCommands from './commands/project';

const program = new Command();

program
  .name('ship')
  .description('Project tracking tool and MCP server')
  .version('1.0.0');

export const registerCommand = (parent: Command, cmd: any) => {
  const command = parent.command(cmd.command)
    .description(cmd.description)
    .action(cmd.action);

  if (cmd.subcommands && Array.isArray(cmd.subcommands)) {
    cmd.subcommands.forEach((sub: any) => registerCommand(command, sub));
  }
};

const commands = [
  initCommand,
  newCommand,
  listCommand,
  showCommand,
  configCommand,
  deleteCommand,
  ...projectCommands
];

commands.forEach(cmd => registerCommand(program, cmd));

program.parse(process.argv);

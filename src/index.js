const { Command } = require('commander');
const initCommand = require('./commands/init');
const newCommand = require('./commands/new');
const listCommand = require('./commands/list');
const generateCommand = require('./commands/generate');
const showCommand = require('./commands/show');
const configCommand = require('./commands/config');
const deleteCommand = require('./commands/delete');

const program = new Command();

program
  .name('vibe')
  .description('AI-assisted feature development CLI')
  .version('1.0.0');

const commands = [
  initCommand,
  newCommand,
  listCommand,
  generateCommand,
  showCommand,
  configCommand,
  deleteCommand,
];

commands.forEach(cmd => {
  const command = program.command(cmd.command)
    .description(cmd.description)
    .action(cmd.action);

  // This is a simplified way to handle arguments.
  // For more complex argument handling, you might need a more robust solution.
  if (cmd.command.includes('<')) {
      const args = cmd.command.match(/<[^>]+>/g);
      if (args) {
          args.forEach(arg => {
              command.argument(arg, `Description for ${arg}`);
          });
      }
  }
   if (cmd.command.includes('[')) {
      const args = cmd.command.match(/\[[^\]]+\]/g);
      if (args) {
          args.forEach(arg => {
              command.argument(arg, `Description for ${arg}`);
          });
      }
  }
});

program.parse(process.argv);
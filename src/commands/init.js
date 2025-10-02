const inquirer = require('inquirer');
const chalk = require('chalk');
const { ensureConfigDir, saveConfig } = require('../core/config');
const { AI_PROVIDERS, LANGUAGES } = require('../constants');

const initCommand = {
  command: 'init',
  description: 'Initialize vibe in current project',
  action: async () => {
    console.log(chalk.blue('🚀 Initializing vibe...'));

    await ensureConfigDir();

    const providerChoices = Object.keys(AI_PROVIDERS).map(key => ({
      name: AI_PROVIDERS[key].name,
      value: key
    }));

    const languageChoices = Object.keys(LANGUAGES).map(key => ({
      name: LANGUAGES[key].name,
      value: key
    }));

    const answers = await inquirer.prompt([
      {
        type: 'list',
        name: 'aiProvider',
        message: 'Choose AI provider:',
        choices: providerChoices
      }
    ]);

    const provider = AI_PROVIDERS[answers.aiProvider];

    const modelAnswer = await inquirer.prompt([
      {
        type: 'list',
        name: 'aiModel',
        message: 'Choose model:',
        choices: provider.models
      }
    ]);

    const languageAnswer = await inquirer.prompt([
      {
        type: 'list',
        name: 'language',
        message: 'Primary programming language:',
        choices: languageChoices
      }
    ]);

    const language = LANGUAGES[languageAnswer.language];

    const frameworkAnswer = await inquirer.prompt([
      {
        type: 'list',
        name: 'testFramework',
        message: 'Test framework:',
        choices: language.testFrameworks
      }
    ]);

    const pathAnswers = await inquirer.prompt([
      {
        type: 'input',
        name: 'testDir',
        message: 'Test directory:',
        default: 'tests'
      },
      {
        type: 'input',
        name: 'srcDir',
        message: 'Source directory:',
        default: 'src'
      },
      {
        type: 'password',
        name: 'apiKey',
        message: `API Key (or set ${provider.envVar} env var):`,
        default: process.env[provider.envVar] || ''
      }
    ]);

    const config = {
      aiProvider: answers.aiProvider,
      aiModel: modelAnswer.aiModel,
      language: languageAnswer.language,
      testFramework: frameworkAnswer.testFramework,
      testDir: pathAnswers.testDir,
      srcDir: pathAnswers.srcDir,
      apiKey: pathAnswers.apiKey
    };

    await saveConfig(config);

    console.log(chalk.green('✓ Configuration saved!'));
    console.log(chalk.gray(`  Config directory: .vibe`));
    console.log(chalk.gray(`  AI Provider: ${provider.name} (${config.aiModel})`));
    console.log(chalk.gray(`  Language: ${language.name}`));
  }
};

module.exports = initCommand;
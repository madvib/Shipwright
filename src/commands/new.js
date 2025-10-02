const inquirer = require('inquirer');
const chalk = require('chalk');
const { ensureConfigDir, loadConfig } = require('../core/config');
const { saveFeature } = require('../core/features');
const { generateId } = require('../utils');
const { LANGUAGES } = require('../constants');

const newCommand = {
  command: 'new <title>',
  description: 'Create a new feature specification',
  action: async (title) => {
    await ensureConfigDir();
    const config = await loadConfig();
    const language = LANGUAGES[config.language];

    console.log(chalk.blue(`📝 Creating feature: ${title}`));

    const answers = await inquirer.prompt([
      {
        type: 'input',
        name: 'description',
        message: 'Feature description:'
      },
      {
        type: 'editor',
        name: 'acceptanceCriteria',
        message: 'Acceptance criteria (one per line):'
      },
      {
        type: 'list',
        name: 'language',
        message: 'Programming language:',
        choices: Object.keys(LANGUAGES).map(key => ({
          name: LANGUAGES[key].name,
          value: key
        })),
        default: config.language
      }
    ]);

    const selectedLanguage = LANGUAGES[answers.language];

    const frameworkAnswer = await inquirer.prompt([
      {
        type: 'list',
        name: 'testFramework',
        message: 'Test framework:',
        choices: selectedLanguage.testFrameworks,
        default: config.testFramework
      }
    ]);

    const id = generateId(title);
    const feature = {
      id,
      title,
      description: answers.description,
      acceptanceCriteria: answers.acceptanceCriteria
        .split('\n')
        .map(c => c.trim())
        .filter(c => c),
      language: answers.language,
      testFramework: frameworkAnswer.testFramework,
      status: 'documented',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      generatedFiles: {}
    };

    await saveFeature(feature);

    console.log(chalk.green(`✓ Feature created: ${id}`));
    console.log(chalk.gray(`  File: .vibe/features/${id}.json`));
  }
};

module.exports = newCommand;
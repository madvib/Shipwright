import inquirer from 'inquirer';
import chalk from 'chalk';
import { ensureConfigDir, saveConfig, ProjectConfig } from '../core/config';
import { LANGUAGES, LanguageConfig } from '../constants';

const initCommand = {
  command: 'init',
  description: 'Initialize vibe in current project',
  action: async () => {
    console.log(chalk.blue('🚀 Initializing vibe tracking...'));

    await ensureConfigDir();

    const languageChoices = Object.keys(LANGUAGES).map(key => ({
      name: LANGUAGES[key].name,
      value: key
    }));

    const languageAnswer = await inquirer.prompt([
      {
        type: 'list',
        name: 'language',
        message: 'Primary programming language:',
        choices: languageChoices
      }
    ]);

    const language: LanguageConfig = LANGUAGES[languageAnswer.language];

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
      }
    ]);

    const config: ProjectConfig = {
      language: languageAnswer.language,
      testFramework: frameworkAnswer.testFramework,
      testDir: pathAnswers.testDir,
      srcDir: pathAnswers.srcDir
    };

    await saveConfig(config);

    console.log(chalk.green('✓ Configuration saved!'));
    console.log(chalk.gray(`  Config directory: .vibe`));
    console.log(chalk.gray(`  Language: ${language.name}`));
  }
};

export default initCommand;
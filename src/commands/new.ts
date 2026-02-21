import inquirer from 'inquirer';
import chalk from 'chalk';
import { ensureConfigDir, loadConfig } from '../core/config';
import { saveFeature, Feature } from '../core/features';
import { generateId } from '../utils';
import { LANGUAGES } from '../constants';

const newCommand = {
  command: 'new <title>',
  description: 'Create a new feature specification',
  action: async (title: string) => {
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
    const feature: Feature = {
      id,
      title,
      description: answers.description,
      acceptanceCriteria: answers.acceptanceCriteria
        .split('\n')
        .map((c: string) => c.trim())
        .filter((c: string) => c),
      language: answers.language,
      testFramework: frameworkAnswer.testFramework,
      status: 'documented' as any, // bypassing strict union type for now based on feature status logic
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      generatedFiles: {}
    };

    await saveFeature(feature);

    console.log(chalk.green(`✓ Feature created: ${id}`));
    console.log(chalk.gray(`  File: .vibe/features/${id}.json`));
  }
};

export default newCommand;
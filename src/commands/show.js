const chalk = require('chalk');
const { ensureConfigDir } = require('../core/config');
const { loadFeature } = require('../core/features');
const { LANGUAGES } = require('../constants');

const showCommand = {
  command: 'show <feature-id>',
  description: 'Show feature details',
  action: async (featureId) => {
    await ensureConfigDir();

    let feature;
    try {
      feature = await loadFeature(featureId);
    } catch {
      console.log(chalk.red(`❌ Feature not found: ${featureId}`));
      return;
    }

    const language = LANGUAGES[feature.language];

    console.log(chalk.blue(`\n📋 ${feature.title}\n`));
    console.log(chalk.gray(`ID: ${feature.id}`));
    console.log(chalk.gray(`Status: ${feature.status}`));
    console.log(chalk.gray(`Language: ${language.name}`));
    console.log(chalk.gray(`Test Framework: ${feature.testFramework}`));
    console.log();

    if (feature.description) {
      console.log(chalk.bold('Description:'));
      console.log(feature.description);
      console.log();
    }

    if (feature.acceptanceCriteria.length > 0) {
      console.log(chalk.bold('Acceptance Criteria:'));
      feature.acceptanceCriteria.forEach((c, i) => {
        console.log(`  ${i + 1}. ${c}`);
      });
      console.log();
    }

    if (Object.keys(feature.generatedFiles).length > 0) {
      console.log(chalk.bold('Generated Files:'));
      for (const [type, filePath] of Object.entries(feature.generatedFiles)) {
        console.log(`  ${type}: ${chalk.cyan(filePath)}`);
      }
      console.log();
    }
  }
};

module.exports = showCommand;
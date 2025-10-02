const ora = require('ora');
const chalk = require('chalk');
const fs = require('fs-extra');
const path = require('path');
const { ensureConfigDir, loadConfig } = require('../core/config');
const { loadFeature, saveFeature } = require('../core/features');
const { callAI } = require('../core/ai');
const { generateTestPrompt } = require('../prompts/testPrompt');
const { generateCodePrompt } = require('../prompts/codePrompt');
const { AI_PROVIDERS, LANGUAGES } = require('../constants');

const generateCommand = {
  command: 'generate <type> <feature-id>',
  description: 'Generate tests or implementation',
  action: async (type, featureId) => {
    await ensureConfigDir();
    const config = await loadConfig();

    if (!config.apiKey && !process.env[AI_PROVIDERS[config.aiProvider].envVar]) {
      console.log(chalk.red(`❌ No API key configured. Run: vibe init`));
      return;
    }

    const apiKey = config.apiKey || process.env[AI_PROVIDERS[config.aiProvider].envVar];

    let feature;
    try {
      feature = await loadFeature(featureId);
    } catch {
      console.log(chalk.red(`❌ Feature not found: ${featureId}`));
      return;
    }

    const language = LANGUAGES[feature.language];

    if (type === 'tests') {
      const spinner = ora(`Generating tests with ${AI_PROVIDERS[config.aiProvider].name}...`).start();

      try {
        const prompt = generateTestPrompt(feature, feature.language, feature.testFramework);
        const tests = await callAI(config.aiProvider, config.aiModel, prompt, apiKey);

        // Write test file
        const testFileName = `${feature.id}${language.testExtension}`;
        const testFilePath = path.join(config.testDir, testFileName);
        await fs.ensureDir(config.testDir);
        await fs.writeFile(testFilePath, tests);

        // Update feature
        feature.generatedFiles.tests = testFilePath;
        feature.status = 'tests-generated';
        feature.updatedAt = new Date().toISOString();
        await saveFeature(feature);

        spinner.succeed(chalk.green(`Tests generated: ${testFilePath}`));
      } catch (error) {
        spinner.fail(chalk.red('Failed to generate tests'));
        console.error(chalk.red(error.message));
      }
    } else if (type === 'code') {
      if (!feature.generatedFiles.tests) {
        console.log(chalk.red('❌ Generate tests first: vibe generate tests ' + featureId));
        return;
      }

      const spinner = ora(`Generating implementation with ${AI_PROVIDERS[config.aiProvider].name}...`).start();

      try {
        const testCode = await fs.readFile(feature.generatedFiles.tests, 'utf-8');
        const prompt = generateCodePrompt(feature, feature.language, testCode);
        const implementation = await callAI(config.aiProvider, config.aiModel, prompt, apiKey);

        // Write implementation file
        const implFileName = `${feature.id}${language.fileExtension}`;
        const implFilePath = path.join(config.srcDir, implFileName);
        await fs.ensureDir(config.srcDir);
        await fs.writeFile(implFilePath, implementation);

        // Update feature
        feature.generatedFiles.implementation = implFilePath;
        feature.status = 'implemented';
        feature.updatedAt = new Date().toISOString();
        await saveFeature(feature);

        spinner.succeed(chalk.green(`Implementation generated: ${implFilePath}`));
      } catch (error) {
        spinner.fail(chalk.red('Failed to generate implementation'));
        console.error(chalk.red(error.message));
      }
    } else {
      console.log(chalk.red('❌ Invalid type. Use: tests or code'));
    }
  }
};

module.exports = generateCommand;
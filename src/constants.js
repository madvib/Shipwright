const AI_PROVIDERS = {
  anthropic: {
    name: 'Anthropic (Claude)',
    models: ['claude-sonnet-4-20250514', 'claude-opus-4-20250514'],
    envVar: 'ANTHROPIC_API_KEY'
  },
  openai: {
    name: 'OpenAI (GPT)',
    models: ['gpt-4o', 'gpt-4-turbo', 'gpt-3.5-turbo'],
    envVar: 'OPENAI_API_KEY'
  },
  google: {
    name: 'Google (Gemini)',
    models: ['gemini-pro', 'gemini-ultra'],
    envVar: 'GOOGLE_API_KEY'
  }
};

const LANGUAGES = {
  javascript: {
    name: 'JavaScript',
    testFrameworks: ['jest', 'vitest', 'mocha'],
    fileExtension: '.js',
    testExtension: '.test.js'
  },
  typescript: {
    name: 'TypeScript',
    testFrameworks: ['jest', 'vitest', 'mocha'],
    fileExtension: '.ts',
    testExtension: '.test.ts'
  },
  python: {
    name: 'Python',
    testFrameworks: ['pytest', 'unittest'],
    fileExtension: '.py',
    testExtension: '_test.py'
  },
  go: {
    name: 'Go',
    testFrameworks: ['testing'],
    fileExtension: '.go',
    testExtension: '_test.go'
  },
  rust: {
    name: 'Rust',
    testFrameworks: ['cargo test'],
    fileExtension: '.rs',
    testExtension: '.rs'
  },
  java: {
    name: 'Java',
    testFrameworks: ['junit', 'testng'],
    fileExtension: '.java',
    testExtension: 'Test.java'
  }
};

module.exports = {
  AI_PROVIDERS,
  LANGUAGES,
};
export interface LanguageConfig {
  name: string;
  testFrameworks: string[];
  fileExtension: string;
  testExtension: string;
}

export const LANGUAGES: Record<string, LanguageConfig> = {
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
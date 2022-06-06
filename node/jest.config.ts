export default {
  forceExit: true,
  clearMocks: true,
  collectCoverage: false,
  testMatch: ['**/?(*.)+(spec|test).ts', '**/?(*.)+(spec|test).js'],
  transform: {
    '^.+\\.tsx?$': 'ts-jest',
  },
};

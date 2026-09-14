// ESLint 9+ flat config (required as of ESLint 10 — .eslintrc.* is no longer
// read at all). See package.json's "lint" script.
//
// NOTE on the typescript devDependency: @typescript-eslint (both
// eslint-plugin and parser) hard-fail at require time against TypeScript
// >=7.0 ("typescript-eslint does not support TS 7.0", see
// https://github.com/typescript-eslint/typescript-eslint/issues/10940).
// The root `typescript` devDependency is pinned to ^6.0.3 for this reason —
// do not bump it past the 6.x line until that issue is resolved upstream,
// or lint will hard-crash again regardless of this config file.
import tseslint from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';

export default [
  {
    ignores: ['out/**', 'node_modules/**', '**/*.vsix', 'media/**'],
  },
  {
    files: ['src/**/*.ts'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: 'module',
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      // The codebase relies on untyped `any` in several places (VS Code API
      // surfaces, dynamic language server payloads); keep this a warning
      // rather than an error so lint stays informative without demanding a
      // separate large-scale typing pass.
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': 'warn',
      // The codebase uses require() in several places for optional/native
      // modules (node-pty, webview bundling helpers) loaded conditionally
      // at runtime rather than via static ESM import; downgrade to a
      // warning instead of rewriting these call sites as part of an
      // unrelated lint-pipeline fix.
      '@typescript-eslint/no-require-imports': 'warn',
    },
  },
];

import js from '@eslint/js';
import tsParser from '@typescript-eslint/parser';
import tsPlugin from '@typescript-eslint/eslint-plugin';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';

// Flat ESLint config for a browser/Tauri React app.
//
// Three things that were worth spelling out rather than leaving implicit:
//
// 1. `no-undef` is OFF for TS/TSX.
//    TypeScript already performs name resolution and understands the DOM
//    lib; re-running ESLint's no-undef on top of that is both redundant and
//    actively wrong — it can't tell a type reference (`HTMLCanvasElement`)
//    from a value reference. The typescript-eslint project explicitly
//    recommends turning it off for TS files:
//      https://typescript-eslint.io/troubleshooting/faqs/general/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
//
// 2. `globals.browser` is still supplied.
//    Mostly defensive — any stray pre-TS `.js` file we add later (the
//    flat-config root itself, vite.config.ts, etc.) benefits from the same
//    global visibility, and it keeps linting consistent with the runtime
//    env (Tauri v2 webviews are browsers).
//
// 3. A `React` global is provided as `readonly`.
//    A few files still use `React.FormEvent` / `React.MouseEvent` rather
//    than named imports. With the automatic JSX runtime (`jsx: "react-jsx"`
//    in tsconfig), `React` is not imported as a value, yet the namespace
//    form shows up as an unresolved identifier to ESLint even though
//    TypeScript resolves it via the JSX import. Declaring the global
//    silences the noise without forcing a churn of imports, and TS still
//    enforces that the referenced members actually exist.
export default [
  js.configs.recommended,
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: 'module',
        ecmaFeatures: { jsx: true },
      },
      globals: {
        ...globals.browser,
        React: 'readonly',
      },
    },
    plugins: {
      '@typescript-eslint': tsPlugin,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...tsPlugin.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': 'warn',
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      // TypeScript handles undefined-name checking; ESLint's no-undef
      // can't see the DOM lib or type-only imports.
      'no-undef': 'off',
    },
  },
];

import js from "@eslint/js"
import vue from "eslint-plugin-vue"
import tseslint from "typescript-eslint"
import vueParser from "vue-eslint-parser"
import prettier from "eslint-config-prettier"

export default [
  { ignores: ["dist/**", "coverage/**"] },

  {
    // vueParser must only wrap .vue files; tseslint.configs.recommended below
    // will set the correct parser for .ts/.js files in ESLint 9 flat config.
    files: ["**/*.vue"],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tseslint.parser,
        ecmaVersion: "latest",
        sourceType: "module",
      },
      globals: {
        window: "readonly",
        document: "readonly",
        navigator: "readonly",
        HTMLElement: "readonly",
        MouseEvent: "readonly",
        PointerEvent: "readonly",
        ResizeObserver: "readonly",
        Node: "readonly",
      },
    },
  },

  js.configs.recommended,

  ...tseslint.configs.recommended,

  ...vue.configs["flat/recommended"],

  prettier,

  {
    rules: {
      "vue/multi-word-component-names": "off",
    },
  },
]

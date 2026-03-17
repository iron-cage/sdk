import js from "@eslint/js"
import vue from "eslint-plugin-vue"
import tseslint from "typescript-eslint"
import vueParser from "vue-eslint-parser"
import prettier from "eslint-config-prettier"

export default [
  {
    files: ["**/*.{js,ts,vue}"],
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

# src/components/ui

Low-level UI primitives (shadcn-vue style) that wrap Reka UI with Tailwind styling. Each subdirectory exposes its primitives through an `index.ts` barrel — always import from the barrel, never from a `.vue` file directly.

| Directory | Responsibility |
|-----------|----------------|
| `alert/` | Alert container and title/description primitives |
| `badge/` | Pill-shaped badge with CVA-driven variants |
| `button/` | Button primitive with CVA-driven variants and sizes |
| `card/` | Card container with header, content, title and footer parts |
| `dialog/` | Accessible dialog primitives wrapping Reka UI |
| `dropdown-menu/` | Accessible dropdown menu primitives wrapping Reka UI |
| `input/` | Styled text input primitive |
| `label/` | Form label primitive bound to inputs by `for` |
| `popover/` | Accessible popover primitives wrapping Reka UI |
| `scroll-area/` | Custom-styled scroll container wrapping Reka UI |
| `select/` | Accessible select primitives wrapping Reka UI |
| `separator/` | Horizontal / vertical separator line |
| `skeleton/` | Animated placeholder for loading states |
| `sonner/` | Toast notification host powered by vue-sonner |
| `switch/` | Accessible toggle switch primitive wrapping Reka UI |
| `tabs/` | Accessible tab primitives wrapping Reka UI |
| `textarea/` | Styled multi-line text input primitive |

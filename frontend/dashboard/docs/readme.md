# docs/

Technical documentation for iron_dashboard developers.

## Purpose

This directory contains in-depth technical documentation for developers working on iron_dashboard. Unlike user-facing documentation in `readme.md`, these docs focus on architecture, implementation details, and integration patterns.

---

## Documentation Index

### Core Documentation

1. **[architecture.md](architecture.md)** - Vue 3 architecture, component hierarchy, state management patterns
   - Composition API patterns (`<script setup>`)
   - Component structure (SFC organization)
   - State management (Pinia stores, TanStack Query)
   - Routing strategy (Vue Router, route guards)
   - Build pipeline (Vite configuration)

2. **[api_integration.md](api_integration.md)** - Backend integration guide
   - REST API endpoints (iron_api)
   - TypeScript type definitions (matching Rust schemas)
   - WebSocket integration (connection lifecycle, message handling)
   - Error handling patterns
   - Authentication flow (JWT tokens)

3. **[sitemap.md](sitemap.md)** - Navigation structure and route hierarchy
   - Route hierarchy and URL structure
   - Navigation menu organization (sidebar, header)
   - Page flow diagrams (authentication, user journey)
   - Access control matrix
   - Navigation patterns (sidebar toggle, active route indication)

4. **[components.md](components.md)** - UI component inventory and catalog
   - Vue component catalog (views, layouts, composables)
   - Component dependencies and relationships
   - Props, emits, and usage examples
   - Shared patterns (data fetching, mutations, date formatting)
   - Component metrics and design patterns

5. **[development_setup.md](development_setup.md)** - Development environment setup guide
   - Node.js and npm installation
   - Project dependency management
   - Environment variable configuration
   - Development server workflow
   - IDE configuration (VSCode, WebStorm)
   - Troubleshooting common issues

---

## Documentation Organization

**Knowledge Hierarchy (iron_cage standards):**

```
iron_dashboard knowledge sources (priority order):
1. Test documentation (tests/readme.md, tests/manual/readme.md)
2. Source code doc comments (src/**/*.ts, src/**/*.vue)
3. Technical documentation (docs/*.md) ← YOU ARE HERE
4. Specification (spec.md) - requirements and architecture
5. User-facing docs (readme.md) - setup and usage
```

**When to Document Here:**
- Architecture decisions (why Vue 3 Composition API, not Options API)
- Integration patterns (how to add new API endpoint)
- Build configuration (Vite plugins, optimization strategies)
- Development workflows (adding new view, creating component)

**When NOT to Document Here:**
- Requirements (use `spec.md` instead)
- User setup instructions (use `readme.md` instead)
- Test procedures (use `tests/manual/readme.md` instead)
- Bug fixes (use test documentation + source comments instead)

---

## Adding New Documentation

**File Naming:**
- Use lowercase_snake_case (e.g., `state_management.md`, not `StateManagement.md`)
- Use descriptive names (e.g., `websocket_integration.md`, not `ws.md`)
- Avoid generic names (e.g., `helpers.md`, `utils.md`, `misc.md`)

**Required Sections:**
1. **Purpose** - Why this document exists
2. **Scope** - What is and isn't covered
3. **Content** - The actual documentation
4. **Examples** - Code snippets demonstrating concepts
5. **References** - Links to related docs or external resources

**Update Responsibility Table:**
When adding new documentation file, update the Responsibility Table below with:
- Entity name (file name)
- Responsibility (what it documents)
- Input→Output (what transforms it describes)
- Scope (what's included)
- Out of Scope (what's excluded)

---

## Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|----------------|--------------|-------|--------------|
| `readme.md` | Documentation index and organization guide | - → Doc navigation | Index of all docs, documentation standards, when to use each doc | Actual technical content (architecture.md, api_integration.md, sitemap.md, components.md, development_setup.md) |
| `architecture.md` | Vue 3 architecture and implementation patterns | - → Architecture guide | Component structure, state management, routing, build pipeline | API integration (api_integration.md), requirements (spec.md), sitemap (sitemap.md), component catalog (components.md), setup (development_setup.md) |
| `api_integration.md` | Backend integration patterns and type safety | - → Integration guide | REST endpoints, TypeScript types, WebSocket, error handling, auth | Frontend architecture (architecture.md), backend spec (iron_api/spec.md), component details (components.md) |
| `sitemap.md` | Navigation structure and route hierarchy | - → Sitemap | Route hierarchy, navigation schema, page flow, access control matrix | Route implementation (architecture.md), component details (components.md), requirements (spec.md) |
| `components.md` | UI component inventory and catalog (shadcn-vue + views) | - → Component catalog | shadcn-vue components (Button, Dialog, etc.), Vue views, composables, props/emits, usage examples, dependency graph | Architecture patterns (architecture.md), API contracts (api_integration.md), navigation structure (sitemap.md) |
| `development_setup.md` | Development environment setup and configuration | - → Setup guide | Node.js install, dependencies, env vars, dev server, IDE config, troubleshooting | Architecture (architecture.md), quick start (readme.md), production deployment (TBD) |

**Complete Entity Coverage Verified:** 6 entities listed (all files and directories in docs/).

---

## Migration Knowledge

**Migration Context:**
Documentation structure created during migration from `dev/frontend/` to `dev/module/iron_dashboard/` (Phase 1 of TDD migration plan).

**Rationale:**
Frontend originally had single `README.md` (Vue template boilerplate). New structure separates concerns:
- `readme.md` - User-facing (setup, usage)
- `spec.md` - Requirements (FR, NFR, API contracts)
- `docs/` - Technical details (architecture, integration)
- `tests/` - Test strategy (manual procedures)

**Knowledge Preservation:**
All development insights captured in docs/ (not scattered across code comments or external wiki). Follows iron_cage knowledge management standards (files_structure.rulebook.md).

---

## External References

### Vue Ecosystem

- [Vue 3 Documentation](https://vuejs.org/guide/) - Framework reference
- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api-faq.html) - API design rationale
- [Vue 3 SFC Spec](https://vuejs.org/api/sfc-spec.html) - Single-file component syntax
- [Vue Router Documentation](https://router.vuejs.org/) - Routing library
- [Pinia Documentation](https://pinia.vuejs.org/) - State management

### Build Tooling

- [Vite Documentation](https://vitejs.dev/guide/) - Build tool
- [Vite Plugin API](https://vitejs.dev/guide/api-plugin.html) - Plugin development
- [Rollup Plugin API](https://rollupjs.org/plugin-development/) - Underlying bundler

### UI and Accessibility

- [Radix Vue Documentation](https://www.radix-vue.com/) - Accessible primitives
- [Tailwind CSS Documentation](https://tailwindcss.com/docs) - Utility classes
- [WCAG 2.1 Quick Reference](https://www.w3.org/WAI/WCAG21/quickref/) - Accessibility guidelines
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/) - ARIA patterns

### TypeScript

- [TypeScript Documentation](https://www.typescriptlang.org/docs/) - Language reference
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html) - Comprehensive guide
- [Vue 3 TypeScript Guide](https://vuejs.org/guide/typescript/overview.html) - Vue-specific TypeScript

### Testing (Future)

- [Vitest Documentation](https://vitest.dev/) - Unit testing framework
- [Cypress Documentation](https://docs.cypress.io/) - Component/E2E testing
- [Playwright Documentation](https://playwright.dev/) - E2E testing
- [Testing Library Documentation](https://testing-library.com/) - Testing utilities

---

## Documentation Standards

Following codebase_hygiene.rulebook.md and documentation.rulebook.md:

1. **Markdown Format** - Use GitHub-flavored markdown
2. **Code Examples** - Always include working code snippets
3. **Inline Comments** - Explain WHY, not WHAT
4. **External Links** - Use absolute URLs, check validity
5. **No Duplication** - Link to existing docs, don't duplicate content
6. **No Backups** - Update existing docs, don't create `_old` versions
7. **Lowercase Filenames** - Use lowercase_snake_case, not PascalCase

---

## Maintenance

**When to Update:**
- New feature added → Update architecture.md (if architectural change)
- New API endpoint → Update api_integration.md
- Build configuration changed → Update architecture.md (Build Pipeline section)
- State management pattern changed → Update architecture.md (State Management section)

**Who Updates:**
- Developer implementing feature updates relevant docs
- Code reviewer verifies docs updated during PR review
- Documentation updates required for PR approval

**Stale Documentation Detection:**
- If code and docs diverge, docs are considered stale
- Stale docs must be updated or removed (never left outdated)
- Use docs/ for persistent knowledge only (temporary insights → test docs)

---

**End of Documentation Index**

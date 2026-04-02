# translate_app

A Tauri v2 desktop application with React 19 frontend for clipboard-based translation with LaTeX rendering support.

## Build & Run Commands

```bash
# Frontend development
pnpm dev                    # Start Vite dev server
pnpm build                  # TypeScript check + Vite production build
pnpm preview                # Preview production build

# Tauri desktop app
pnpm tauri dev              # Start Tauri dev app (frontend + Rust backend)
pnpm tauri build            # Build production desktop app
pnpm tauri build --debug    # Build debug version for testing
```

## Type Checking

```bash
pnpm tsc --noEmit           # TypeScript type checking only
```

**No test framework configured** — verify changes manually via `pnpm tauri dev`.

## Architecture

```
src/                        # React frontend
├── components/             # UI components
│   ├── ui/                 # Radix UI primitives (button, scroll-area, etc.)
│   ├── TranslationView.tsx # Translation output display
│   ├── SettingsModal.tsx   # Settings UI
│   └── Titlebar.tsx        # Window titlebar
├── contexts/               # React Context providers
│   └── ThemeContext.tsx    # Theme state + cross-window sync
├── hooks/                  # Custom hooks
├── lib/                    # Utilities
│   ├── utils.ts            # cn(), hexToRgba()
│   └── latex.ts            # LaTeX parsing/rendering
└── types/                  # TypeScript type definitions

src-tauri/src/              # Rust backend
├── settings.rs             # Settings persistence + events
├── clipboard.rs            # Clipboard monitoring
├── engines/                # Translation engines (baidu, google, ollama, siliconflow)
└── lib.rs                  # Tauri app setup
```

## Code Style

### TypeScript

- **Strict mode enabled** — `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`
- **Path alias**: Use `@/` for all imports from `src/`
  ```typescript
  import { Titlebar } from '@/components/Titlebar';
  import { useTheme } from '@/contexts/ThemeContext';
  import { hexToRgba } from '@/lib/utils';
  ```
- **No `any`** — use proper types or `unknown` with type guards
- **No `@ts-ignore` / `@ts-expect-error`** — fix the underlying type issue

### Naming Conventions

| Type | Convention | Example |
|------|-----------|---------|
| Components | PascalCase | `TranslationView.tsx` |
| Hooks | `use` prefix, camelCase | `useClipboard.ts` |
| Utilities | camelCase | `hexToRgba()`, `cn()` |
| Interfaces | PascalCase | `interface ThemeSettings { ... }` |
| Constants | SCREAMING_SNAKE_CASE | `const LATEX_INLINE_REGEX = ...` |
| Engine types | Discriminated unions | `type Engine = 'baidu' \| 'google'` |

### Import Order

1. React/standard libraries
2. Tauri APIs (`@tauri-apps/api/*`)
3. Local components (`@/components/*`)
4. Contexts/hooks (`@/contexts/*`, `@/hooks/*`)
5. Utils/libs (`@/lib/*`)

### Styling

- **Tailwind CSS v4** with `@tailwindcss/vite` plugin
- Use `cn()` from `@/lib/utils` for conditional classes
- Inline styles only for dynamic values (theme colors, transparency)
- Radix UI primitives for accessible components

### Error Handling

```typescript
// Tauri invoke — catch and display in UI
try {
  const result = await invoke<T>('command', { args });
  setData(result);
} catch (e) {
  setError(String(e));
}

// Background tasks — catch and log
invoke('set_settings', { settings }).catch(console.error);
```

### Tauri Events

- Use `app.emit_all()` in Rust to broadcast events to all windows
- Use `listen()` from `@tauri-apps/api/event` in React to receive events
- Always clean up listeners in useEffect return

## Key Patterns

### React Context + Tauri Backend

```typescript
// Load on mount, listen for changes
useEffect(() => {
  loadTheme().then(setTheme).catch(console.error);
}, []);

useEffect(() => {
  const unlisten = listen('theme-changed', () => {
    loadTheme().then(setTheme).catch(console.error);
  });
  return () => { unlisten.then(f => f()); };
}, []);
```

### Component Structure

- Keep components focused on one responsibility
- Extract shared UI to `src/components/ui/`
- Use composition over prop drilling

## Documentation

- Specs: `docs/superpowers/specs/`
- Plans: `docs/superpowers/plans/`
- CHANGELOG: `CHANGELOG.md`

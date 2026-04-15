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
cargo check --manifest-path src-tauri/Cargo.toml   # Rust type checking
cargo test --manifest-path src-tauri/Cargo.toml    # Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml baidu   # Run single test
```

**No frontend test framework configured** — verify changes manually via `pnpm tauri dev`.

## Agent Workflow (MANDATORY — read WORKFLOW.md for full details)

1. **On implementation tasks**: launch Hephaestus (impl) + Momus (review) + Librarian (research) in parallel background agents.
2. **After implementation**: update `Feature.md` with what changed (small incremental notes).
3. **When user says "commit"**: write `Feature.md` → `CHANGELOG.md`, clear `Feature.md`, bump `package.json` version (`0.0.x + 1`), git commit.
4. **Skills**: `.opencode/skills/hephaestus/`, `momus/`, `librarian/` — always load relevant skills when delegating.

Git branches: `main` (stable) → `dev` (development) → `feature-*`/`fix-*` (work branches). Merge work branches → `dev`.

## Architecture

```
src/                        # React frontend
├── components/
│   ├── ui/                 # Radix UI primitives
│   ├── TranslationView.tsx # Translation output (uses LatexRenderer)
│   ├── LatexRenderer.tsx   # Detects $...$ / $$...$$ → renders via react-katex
│   ├── SettingsModal.tsx   # Settings UI (all 4 engines)
│   └── Titlebar.tsx        # Window titlebar + engine selector
├── contexts/
│   └── ThemeContext.tsx    # Theme state + cross-window sync via 'theme-changed' event
├── hooks/
│   └── useClipboard.ts     # Clipboard polling hook
├── lib/
│   ├── utils.ts            # cn(), hexToRgba()
│   └── latex.ts            # extractLatex(), reinsertLatex(), detectLatex()
└── types/
    └── engine.ts           # Engine discriminated union type

src-tauri/src/              # Rust backend
├── settings.rs             # Settings persistence (%APPDATA%/translate_app/settings.json)
├── clipboard.rs            # Clipboard monitoring
├── engines/
│   ├── mod.rs              # TranslationEngine trait + EngineError enum + lang_code_to_name()
│   ├── baidu.rs            # Baidu standard API (MD5 signature)
│   ├── google.rs           # Google (mirror-first, fallback to official)
│   ├── llmapi.rs           # LLM API (OpenAI-compatible, default: SiliconFlow)
│   └── ollama.rs           # Local Ollama
└── lib.rs                  # Tauri commands: translate, get_settings, set_settings
```

## Key Runtime Behaviour

### LaTeX Handling (critical — do not break)

- **LLM engines** (`llmapi`, `ollama`): send raw clipboard text; system prompt instructs LLM to preserve `$...$` and `$$...$$` delimiters unchanged.
- **Non-LLM engines** (`baidu`, `google`): `extractLatex()` replaces LaTeX with `[[LATEX_BLOCK_N]]` / `[[LATEX_INLINE_N]]` placeholders before translation, then `reinsertLatex()` restores them. LLMs would corrupt these placeholders — that's why LLM engines skip this step.
- Branch in `App.tsx`: `const isLLMEngine = engine === 'llmapi' || engine === 'ollama'`.

### Settings

- Persisted to `%APPDATA%\translate_app\settings.json` (Windows) via `dirs::config_dir()`.
- Every `set_settings` call emits a `theme-changed` event to all windows — `App.tsx` and `ThemeContext.tsx` both listen to reload.
- `google_url` field is `#[serde(skip_serializing)]` — a legacy/migration field, not saved.

### Engine: llmapi

- Endpoint hardcoded: `https://api.siliconflow.cn/v1/chat/completions` (OpenAI-compatible).
- Default model: `deepseek-ai/DeepSeek-V3`. Falls back to this if `llmapi_model` is empty.
- System prompt uses `lang_code_to_name()` from `engines/mod.rs` to convert codes (`zh` → `Chinese`, `en` → `English`, `ja` → `Japanese`, `ko` → `Korean`, `auto` → `auto-detect`).

### Engine: Google

- Mirror URL tried first; falls back to `google_official_url` + API key if mirror fails.
- Credential check: at least one of `google_mirror_url`, `google_official_url`, or `google_api_key` must be set.

## Code Style

### TypeScript

- **Strict mode** — `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`
- **Path alias**: `@/` maps to `src/`
- **No `any`** — use proper types or `unknown` with type guards
- **No `@ts-ignore` / `@ts-expect-error`**

### Rust

- **Error handling**: `Result<T, EngineError>` pattern; never `unwrap()` on external I/O
- **Async**: `async_trait` for trait methods, `tokio` runtime
- **No `#[allow(dead_code)]`** without justification

### Naming Conventions

| Type | Convention | Example |
|------|-----------|---------|
| Components | PascalCase | `TranslationView.tsx` |
| Hooks | `use` prefix, camelCase | `useClipboard.ts` |
| Utilities | camelCase | `hexToRgba()`, `cn()` |
| Constants | SCREAMING_SNAKE_CASE | `LATEX_INLINE_REGEX` |
| Engine types | discriminated union | `type Engine = 'baidu' \| 'google' \| 'llmapi' \| 'ollama'` |
| Rust structs | PascalCase | `LLMApiEngine` |
| Rust functions | snake_case | `generate_sign()` |

### Import Order (TypeScript)

1. React / standard libraries
2. Tauri APIs (`@tauri-apps/api/*`)
3. Local components (`@/components/*`)
4. Contexts/hooks (`@/contexts/*`, `@/hooks/*`)
5. Utils/libs (`@/lib/*`)

### Styling

- **Tailwind CSS v4** with `@tailwindcss/vite` plugin
- Use `cn()` from `@/lib/utils` for conditional classes
- Inline styles only for dynamic values (theme colors, transparency)

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

- Rust: `app.emit_all()` to broadcast to all windows
- React: `listen()` from `@tauri-apps/api/event`; always clean up in `useEffect` return

## Adding a New Translation Engine

1. Create `src-tauri/src/engines/new_engine.rs`, implement `TranslationEngine` trait from `mod.rs`
2. Declare `pub mod new_engine` in `engines/mod.rs`
3. Add settings fields in `settings.rs` (`Settings` struct + `Default` impl)
4. Add credential validation + engine instantiation in `lib.rs` `translate` command
5. Add UI section in `SettingsModal.tsx` and engine option in `Titlebar.tsx`
6. Decide: does the engine handle LaTeX natively (LLM)? If yes, add it to the `isLLMEngine` check in `App.tsx`

## Baidu API Notes

- **Endpoint**: `https://api.fanyi.baidu.com/api/trans/vip/translate` (GET)
- **Signature**: `md5(appid + q + salt + secret_key)` — use raw text for `q` in signature, URL-encode only in the final request
- **Salt**: `rand::random::<u16>()`
- **Error codes**: `54001` = bad signature, `54003` = rate limited, `52001` = timeout

## Documentation

- Workflow: `WORKFLOW.md`
- Changelog: `CHANGELOG.md`
- In-progress features: `Feature.md` (cleared on each commit)
- Specs/plans: `docs/superpowers/`

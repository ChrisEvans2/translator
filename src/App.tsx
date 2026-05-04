import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { writeText, readImage } from '@tauri-apps/plugin-clipboard-manager';
import { Titlebar } from '@/components/Titlebar';
import { TranslationView } from '@/components/TranslationView';
import { ThemeProvider } from '@/contexts/ThemeContext';
import { useClipboard } from '@/hooks/useClipboard';
import { extractLatex, reinsertLatex } from '@/lib/latex';

interface Settings {
  source_lang: string;
  target_lang: string;
  engine: string;
  clipboard_enabled: boolean;
  image_translation_enabled: boolean;
  always_on_top: boolean;
  auto_show: boolean;
  theme_color: string;
  bg_color: string;
  text_color: string;
  transparency: number;
  locale: string;
  baidu_app_id: string;
  baidu_secret_key: string;
  google_mirror_url: string;
  google_official_url: string;
  google_api_key: string;
  llmapi_api_key: string;
  llmapi_url: string;
  llmapi_model: string;
  ollama_url: string;
  ollama_model: string;
}

function TranslationApp() {
  const [translatedText, setTranslatedText] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [settings, setSettings] = useState<Settings | null>(null);
  const settingsRef = useRef<Settings | null>(null);
  const isLoadingRef = useRef(false);
  const abortRef = useRef<AbortController | null>(null);
  const clipboardText = useClipboard();

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  useEffect(() => {
    invoke<Settings>('get_settings').then((s) => {
      setSettings(s);
      invoke('set_window_always_on_top', { alwaysOnTop: s.always_on_top }).catch(console.error);
    }).catch(console.error);

    const unlisten = listen('theme-changed', async () => {
      const newSettings = await invoke<Settings>('get_settings');
      setSettings(newSettings);
      invoke('set_window_always_on_top', { alwaysOnTop: newSettings.always_on_top }).catch(console.error);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    if (!clipboardText) return;

    const translate = async () => {
      const currentSettings = settingsRef.current;
      if (!currentSettings || !currentSettings.clipboard_enabled) return;
      if (isLoadingRef.current) return;

      if (abortRef.current) {
        abortRef.current.abort();
      }
      abortRef.current = new AbortController();

      isLoadingRef.current = true;
      setIsLoading(true);
      setError(undefined);
      
      try {
        const isLLMEngine = currentSettings.engine === 'llmapi' || currentSettings.engine === 'ollama';
        const { plainText, segments } = isLLMEngine
          ? { plainText: clipboardText, segments: [] }
          : extractLatex(clipboardText);
        
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => reject(new Error('翻译请求超时 (30s)')), 30000);
        });

        const result = await Promise.race([
          invoke<{ text: string; error?: string }>('translate', {
            text: plainText,
            from: currentSettings.source_lang,
            to: currentSettings.target_lang,
            engine: currentSettings.engine,
          }),
          timeoutPromise,
        ]);

        if (result.error) {
          setError(result.error);
        } else {
          const finalText = segments.length > 0 ? reinsertLatex(result.text, segments) : result.text;
          setTranslatedText(finalText);
          if (currentSettings.auto_show) {
            invoke('show_main_window').catch(console.error);
          }
        }
      } catch (e) {
        const msg = String(e);
        if (!msg.includes('abort')) {
          setError(msg);
        }
      } finally {
        isLoadingRef.current = false;
        setIsLoading(false);
      }
    };

    const translateImage = async () => {
      const currentSettings = settingsRef.current;
      if (!currentSettings || !currentSettings.image_translation_enabled) return;
      if (isLoadingRef.current) return;

      const isLLMEngine = currentSettings.engine === 'llmapi' || currentSettings.engine === 'ollama';
      if (!isLLMEngine) return;

      try {
        const clipboardImage = await readImage();
        const rgba = await clipboardImage.rgba();
        const base64 = btoa(String.fromCharCode(...rgba));
        
        isLoadingRef.current = true;
        setIsLoading(true);
        setError(undefined);

        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => reject(new Error('图片翻译请求超时 (30s)')), 30000);
        });

        const result = await Promise.race([
          invoke<{ text: string; error?: string }>('translate_image', {
            imageBase64: base64,
            to: currentSettings.target_lang,
            engine: currentSettings.engine,
          }),
          timeoutPromise,
        ]);

        if (result.error) {
          setError(result.error);
        } else {
          setTranslatedText(result.text);
          if (currentSettings.auto_show) {
            invoke('show_main_window').catch(console.error);
          }
        }
      } catch (e) {
      } finally {
        isLoadingRef.current = false;
        setIsLoading(false);
      }
    };

    translate();
    translateImage();
  }, [clipboardText, settings?.engine, settings?.source_lang, settings?.target_lang, settings?.image_translation_enabled]);

  const handleCopy = async () => {
    if (translatedText) {
      await writeText(translatedText);
    }
  };

  const handleOpenSettingsWindow = async () => {
    await invoke('open_settings_window').catch((error) => {
      console.error('open_settings_window failed:', error);
      setError(`打开设置失败: ${String(error)}`);
    });
  };

  return (
    <div className="h-screen flex flex-col">
      <Titlebar 
        onSettingsClick={() => {
          void handleOpenSettingsWindow();
        }}
        onCopyClick={handleCopy}
        currentEngine={settings?.engine || 'baidu'}
        onEngineChange={(engine) => {
          console.log('[App] Engine changed to:', engine);
          if (settings) {
            const newSettings = { ...settings, engine };
            setSettings(newSettings);
            invoke('set_settings', { settings: newSettings }).catch(console.error);
          }
        }}
      />
      <TranslationView 
        translatedText={translatedText}
        isLoading={isLoading}
        error={error}
      />
    </div>
  );
}

function App() {
  return (
    <ThemeProvider>
      <TranslationApp />
    </ThemeProvider>
  );
}

export default App;

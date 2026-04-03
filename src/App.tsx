import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
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
  siliconflow_api_key: string;
  siliconflow_model: string;
  ollama_url: string;
  ollama_model: string;
}

function TranslationApp() {
  const [translatedText, setTranslatedText] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [settings, setSettings] = useState<Settings | null>(null);
  const clipboardText = useClipboard();

  useEffect(() => {
    invoke<Settings>('get_settings').then(setSettings).catch(console.error);

    const unlisten = listen('theme-changed', async () => {
      console.log('[App] Received theme-changed event, reloading settings...');
      const newSettings = await invoke<Settings>('get_settings');
      console.log('[App] Settings loaded:', newSettings);
      setSettings(newSettings);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    if (!clipboardText || !settings?.clipboard_enabled || isLoading) return;

    const translate = async () => {
      setIsLoading(true);
      setError(undefined);
      
      try {
        const { plainText, segments } = extractLatex(clipboardText);
        
        const result = await invoke<{ text: string; error?: string }>('translate', {
          text: plainText,
          from: settings.source_lang,
          to: settings.target_lang,
          engine: settings.engine,
        });

        if (result.error) {
          setError(result.error);
        } else {
          const finalText = reinsertLatex(result.text, segments);
          setTranslatedText(finalText);
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setIsLoading(false);
      }
    };

    translate();
  }, [clipboardText, settings]);

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

import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ThemeSettings {
  themeColor: string;
  bgColor: string;
  textColor: string;
  transparency: number;
}

interface ThemeContextType {
  theme: ThemeSettings;
  setTheme: (theme: ThemeSettings) => void;
}

const defaultTheme: ThemeSettings = {
  themeColor: '#426666',
  bgColor: '#3f3f3f',
  textColor: '#ffffff',
  transparency: 50,
};

function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
}

async function loadTheme(): Promise<ThemeSettings> {
  const settings = await invoke<any>('get_settings');
  return {
    themeColor: settings.theme_color || defaultTheme.themeColor,
    bgColor: settings.bg_color || defaultTheme.bgColor,
    textColor: settings.text_color || defaultTheme.textColor,
    transparency: settings.transparency ?? defaultTheme.transparency,
  };
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<ThemeSettings>(defaultTheme);

  useEffect(() => {
    loadTheme().then(setTheme).catch(console.error);
  }, []);

  useEffect(() => {
    const unlisten = listen('theme-changed', () => {
      console.log('[ThemeContext] Received theme-changed event, reloading theme...');
      loadTheme().then(newTheme => {
        console.log('[ThemeContext] Theme loaded:', newTheme);
        setTheme(newTheme);
      }).catch(console.error);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}

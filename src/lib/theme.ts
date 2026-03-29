import { useState, useEffect } from 'react';

export interface ThemeSettings {
  themeColor: string;
  bgColor: string;
  textColor: string;
  transparency: number;
}

export function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));
  root.style.setProperty('--titlebar-opacity', String(settings.transparency / 200));
}

export function useTheme() {
  const [theme, setTheme] = useState<ThemeSettings>({
    themeColor: '#426666',
    bgColor: '#3f3f3f',
    textColor: '#ffffff',
    transparency: 50,
  });

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  return { theme, setTheme, applyTheme };
}

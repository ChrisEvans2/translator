import { createContext, useContext, useState, useEffect, ReactNode } from 'react';

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
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));
  root.style.setProperty('--titlebar-opacity', String(settings.transparency / 200));
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<ThemeSettings>(defaultTheme);

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

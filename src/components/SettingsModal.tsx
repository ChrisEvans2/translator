import { useTheme } from '@/contexts/ThemeContext';
import { useEffect, useState } from 'react';
import { X, ChevronDown, ChevronUp, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

interface SettingsModalProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

const defaultSettings = {
  source_lang: 'auto',
  target_lang: 'zh',
  engine: 'google',
  google_url: 'https://translate.googleapis.com/translate_a/single',
  google_api_key: '',
  baidu_app_id: '',
  baidu_secret_key: '',
  siliconflow_api_key: '',
  siliconflow_model: 'deepseek-ai/DeepSeek-V3',
  ollama_url: 'http://localhost:11434',
  ollama_model: 'llama2',
};

export function SettingsModal({ open = true, onOpenChange }: SettingsModalProps) {
  const { theme, setTheme } = useTheme();
  const sections = [
    { id: 'general', label: '通用' },
    { id: 'appearance', label: '外观' },
    { id: 'engines', label: '引擎' },
  ] as const;

  const [activeSection, setActiveSection] = useState<(typeof sections)[number]['id']>('general');
  const [expandedEngine, setExpandedEngine] = useState<string | null>('baidu');
  const [settings, setSettings] = useState(defaultSettings);

  useEffect(() => {
    if (!open) return;
    invoke<any>('get_settings').then((s) => {
      setSettings({
        source_lang: s.source_lang || defaultSettings.source_lang,
        target_lang: s.target_lang || defaultSettings.target_lang,
        engine: s.engine || defaultSettings.engine,
        google_url: s.google_url || defaultSettings.google_url,
        google_api_key: s.google_api_key || '',
        baidu_app_id: s.baidu_app_id || '',
        baidu_secret_key: s.baidu_secret_key || '',
        siliconflow_api_key: s.siliconflow_api_key || '',
        siliconflow_model: s.siliconflow_model || defaultSettings.siliconflow_model,
        ollama_url: s.ollama_url || defaultSettings.ollama_url,
        ollama_model: s.ollama_model || defaultSettings.ollama_model,
      });
    }).catch(console.error);
  }, [open]);

  const handleSettingsChange = async (updates: Partial<typeof settings>) => {
    const newSettings = { ...settings, ...updates };
    setSettings(newSettings);
    try {
      const current = await invoke<any>('get_settings');
      await invoke('set_settings', { settings: { ...current, ...newSettings } });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  };

  const handleThemeChange = async (newTheme: typeof theme) => {
    setTheme(newTheme);
    
    try {
      const current = await invoke<any>('get_settings');
      await invoke('set_settings', {
        settings: {
          ...current,
          theme_color: newTheme.themeColor,
          bg_color: newTheme.bgColor,
          text_color: newTheme.textColor,
          transparency: newTheme.transparency,
        },
      });
    } catch (error) {
      console.error('Failed to save theme settings:', error);
    }
  };

  const handleReset = async () => {
    setSettings(defaultSettings);
    setTheme({
      themeColor: '#789262',
      bgColor: '#3F3F3F',
      textColor: '#ffffff',
      transparency: 50,
    });
    try {
      const current = await invoke<any>('get_settings');
      await invoke('set_settings', {
        settings: {
          ...current,
          ...defaultSettings,
          theme_color: '#789262',
          bg_color: '#3F3F3F',
          text_color: '#ffffff',
          transparency: 50,
        },
      });
    } catch (error) {
      console.error('Failed to reset settings:', error);
    }
  };

  useEffect(() => {
    if (!open) return;

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        if (onOpenChange) {
          onOpenChange(false);
        } else {
          void getCurrentWindow().close();
        }
      }
    };

    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [open, onOpenChange]);

  const handleClose = async () => {
    if (onOpenChange) {
      onOpenChange(false);
      return;
    }

    await getCurrentWindow().close();
  };

  const handleSettingsHeaderDrag = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;

    const target = e.target as HTMLElement;
    if (target.closest('[data-settings-no-drag="true"]')) {
      return;
    }

    e.preventDefault();
    void getCurrentWindow().startDragging();
  };

  if (!open) {
    return null;
  }

  return (
    <div
      className="h-screen w-screen overflow-hidden"
      style={{ backgroundColor: '#212121', color: 'white' }}
    >
      <div className="h-full w-full overflow-hidden border border-white/10">
        <div
          className="h-11 flex items-center px-3 border-b border-white/10 cursor-default"
          style={{ backgroundColor: theme.themeColor }}
          onMouseDown={handleSettingsHeaderDrag}
        >
          <span className="text-sm font-medium text-white">设置</span>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              void handleClose();
            }}
            data-settings-no-drag="true"
            className="ml-auto h-8 w-8 rounded-lg text-white hover:bg-white/20"
            title="关闭设置"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        <div className="grid grid-cols-[120px_1fr] h-[calc(100%-2.75rem)]" style={{ backgroundColor: '#212121' }}>
          <aside className="border-r border-white/10 p-2 flex flex-col gap-1" style={{ backgroundColor: '#212121' }}>
            {sections.map((section) => (
              <button
                key={section.id}
                type="button"
                onClick={() => setActiveSection(section.id)}
                className={cn(
                  'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
                  activeSection === section.id ? 'bg-white/15 text-white' : 'text-white/75 hover:bg-white/10 hover:text-white'
                )}
              >
                {section.label}
              </button>
            ))}
          </aside>

          <section className="settings-scroll overflow-y-auto p-4" style={{ backgroundColor: '#212121' }}>
            {activeSection === 'general' && (
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">源语言</label>
                  <select
                    value={settings.source_lang}
                    onChange={(e) => void handleSettingsChange({ source_lang: e.target.value })}
                    className="settings-select w-full p-2 rounded text-white outline-none"
                  >
                    <option value="auto">自动检测</option>
                    <option value="zh">中文</option>
                    <option value="en">英语</option>
                    <option value="ja">日语</option>
                    <option value="ko">韩语</option>
                  </select>
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">目标语言</label>
                  <select
                    value={settings.target_lang}
                    onChange={(e) => void handleSettingsChange({ target_lang: e.target.value })}
                    className="settings-select w-full p-2 rounded text-white outline-none"
                  >
                    <option value="zh">中文</option>
                    <option value="en">英语</option>
                    <option value="ja">日语</option>
                    <option value="ko">韩语</option>
                  </select>
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">翻译引擎</label>
                  <select
                    value={settings.engine}
                    onChange={(e) => void handleSettingsChange({ engine: e.target.value })}
                    className="settings-select w-full p-2 rounded text-white outline-none"
                  >
                    <option value="google">谷歌翻译</option>
                    <option value="baidu">百度翻译</option>
                    <option value="siliconflow">硅基流动</option>
                    <option value="ollama">Ollama</option>
                  </select>
                </div>
                <button
                  type="button"
                  onClick={() => void handleReset()}
                  className="w-full p-2 rounded bg-[#212121] border border-[#9e9e9e] text-white text-sm hover:bg-white/10 transition-colors flex items-center justify-center gap-2"
                >
                  <RotateCcw className="h-4 w-4" />
                  重置设置
                </button>
              </div>
            )}

            {activeSection === 'appearance' && (
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">主题颜色</label>
                  <input
                    type="color"
                    value={theme.themeColor}
                    onChange={(e) => {
                      void handleThemeChange({ ...theme, themeColor: e.target.value });
                    }}
                    className="w-full h-10 rounded cursor-pointer"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">背景颜色</label>
                  <input
                    type="color"
                    value={theme.bgColor}
                    onChange={(e) => {
                      void handleThemeChange({ ...theme, bgColor: e.target.value });
                    }}
                    className="w-full h-10 rounded cursor-pointer"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">文本颜色</label>
                  <input
                    type="color"
                    value={theme.textColor}
                    onChange={(e) => {
                      void handleThemeChange({ ...theme, textColor: e.target.value });
                    }}
                    className="w-full h-10 rounded cursor-pointer"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">透明度: {theme.transparency}%</label>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    value={theme.transparency}
                    onChange={(e) => {
                      void handleThemeChange({ ...theme, transparency: Number(e.target.value) });
                    }}
                    className="w-full"
                  />
                </div>
              </div>
            )}

            {activeSection === 'engines' && (
              <div className="space-y-0">
                {/* 百度翻译 */}
                <div className="rounded overflow-hidden" style={{ backgroundColor: '#212121' }}>
                  <button
                    type="button"
                    onClick={() => setExpandedEngine(expandedEngine === 'baidu' ? null : 'baidu')}
                    className="group relative w-full text-left p-3 text-sm font-medium hover:bg-white/5 transition-colors flex items-center justify-between"
                  >
                    <span>百度翻译</span>
                    <span className="opacity-0 group-hover:opacity-100 transition-opacity bg-[#616161] text-white text-xs px-2 py-1 rounded absolute left-20 top-2.5 whitespace-nowrap">默认使用百度翻译开发平台API</span>
                    {expandedEngine === 'baidu' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'baidu' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="APP ID"
                          value={settings.baidu_app_id}
                          onChange={(e) => void handleSettingsChange({ baidu_app_id: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="password"
                          placeholder="Secret Key"
                          value={settings.baidu_secret_key}
                          onChange={(e) => void handleSettingsChange({ baidu_secret_key: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="翻译源 URL"
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                    </div>
                  )}
                </div>

                {/* 谷歌翻译 */}
                <div className="rounded overflow-hidden border-t border-[#9e9e9e]" style={{ backgroundColor: '#212121' }}>
                  <button
                    type="button"
                    onClick={() => setExpandedEngine(expandedEngine === 'google' ? null : 'google')}
                    className="group relative w-full text-left p-3 text-sm font-medium hover:bg-white/5 transition-colors flex items-center justify-between"
                  >
                    <span>谷歌翻译</span>
                    <span className="opacity-0 group-hover:opacity-100 transition-opacity bg-[#616161] text-white text-xs px-2 py-1 rounded absolute left-16 top-2.5 whitespace-nowrap">默认使用谷歌翻译源URL</span>
                    {expandedEngine === 'google' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'google' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="镜像 URL"
                          value={settings.google_url}
                          onChange={(e) => void handleSettingsChange({ google_url: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="password"
                          placeholder="API Key"
                          value={settings.google_api_key}
                          onChange={(e) => void handleSettingsChange({ google_api_key: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                    </div>
                  )}
                </div>

                {/* 硅基流动 */}
                <div className="rounded overflow-hidden border-t border-[#9e9e9e]" style={{ backgroundColor: '#212121' }}>
                  <button
                    type="button"
                    onClick={() => setExpandedEngine(expandedEngine === 'siliconflow' ? null : 'siliconflow')}
                    className="w-full text-left p-3 text-sm font-medium hover:bg-white/5 transition-colors flex items-center justify-between"
                  >
                    <span>硅基流动</span>
                    {expandedEngine === 'siliconflow' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'siliconflow' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="password"
                          placeholder="API Key"
                          value={settings.siliconflow_api_key}
                          onChange={(e) => void handleSettingsChange({ siliconflow_api_key: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="模型 (默认 deepseek-ai/DeepSeek-V3)"
                          value={settings.siliconflow_model}
                          onChange={(e) => void handleSettingsChange({ siliconflow_model: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                    </div>
                  )}
                </div>

                {/* Ollama */}
                <div className="rounded overflow-hidden border-t border-[#9e9e9e]" style={{ backgroundColor: '#212121' }}>
                  <button
                    type="button"
                    onClick={() => setExpandedEngine(expandedEngine === 'ollama' ? null : 'ollama')}
                    className="w-full text-left p-3 text-sm font-medium hover:bg-white/5 transition-colors flex items-center justify-between"
                  >
                    <span>Ollama</span>
                    {expandedEngine === 'ollama' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'ollama' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="URL (默认 http://localhost:11434)"
                          value={settings.ollama_url}
                          onChange={(e) => void handleSettingsChange({ ollama_url: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="模型 (默认 llama2)"
                          value={settings.ollama_model}
                          onChange={(e) => void handleSettingsChange({ ollama_model: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

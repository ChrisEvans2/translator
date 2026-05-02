import { useTheme } from '@/contexts/ThemeContext';
import { useEffect, useState, useRef } from 'react';
import { X, ChevronDown, ChevronUp, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface SettingsModalProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

const defaultSettings = {
  source_lang: 'auto',
  target_lang: 'zh',
  engine: 'google',
  image_translation_enabled: false,
  always_on_top: true,
  auto_show: true,
  google_mirror_url: 'https://translate.googleapis.com/translate_a/single',
  google_official_url: '',
  google_api_key: '',
  baidu_app_id: '',
  baidu_secret_key: '',
  llmapi_api_key: '',
  llmapi_model: 'deepseek-ai/DeepSeek-V3',
  llmapi_vlm_model: '',
  ollama_url: 'http://localhost:11434',
  ollama_model: 'llama2',
  ollama_vlm_model: '',
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
  const saveTimeoutRef = useRef<number | null>(null);
  const pendingSettingsRef = useRef<typeof settings | null>(null);
  const asideRef = useRef<HTMLElement>(null);
  const [indicatorStyle, setIndicatorStyle] = useState({ top: 8, height: 36 });
  const [hoveredTooltip, setHoveredTooltip] = useState<string | null>(null);

  useEffect(() => {
    if (!asideRef.current) return;
    const activeIndex = sections.findIndex(s => s.id === activeSection);
    const buttons = asideRef.current.querySelectorAll('button');
    const activeButton = buttons[activeIndex];
    if (activeButton) {
      setIndicatorStyle({
        top: activeButton.offsetTop,
        height: activeButton.offsetHeight
      });
    }
  }, [activeSection, sections]);

  useEffect(() => {
    if (!open) return;
    
    invoke<any>('get_settings').then((s) => {
      setSettings({
        source_lang: s.source_lang || defaultSettings.source_lang,
        target_lang: s.target_lang || defaultSettings.target_lang,
        engine: s.engine || defaultSettings.engine,
        image_translation_enabled: s.image_translation_enabled ?? defaultSettings.image_translation_enabled,
        always_on_top: s.always_on_top ?? defaultSettings.always_on_top,
        auto_show: s.auto_show ?? defaultSettings.auto_show,
        google_mirror_url: s.google_mirror_url || defaultSettings.google_mirror_url,
        google_official_url: s.google_official_url || '',
        google_api_key: s.google_api_key || '',
        baidu_app_id: s.baidu_app_id || '',
        baidu_secret_key: s.baidu_secret_key || '',
        llmapi_api_key: s.llmapi_api_key || '',
        llmapi_model: s.llmapi_model || defaultSettings.llmapi_model,
        llmapi_vlm_model: s.llmapi_vlm_model || '',
        ollama_url: s.ollama_url || defaultSettings.ollama_url,
        ollama_model: s.ollama_model || defaultSettings.ollama_model,
        ollama_vlm_model: s.ollama_vlm_model || '',
      });
    }).catch(console.error);

    const unlisten = listen('theme-changed', async () => {
      console.log('[SettingsModal] Received theme-changed event, syncing engine/lang...');
      const s = await invoke<any>('get_settings');
      console.log('[SettingsModal] Settings loaded:', { engine: s.engine, source_lang: s.source_lang, target_lang: s.target_lang });
      setSettings(prev => ({
        ...prev,
        source_lang: s.source_lang || prev.source_lang,
        target_lang: s.target_lang || prev.target_lang,
        engine: s.engine || prev.engine,
      }));
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [open]);

  const handleSettingsChange = async (updates: Partial<typeof settings>) => {
    const newSettings = { ...settings, ...updates };
    setSettings(newSettings);
    
    if (saveTimeoutRef.current !== null) {
      window.clearTimeout(saveTimeoutRef.current);
    }
    
    pendingSettingsRef.current = newSettings;
    
    const shouldSaveImmediately = 'source_lang' in updates || 'target_lang' in updates || 'engine' in updates || 'always_on_top' in updates || 'auto_show' in updates;
    const delay = shouldSaveImmediately ? 0 : 2000;
    
    saveTimeoutRef.current = window.setTimeout(async () => {
      if (pendingSettingsRef.current) {
        try {
          const current = await invoke<any>('get_settings');
          await invoke('set_settings', { settings: { ...current, ...pendingSettingsRef.current } });
          pendingSettingsRef.current = null;
        } catch (error) {
          console.error('Failed to save settings:', error);
        }
      }
    }, delay);
  };

  const handleThemeChange = async (newTheme: typeof theme) => {
    setTheme(newTheme);
    
    try {
      const current = await invoke<any>('get_settings');
      console.log('[SettingsModal] Current settings from backend:', current);
      const updatedSettings = {
        ...current,
        theme_color: newTheme.themeColor,
        bg_color: newTheme.bgColor,
        text_color: newTheme.textColor,
        transparency: newTheme.transparency,
      };
      console.log('[SettingsModal] Saving settings:', updatedSettings);
      await invoke('set_settings', { settings: updatedSettings });
      console.log('[SettingsModal] Theme saved successfully');
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
    if (saveTimeoutRef.current !== null) {
      window.clearTimeout(saveTimeoutRef.current);
    }
    
    if (pendingSettingsRef.current) {
      try {
        const current = await invoke<any>('get_settings');
        await invoke('set_settings', { settings: { ...current, ...pendingSettingsRef.current } });
      } catch (error) {
        console.error('Failed to save settings on close:', error);
      }
    }
    
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
          <aside ref={asideRef} className="border-r border-white/10 p-2 flex flex-col gap-1 relative" style={{ backgroundColor: '#212121' }}>
            <div
              className="absolute pointer-events-none"
              style={{
                left: 0,
                right: 0,
                top: `${indicatorStyle.top}px`,
                height: `${indicatorStyle.height}px`,
                borderLeft: `3px solid ${theme.themeColor}`,
                borderRight: `3px solid ${theme.themeColor}`,
                background: 'transparent',
                transition: 'top 0.25s ease, height 0.25s ease',
              }}
            />
            {sections.map((section) => (
              <button
                key={section.id}
                type="button"
                onClick={() => setActiveSection(section.id)}
                className={cn(
                  'w-full text-left px-3 py-2 rounded-md text-sm transition-colors relative z-10',
                  activeSection === section.id ? 'text-white' : 'text-white/75 hover:text-white/75'
                )}
              >
                {section.label}
              </button>
            ))}
          </aside>

          <section className="settings-scroll overflow-y-auto p-4" style={{ backgroundColor: '#212121' }}>
            {activeSection === 'general' && (
              <div className="space-y-4">
                {/* 开关按钮区域 */}
                <div className="space-y-1">
                  {/* 图片翻译 */}
                  <div className="flex items-center justify-between p-2 rounded bg-[#212121]">
                    <label className="text-sm font-medium">图片翻译 (仅LLM引擎)</label>
                    <button
                      type="button"
                      onClick={() => void handleSettingsChange({ image_translation_enabled: !settings.image_translation_enabled })}
                      className="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none"
                      style={{ backgroundColor: settings.image_translation_enabled ? theme.themeColor : undefined }}
                    >
                      {!settings.image_translation_enabled && <span className="absolute inset-0 rounded-full bg-gray-600" />}
                      <span
                        className={cn(
                          "inline-block h-4 w-4 transform rounded-full bg-white transition-transform relative z-10",
                          settings.image_translation_enabled ? "translate-x-6" : "translate-x-1"
                        )}
                      />
                    </button>
                  </div>

                  {/* 最顶层显示 */}
                  <div className="flex items-center justify-between p-2 rounded bg-[#212121]">
                    <label className="text-sm font-medium">最顶层显示</label>
                    <div className="relative">
                      <button
                        type="button"
                        onClick={() => void handleSettingsChange({ always_on_top: !settings.always_on_top })}
                        onMouseEnter={() => setHoveredTooltip('always_on_top')}
                        onMouseLeave={() => setHoveredTooltip(null)}
                        className="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none"
                        style={{ backgroundColor: settings.always_on_top ? theme.themeColor : undefined }}
                      >
                        {!settings.always_on_top && <span className="absolute inset-0 rounded-full bg-gray-600" />}
                        <span
                          className={cn(
                            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform relative z-10",
                            settings.always_on_top ? "translate-x-6" : "translate-x-1"
                          )}
                        />
                      </button>
                      {hoveredTooltip === 'always_on_top' && (
                        <div className="absolute right-0 bottom-full mb-2 px-3 py-1.5 bg-[#616161] text-white text-xs rounded shadow-lg whitespace-nowrap z-50">
                          翻译窗口始终显示在其他窗口上方
                          <div className="absolute right-3 top-full w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-[#616161]" />
                        </div>
                      )}
                    </div>
                  </div>

                  {/* 自动显示 */}
                  <div className="flex items-center justify-between p-2 rounded bg-[#212121]">
                    <label className="text-sm font-medium">自动显示</label>
                    <div className="relative">
                      <button
                        type="button"
                        onClick={() => void handleSettingsChange({ auto_show: !settings.auto_show })}
                        onMouseEnter={() => setHoveredTooltip('auto_show')}
                        onMouseLeave={() => setHoveredTooltip(null)}
                        className="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none"
                        style={{ backgroundColor: settings.auto_show ? theme.themeColor : undefined }}
                      >
                        {!settings.auto_show && <span className="absolute inset-0 rounded-full bg-gray-600" />}
                        <span
                          className={cn(
                            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform relative z-10",
                            settings.auto_show ? "translate-x-6" : "translate-x-1"
                          )}
                        />
                      </button>
                      {hoveredTooltip === 'auto_show' && (
                        <div className="absolute right-0 bottom-full mb-2 px-3 py-1.5 bg-[#616161] text-white text-xs rounded shadow-lg whitespace-nowrap z-50">
                          翻译完成时自动弹出窗口并显示翻译内容
                          <div className="absolute right-3 top-full w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-[#616161]" />
                        </div>
                      )}
                    </div>
                  </div>
                </div>

                {/* 分割线 */}
                <div className="border-t border-gray-600/50" />

                {/* 语言和引擎选择区域 */}
                <div className="space-y-2">
                  <label className="text-sm font-medium">源语言</label>
                  <div className="settings-select-wrapper">
                    <select
                      value={settings.source_lang}
                      onChange={(e) => void handleSettingsChange({ source_lang: e.target.value })}
                      className="settings-select w-full p-2 text-white outline-none"
                    >
                      <option value="auto">自动检测</option>
                      <option value="zh">中文</option>
                      <option value="en">英语</option>
                      <option value="ja">日语</option>
                      <option value="ko">韩语</option>
                    </select>
                  </div>
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">目标语言</label>
                  <div className="settings-select-wrapper">
                    <select
                      value={settings.target_lang}
                      onChange={(e) => void handleSettingsChange({ target_lang: e.target.value })}
                      className="settings-select w-full p-2 text-white outline-none"
                    >
                      <option value="zh">中文</option>
                      <option value="en">英语</option>
                      <option value="ja">日语</option>
                      <option value="ko">韩语</option>
                    </select>
                  </div>
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">翻译引擎</label>
                  <div className="settings-select-wrapper">
                    <select
                      value={settings.engine}
                      onChange={(e) => void handleSettingsChange({ engine: e.target.value })}
                      className="settings-select w-full p-2 text-white outline-none"
                    >
                      <option value="google">谷歌翻译</option>
                      <option value="baidu">百度翻译</option>
                      <option value="llmapi">大模型API</option>
                      <option value="ollama">Ollama</option>
                    </select>
                  </div>
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
                    className="w-full text-left p-3 text-sm font-medium hover:bg-white/5 transition-colors flex items-center justify-between"
                  >
                    <span>百度翻译</span>
                    {expandedEngine === 'baidu' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'baidu' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="APP ID（通用翻译API - fanyi-api.baidu.com → 开发者中心 → 开发者信息）"
                          value={settings.baidu_app_id}
                          onChange={(e) => void handleSettingsChange({ baidu_app_id: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="password"
                          placeholder="密钥（通用翻译API - fanyi-api.baidu.com → 开发者中心 → 开发者信息）"
                          value={settings.baidu_secret_key}
                          onChange={(e) => void handleSettingsChange({ baidu_secret_key: e.target.value })}
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
                    <span className="opacity-0 group-hover:opacity-100 transition-opacity bg-[#616161] text-white text-xs px-2 py-1 rounded absolute left-16 top-2.5 whitespace-nowrap">优先使用镜像源，失败后回退到官方API</span>
                    {expandedEngine === 'google' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'google' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="镜像源 URL (如 translate.googleapis.com/translate_a/single)"
                          value={settings.google_mirror_url}
                          onChange={(e) => void handleSettingsChange({ google_mirror_url: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="官方 URL (可选，默认 translation.googleapis.com/language/translate/v2)"
                          value={settings.google_official_url}
                          onChange={(e) => void handleSettingsChange({ google_official_url: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="password"
                          placeholder="API Key (官方 API 必需)"
                          value={settings.google_api_key}
                          onChange={(e) => void handleSettingsChange({ google_api_key: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                    </div>
                  )}
                </div>

                {/* 大模型API */}
                <div className="rounded overflow-hidden border-t border-[#9e9e9e]" style={{ backgroundColor: '#212121' }}>
                  <button
                    type="button"
                    onClick={() => setExpandedEngine(expandedEngine === 'llmapi' ? null : 'llmapi')}
                    className="w-full text-left p-3 text-sm font-medium hover:bg-white/5 transition-colors flex items-center justify-between"
                  >
                    <span>大模型API</span>
                    {expandedEngine === 'llmapi' ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                  </button>
                  {expandedEngine === 'llmapi' && (
                    <div className="space-y-2 p-3 pt-0">
                      <div className="engine-input-wrapper">
                        <input
                          type="password"
                          placeholder="API Key"
                          value={settings.llmapi_api_key}
                          onChange={(e) => void handleSettingsChange({ llmapi_api_key: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="使用模型"
                          value={settings.llmapi_model}
                          onChange={(e) => void handleSettingsChange({ llmapi_model: e.target.value })}
                          className="engine-input w-full p-1.5 bg-[#212121] text-white placeholder:text-gray-500 outline-none"
                        />
                      </div>
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="VLM 模型 (用于图片翻译, 可选)"
                          value={settings.llmapi_vlm_model}
                          onChange={(e) => void handleSettingsChange({ llmapi_vlm_model: e.target.value })}
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
                      <div className="engine-input-wrapper">
                        <input
                          type="text"
                          placeholder="VLM 模型 (用于图片翻译, 可选)"
                          value={settings.ollama_vlm_model}
                          onChange={(e) => void handleSettingsChange({ ollama_vlm_model: e.target.value })}
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

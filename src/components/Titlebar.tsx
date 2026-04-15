import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Settings, Copy, Minus, ChevronDown } from 'lucide-react';
import { useTheme } from '@/contexts/ThemeContext';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { hexToRgba } from '@/lib/utils';

interface TitlebarProps {
  onSettingsClick: () => void;
  onCopyClick: () => void;
  onEngineChange: (engine: string) => void;
  currentEngine: string;
}

const engines = [
  { id: 'baidu', name: '百度翻译' },
  { id: 'google', name: '谷歌翻译' },
  { id: 'llmapi', name: '大模型API' },
  { id: 'ollama', name: 'Ollama' },
];

export function Titlebar({ onSettingsClick, onCopyClick, onEngineChange, currentEngine }: TitlebarProps) {
  const { theme } = useTheme();
  const titlebarBgAlpha = 1 - theme.transparency / 200;
  const titlebarBgColor = hexToRgba(theme.themeColor, titlebarBgAlpha);

  const handleSettingsClick = () => {
    onSettingsClick();
  };

  const handleTitlebarDrag = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;

    void getCurrentWindow().startDragging().catch((error) => {
      console.error('start_dragging failed:', error);
    });
  };

  const handleMinimizeOrClose = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    e.stopPropagation();

    if (e.button === 0) {
      void getCurrentWindow().minimize().catch((error) => {
        console.error('minimize_window failed:', error);
        void invoke('minimize_window').catch((fallbackError) => {
          console.error('minimize_window fallback failed:', fallbackError);
        });
      });
      return;
    }

    if (e.button === 2) {
      void getCurrentWindow().close().catch((error) => {
        console.error('close_window failed:', error);
        void invoke('close_window').catch((fallbackError) => {
          console.error('close_window fallback failed:', fallbackError);
        });
      });
    }
  };

  const engineName = engines.find(e => e.id === currentEngine)?.name || currentEngine;

  return (
    <div
      className="h-10 flex items-center px-3 select-none relative cursor-default"
      style={{ 
        backgroundColor: titlebarBgColor,
      }}
    >
      {/* 左侧 - 设置按钮 */}
      <div className="flex items-center z-10" data-no-drag="true">
        <Button
          variant="ghost"
          size="icon"
          onClick={handleSettingsClick}
          className="h-8 w-8 text-white hover:bg-white/20"
          title="设置"
        >
          <Settings className="h-4 w-4" />
        </Button>
      </div>

      {/* 中间 - 可拖动区域 */}
      <div className="flex-1 h-full cursor-default" onMouseDown={handleTitlebarDrag} />

      {/* 右侧 - 操作按钮 */}
      <div className="flex items-center gap-1 z-10" data-no-drag="true">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="h-8 px-2 text-white hover:bg-white/20"
            >
              <span className="text-xs">{engineName}</span>
              <ChevronDown className="h-3 w-3 ml-1" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align="end"
            className="border-white/10"
            style={{ backgroundColor: titlebarBgColor, color: 'white' }}
          >
            {engines.map(engine => (
              <DropdownMenuItem 
                key={engine.id}
                onSelect={(e) => {
                  e.preventDefault();
                  onEngineChange(engine.id);
                }}
                className="focus:bg-white/20 focus:text-white"
              >
                {engine.name}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        <Button
          variant="ghost"
          size="icon"
          onClick={onCopyClick}
          className="h-8 w-8 text-white hover:bg-white/20"
          title="复制译文"
        >
          <Copy className="h-4 w-4" />
        </Button>

        <Button
          variant="ghost"
          size="icon"
          onMouseDown={handleMinimizeOrClose}
          onContextMenu={(e) => {
            e.preventDefault();
          }}
          className="h-8 w-8 text-white hover:bg-white/20"
          title="左键最小化，右键关闭"
        >
          <Minus className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

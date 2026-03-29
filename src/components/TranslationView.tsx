import { ScrollArea } from '@/components/ui/scroll-area';
import { useTheme } from '@/contexts/ThemeContext';
import { LatexRenderer } from '@/components/LatexRenderer';
import { hexToRgba } from '@/lib/utils';

interface TranslationViewProps {
  translatedText: string;
  isLoading: boolean;
  error?: string;
}

export function TranslationView({ translatedText, isLoading, error }: TranslationViewProps) {
  const { theme } = useTheme();
  const bgAlpha = 1 - theme.transparency / 100;
  const bgColorWithAlpha = hexToRgba(theme.bgColor, bgAlpha);

  return (
    <ScrollArea 
      className="h-[calc(100vh-2.5rem)]"
      style={{ 
        backgroundColor: bgColorWithAlpha,
      }}
    >
      <div className="p-4 min-h-full" style={{ color: theme.textColor }}>
        {isLoading && (
          <div className="flex items-center justify-center h-32">
            <span className="text-lg">翻译中...</span>
          </div>
        )}

        {error && (
          <div className="p-3 rounded bg-red-500/20 border border-red-500">
            <p className="text-red-400">{error}</p>
          </div>
        )}

        {!isLoading && !error && !translatedText && (
          <div className="flex items-center justify-center h-32 text-gray-400">
            <p>复制文本以开始翻译</p>
          </div>
        )}

        {!isLoading && !error && translatedText && (
          <div className="text-lg leading-relaxed">
            <LatexRenderer text={translatedText} />
          </div>
        )}
      </div>
    </ScrollArea>
  );
}

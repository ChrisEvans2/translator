import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize, PhysicalPosition } from '@tauri-apps/api/dpi';
import { X, Copy, Check } from 'lucide-react';

interface SelectionPayload {
  text: string;
  x: number;
  y: number;
}

export function SelectionPopup() {
  const [text, setText] = useState('');
  const [translated, setTranslated] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [copied, setCopied] = useState(false);
  const autoCloseRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  const resizeWindow = () => {
    requestAnimationFrame(() => {
      if (contentRef.current) {
        const rect = contentRef.current.getBoundingClientRect();
        const w = Math.ceil(rect.width);
        const h = Math.ceil(rect.height);
        if (w > 0 && h > 0) {
          getCurrentWindow().setSize(new LogicalSize(w, h)).catch(() => {});
        }
      }
    });
  };

  useEffect(() => {
    const win = getCurrentWindow();

    const handlePayload = async (payload: SelectionPayload) => {
      if (autoCloseRef.current) {
        clearTimeout(autoCloseRef.current);
        autoCloseRef.current = null;
      }

      setText(payload.text);
      setTranslated('');
      setError('');
      setLoading(true);
      setCopied(false);

      win.setPosition(new PhysicalPosition(Math.round(payload.x + 10), Math.round(payload.y + 10))).catch(() => {});

      try {
        const settings = await invoke<{
          engine: string;
          source_lang: string;
          target_lang: string;
          selection_auto_close_ms: number;
        }>('get_settings');

        const result = await invoke<{ text: string; error?: string }>('translate', {
          text: payload.text,
          from: settings.source_lang || 'auto',
          to: settings.target_lang || 'zh',
          engine: settings.engine || 'google',
        });

        if (result.error) {
          setError(result.error);
        } else {
          setTranslated(result.text);
        }

        if (settings.selection_auto_close_ms > 0) {
          if (autoCloseRef.current) clearTimeout(autoCloseRef.current);
          autoCloseRef.current = setTimeout(() => {
            void win.hide();
          }, settings.selection_auto_close_ms);
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };

    invoke<SelectionPayload | null>('get_last_selection_payload')
      .then((payload) => {
        if (payload) {
          void handlePayload(payload);
        }
      })
      .catch(() => {});

    const unlistenText = win.listen<SelectionPayload>('selection-text-ready', (event) => {
      void handlePayload(event.payload);
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        void win.hide();
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      unlistenText.then(fn => fn());
      window.removeEventListener('keydown', handleKeyDown);
      if (autoCloseRef.current) clearTimeout(autoCloseRef.current);
    };
  }, []);

  useEffect(() => {
    resizeWindow();
  }, [text, translated, loading, error]);

  const handleCopy = async () => {
    if (translated) {
      await writeText(translated);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  };

  const handleClose = () => {
    void getCurrentWindow().hide();
  };

  if (!text && !loading && !error) return null;

  return (
    <div
      ref={contentRef}
      style={{
        backgroundColor: 'rgba(42, 42, 42, 0.95)',
        minWidth: 120,
        maxWidth: 420,
        borderRadius: 8,
        overflow: 'hidden',
        boxShadow: '0 4px 24px rgba(0,0,0,0.4)',
        border: '1px solid rgba(255,255,255,0.1)',
      }}
    >
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '6px 10px',
        borderBottom: '1px solid rgba(255,255,255,0.1)',
        backgroundColor: 'rgba(51,51,51,0.9)',
        gap: 8,
      }}>
        <span style={{
          fontSize: 11,
          color: '#9ca3af',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          maxWidth: 300,
        }}>{text}</span>
        <div style={{ display: 'flex', gap: 2, flexShrink: 0 }}>
          <button
            onClick={() => void handleCopy()}
            style={{ padding: 2, borderRadius: 4, border: 'none', background: 'transparent', cursor: 'pointer', color: '#9ca3af', display: 'flex', alignItems: 'center' }}
            title="复制翻译"
          >
            {copied ? <Check size={14} color="#4ade80" /> : <Copy size={14} />}
          </button>
          <button
            onClick={handleClose}
            style={{ padding: 2, borderRadius: 4, border: 'none', background: 'transparent', cursor: 'pointer', color: '#9ca3af', display: 'flex', alignItems: 'center' }}
            title="关闭"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      <div style={{ padding: '10px 12px', minHeight: 24 }}>
        {loading && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: '#9ca3af', fontSize: 13 }}>
            <div style={{
              width: 12, height: 12,
              border: '2px solid #6b7280', borderTopColor: 'transparent',
              borderRadius: '50%', animation: 'spin 0.6s linear infinite',
            }} />
            翻译中...
            <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
          </div>
        )}
        {error && <div style={{ color: '#f87171', fontSize: 13 }}>{error}</div>}
        {translated && !loading && (
          <div style={{ color: 'white', fontSize: 13, whiteSpace: 'pre-wrap', wordBreak: 'break-word', lineHeight: 1.5 }}>
            {translated}
          </div>
        )}
      </div>
    </div>
  );
}

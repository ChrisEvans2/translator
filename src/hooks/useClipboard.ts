import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

export interface ClipboardPayload {
  text: string;
}

export function useClipboard() {
  const [clipboardText, setClipboardText] = useState<string>('');

  useEffect(() => {
    const unlisten = listen<ClipboardPayload>('clipboard-changed', (event) => {
      setClipboardText(event.payload.text);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return clipboardText;
}

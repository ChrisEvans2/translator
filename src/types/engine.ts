export interface TranslationResult {
  text: string;
  engine: string;
  error?: string;
}

export type Engine = 'baidu' | 'google' | 'llmapi' | 'ollama';

export interface EngineConfig {
  baidu: {
    appId: string;
    secretKey: string;
  };
  google: {
    url: string;
    apiKey: string;
  };
  llmapi: {
    apiKey: string;
    model: string;
  };
  ollama: {
    url: string;
    model: string;
  };
}

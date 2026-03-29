export interface LatexSegment {
  original: string;
  placeholder: string;
  type: 'inline' | 'block';
}

const LATEX_INLINE_REGEX = /\$([^$\n]+?)\$/g;
const LATEX_BLOCK_REGEX = /\$\$([\s\S]+?)\$\$/g;
const ESCAPED_DOLLAR = /\\\$/g;

export function detectLatex(text: string): LatexSegment[] {
  const segments: LatexSegment[] = [];
  let placeholderIndex = 0;

  const escaped = text.replace(ESCAPED_DOLLAR, '\u0000');

  let match: RegExpExecArray | null;
  while ((match = LATEX_BLOCK_REGEX.exec(escaped)) !== null) {
    segments.push({
      original: match[0],
      placeholder: `[[LATEX_BLOCK_${placeholderIndex}]]`,
      type: 'block',
    });
    placeholderIndex++;
  }

  LATEX_BLOCK_REGEX.lastIndex = 0;
  while ((match = LATEX_INLINE_REGEX.exec(escaped)) !== null) {
    if (!segments.some(s => s.original.includes(match![0]))) {
      segments.push({
        original: match[0],
        placeholder: `[[LATEX_INLINE_${placeholderIndex}]]`,
        type: 'inline',
      });
      placeholderIndex++;
    }
  }

  return segments;
}

export function extractLatex(text: string): { plainText: string; segments: LatexSegment[] } {
  const segments = detectLatex(text);
  let plainText = text;

  for (const segment of segments) {
    plainText = plainText.replace(segment.original, segment.placeholder);
  }

  return { plainText, segments };
}

export function reinsertLatex(translatedText: string, segments: LatexSegment[]): string {
  let result = translatedText;

  for (const segment of segments) {
    result = result.replace(segment.placeholder, segment.original);
  }

  return result;
}

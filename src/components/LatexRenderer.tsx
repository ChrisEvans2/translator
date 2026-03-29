import { InlineMath, BlockMath } from 'react-katex';
import { detectLatex } from '@/lib/latex';

interface LatexRendererProps {
  text: string;
}

export function LatexRenderer({ text }: LatexRendererProps) {
  const segments = detectLatex(text);
  
  if (segments.length === 0) {
    return <span>{text}</span>;
  }

  const parts: React.ReactNode[] = [];
  let lastIndex = 0;

  for (const segment of segments) {
    const index = text.indexOf(segment.original, lastIndex);
    if (index > lastIndex) {
      parts.push(<span key={`text-${lastIndex}`}>{text.slice(lastIndex, index)}</span>);
    }

    try {
      const latexContent = segment.type === 'block' 
        ? segment.original.slice(2, -2)
        : segment.original.slice(1, -1);
      
      if (segment.type === 'block') {
        parts.push(<BlockMath key={`latex-${index}`}>{latexContent}</BlockMath>);
      } else {
        parts.push(<InlineMath key={`latex-${index}`}>{latexContent}</InlineMath>);
      }
    } catch {
      parts.push(<span key={`latex-${index}`} className="text-red-500">{segment.original}</span>);
    }

    lastIndex = index + segment.original.length;
  }

  if (lastIndex < text.length) {
    parts.push(<span key={`text-${lastIndex}`}>{text.slice(lastIndex)}</span>);
  }

  return <>{parts}</>;
}

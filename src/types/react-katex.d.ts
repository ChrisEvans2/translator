declare module 'react-katex' {
  import { ReactNode } from 'react';
  export interface MathProps {
    children?: ReactNode;
    math?: string;
  }
  export function InlineMath(props: MathProps): React.ReactElement;
  export function BlockMath(props: MathProps): React.ReactElement;
}

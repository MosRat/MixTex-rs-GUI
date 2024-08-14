declare module 'katex/dist/contrib/auto-render.mjs' {
    const renderMathInElement: (element: HTMLElement | null, options?: any) => void;
    export default renderMathInElement;
}
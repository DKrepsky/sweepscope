export type TokenMap = Record<string, string>;
export function parseTokenBlock(css: string, selector: string): TokenMap;
export function parseThemeNames(css: string): string[];
export function parseReducedMotionTokens(css: string): TokenMap;
export function generate(css: string): string;
export function readTokenSources(): { frontendCss: string; prototypeCss: string; generatedTs: string };
export interface GeneratorIo {
  read(path: string): string;
  write(path: string, content: string): void;
  error(message: string): void;
}
export function main(args: string[], io?: GeneratorIo): number;

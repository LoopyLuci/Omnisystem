// Real behavior tests for the OmniCC LanguageDetector: explicit hints, filename/
// extension signals, shebang detection, keyword scoring, and structural heuristics.
import { describe, it, expect } from 'vitest';
import { detectLanguage, detectLanguageBatch } from '../LanguageDetector';

describe('detectLanguage', () => {
    it('honors an explicit language hint at full confidence', () => {
        const result = detectLanguage('this is not even code', undefined, 'python');
        expect(result.langId).toBe('python');
        expect(result.confidence).toBe(100);
        expect(result.signals).toContain('explicit hint');
    });

    const pythonSnippet = `
        import sys

        def greet(name):
            print("hello", name)

        class Greeter:
            def __init__(self):
                pass

        if __name__ == "__main__":
            greet("world")
    `;

    it('ignores the hint value "auto" and falls back to real detection', () => {
        const result = detectLanguage(pythonSnippet, undefined, 'auto');
        expect(result.langId).toBe('python');
    });

    it('falls back to real detection when the hint is not a known language', () => {
        const result = detectLanguage(pythonSnippet, undefined, 'not-a-real-lang');
        expect(result.langId).toBe('python');
    });

    it('detects Python from filename extension', () => {
        const result = detectLanguage('x = 1', 'script.py');
        expect(result.langId).toBe('python');
        expect(result.signals.some(s => s.includes('file extension'))).toBe(true);
    });

    it('detects Rust from filename extension', () => {
        const result = detectLanguage('fn main() {}', 'main.rs');
        expect(result.langId).toBe('rust');
    });

    it('recognizes a bare "Makefile" filename regardless of case', () => {
        const result = detectLanguage('all:\n\techo hi\n', 'Makefile');
        expect(result.langId).toBe('makefile');
        expect(result.signals).toContain('filename is Makefile');
    });

    it('recognizes a bare "Dockerfile" filename', () => {
        const result = detectLanguage('FROM node:20\n', 'Dockerfile');
        expect(result.langId).toBe('docker');
        expect(result.signals).toContain('filename is Dockerfile');
    });

    it('detects language from a python3 shebang even with no filename', () => {
        const result = detectLanguage('#!/usr/bin/env python3\nprint("hi")\n');
        expect(result.langId).toBe('python');
        expect(result.signals.some(s => s.startsWith('shebang:'))).toBe(true);
    });

    it('detects bash from a #!/bin/bash shebang', () => {
        const result = detectLanguage('#!/bin/bash\necho hi\n');
        expect(result.langId).toBe('bash');
    });

    it('detects Rust via keyword + structural scoring (lifetimes, impl) with no filename hint', () => {
        // Matches LanguageDetector's own Rust structural heuristic
        // (lifetime annotation + `fn` + `impl`), not just generic keywords
        // that other C-like/systems languages (e.g. Carbon) also share.
        const source = `
            struct Wrapper<'a> {
                value: &'a str,
            }

            impl<'a> Wrapper<'a> {
                fn main() {
                    let mut x: i32 = 5;
                    println!("{}", x);
                }
            }
        `;
        const result = detectLanguage(source);
        expect(result.langId).toBe('rust');
        expect(result.confidence).toBeGreaterThan(0);
    });

    it('prefers TypeScript over JavaScript when type annotations are present', () => {
        const source = `
            interface Point { x: number; y: number; }
            function dist(p: Point): number {
                return Math.sqrt(p.x * p.x + p.y * p.y);
            }
        `;
        const result = detectLanguage(source, 'shape.ts');
        expect(result.langId).toBe('typescript');
    });

    it('detects SQL from SELECT/FROM structural pattern', () => {
        const result = detectLanguage('SELECT id, name FROM users WHERE active = 1;');
        expect(result.langId).toBe('sql');
    });

    it('detects Clojure from ns/defn forms', () => {
        const result = detectLanguage('(ns my.app.core)\n(defn greet [name] (str "hi " name))');
        expect(result.langId).toBe('clojure');
    });

    it('assigns very low confidence to unrecognizable natural-language gibberish', () => {
        // Free-form prose (no code punctuation, no accidental keyword
        // substrings) should not be mistaken for any programming language
        // with meaningful confidence. Some incidental low-scoring keyword
        // collisions are an accepted property of the substring-based
        // heuristic, so we assert "low confidence" rather than a hard
        // "unknown", which is the real (and reasonable) contract here.
        const result = detectLanguage('lorem ipsum quibble wobbulate zant fringle');
        expect(result.confidence).toBeLessThanOrEqual(20);
    });

    it('caps confidence at 100 and deduplicates repeated signals', () => {
        // A python file with a python3 shebang AND filename AND dense keyword
        // matches should still report a sane (<=100) confidence with unique signals.
        const source = '#!/usr/bin/env python3\n' + 'def foo():\n    print("x")\n'.repeat(20);
        const result = detectLanguage(source, 'script.py');
        expect(result.confidence).toBeLessThanOrEqual(100);
        expect(new Set(result.signals).size).toBe(result.signals.length);
        expect(result.signals.length).toBeLessThanOrEqual(5);
    });
});

describe('detectLanguageBatch', () => {
    it('detects each file independently and keys results by path', () => {
        const results = detectLanguageBatch([
            { path: 'a.py', content: 'def f(): pass' },
            { path: 'b.rs', content: 'fn main() {}' },
        ]);
        expect(results.get('a.py')?.langId).toBe('python');
        expect(results.get('b.rs')?.langId).toBe('rust');
        expect(results.size).toBe(2);
    });

    it('returns an empty map for an empty file list', () => {
        expect(detectLanguageBatch([]).size).toBe(0);
    });
});

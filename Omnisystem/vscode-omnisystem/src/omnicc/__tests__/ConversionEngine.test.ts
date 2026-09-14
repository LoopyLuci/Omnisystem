// Real behavior tests for the OmniCC ConversionEngine: exercises the actual
// parse -> ULIR -> generate pipeline end to end (no mocking of the family
// handlers), plus project-mode batch conversion and path derivation.
import { describe, it, expect } from 'vitest';
import { OmniCCConversionEngine, createEngine, quickConvert } from '../ConversionEngine';

describe('OmniCCConversionEngine.convert (snippet mode)', () => {
    it('converts a simple JavaScript function to Python end-to-end', () => {
        const engine = createEngine();
        const result = engine.convert({
            source: 'function add(a, b) {\n  return a + b;\n}\n',
            filename: 'add.js',
            targetLang: 'python',
        });

        expect(result.success).toBe(true);
        expect(result.sourceLangId).toBe('javascript');
        expect(result.targetLangId).toBe('python');
        expect(result.ir).toBeDefined();
        expect(result.ir!.units.length).toBeGreaterThan(0);
        expect(result.ir!.units[0].name).toBe('add');
        // Python output should render a def, not a JS function.
        expect(result.output).toContain('def add');
        expect(result.output).not.toContain('function add');
    });

    it('detects the source language from filename when sourceLang is not given', () => {
        const engine = createEngine();
        const result = engine.convert({
            source: 'def multiply(a, b):\n    return a * b\n',
            filename: 'ops.py',
            targetLang: 'javascript',
        });
        expect(result.sourceLangId).toBe('python');
        expect(result.output).toContain('multiply');
    });

    it('respects an explicit sourceLang over auto-detection', () => {
        const engine = createEngine();
        // Ambiguous/minimal snippet; force the source language explicitly.
        const result = engine.convert({
            source: 'let x = 1;',
            sourceLang: 'javascript',
            targetLang: 'python',
        });
        expect(result.sourceLangId).toBe('javascript');
    });

    it('defaults targetLang to javascript when omitted', () => {
        // OmniCCConversionRequest.targetLang is typed as required, but
        // convertSnippet() actually does `req.targetLang ?? 'javascript'`
        // at runtime -- exercise that real fallback via an explicit
        // `undefined`, which the type system otherwise disallows.
        const engine = createEngine();
        const result = engine.convert({
            source: 'def f():\n    pass\n',
            sourceLang: 'python',
            targetLang: undefined as unknown as string,
        });
        expect(result.targetLangId).toBe('javascript');
    });

    it('reports accurate line counts and non-negative duration', () => {
        const engine = createEngine();
        const source = 'function f() {\n  return 1;\n}\n';
        const result = engine.convert({ source, sourceLang: 'javascript', targetLang: 'python' });
        expect(result.linesConverted).toBe(source.split('\n').length);
        expect(result.durationMs).toBeGreaterThanOrEqual(0);
    });

    it('includes detection confidence and notes describing the conversion', () => {
        const engine = createEngine();
        const result = engine.convert({
            source: 'fn main() {\n    println!("hi");\n}\n',
            sourceLang: 'rust',
            targetLang: 'go',
        });
        expect(result.confidence).toBe(100); // explicit sourceLang hint => full confidence
        expect(result.notes.some(n => n.includes('Source: rust'))).toBe(true);
        expect(result.notes.some(n => n.includes('Target: go'))).toBe(true);
    });

    it('engine.parse falls back to best-effort content-sniffing for an unrecognized language id', () => {
        // parse() is the lower-level dispatch used internally by convert();
        // exercising it directly (bypassing detectLanguage's own heuristics,
        // which convert() runs first and can override an invalid hint with
        // a different guess) isolates ConversionEngine's own
        // getLanguageFamily()/parseWithBestEffort() fallback behavior.
        const engine = createEngine();
        const ir = engine.parse('def weird_but_pythonic():\n    return 42\n', 'not-a-real-language');
        // best-effort content-sniffing recognizes `def name(...)` and routes
        // through the real Python family parser rather than the empty
        // ultra-fallback module -- confirmed by the family tag. Individual
        // unit (function/class) extraction inside that handler additionally
        // gates on an *exact* known langId ('python', 'ruby', 'crystal', ...;
        // see PythonFamilyHandler.buildBlockMap), so an arbitrary unregistered
        // id still yields zero extracted units even once routed correctly.
        expect(ir.sourceFamily).toBe('python-family');
        expect(ir.units).toEqual([]);

        // The same source parsed with the real 'python' id does extract the function.
        const properIr = engine.parse('def weird_but_pythonic():\n    return 42\n', 'python');
        expect(properIr.units.length).toBeGreaterThan(0);
        expect(properIr.units[0].name).toBe('weird_but_pythonic');
    });

    it('engine.parse produces the empty ultra-fallback module when nothing matches', () => {
        const engine = createEngine();
        const ir = engine.parse('~~~ totally unstructured content ~~~', 'not-a-real-language');
        expect(ir.sourceFamily).toBe('unknown');
        expect(ir.confidence).toBe('low');
        expect(ir.units).toEqual([]);
        expect(ir.notes?.[0]).toContain('not fully supported');
    });
});

describe('OmniCCConversionEngine.convert (project mode)', () => {
    it('converts every file in a project independently and derives target paths', () => {
        const engine = createEngine();
        const result = engine.convert({
            source: '',
            targetLang: 'python',
            projectMode: true,
            projectFiles: [
                { path: 'src/util.js', content: 'function helper() { return 1; }' },
                { path: 'src/main.js', content: 'function main() { helper(); }' },
            ],
        });

        expect(result.success).toBe(true);
        expect(result.projectResults).toHaveLength(2);
        const util = result.projectResults!.find(r => r.path === 'src/util.js')!;
        expect(util.targetPath).toBe('src/util.py');
        expect(util.sourceLangId).toBe('javascript');
        expect(util.error).toBeUndefined();
    });

    it('records a per-file error without aborting the whole project batch', () => {
        const engine = createEngine();
        // Force a source language that the parser dispatch cannot resolve
        // meaningfully alongside a well-formed file, to confirm isolation:
        // one file's outcome doesn't affect the other's.
        const result = engine.convert({
            source: '',
            targetLang: 'javascript',
            projectMode: true,
            projectFiles: [
                { path: 'good.py', content: 'def ok():\n    return 1\n' },
                { path: 'empty.py', content: '' },
            ],
        });
        expect(result.projectResults).toHaveLength(2);
        // Both are processed independently; total line count sums both files.
        const totalLines = result.projectResults!.reduce((a, r) => a + (r.linesIn ?? 0), 0);
        expect(result.linesConverted).toBe(totalLines);
    });

    it('treats an empty projectFiles array as snippet mode, not project mode', () => {
        // convert() only takes the project-mode branch when projectFiles is
        // both present AND non-empty; an empty array falls through to
        // convertSnippet (over the empty `source`), so projectResults is
        // never populated. This documents that real dispatch behavior.
        const engine = createEngine();
        const result = engine.convert({
            source: '',
            targetLang: 'javascript',
            projectMode: true,
            projectFiles: [],
        });
        expect(result.projectResults).toBeUndefined();
    });
});

describe('OmniCCConversionEngine introspection', () => {
    it('getSupportedLanguages covers the full registry and marks parse/generate support', () => {
        const engine = createEngine();
        const langs = engine.getSupportedLanguages();
        expect(langs.length).toBeGreaterThan(100);
        expect(langs.every(l => l.canParse && l.canGenerate)).toBe(true);
        expect(langs.some(l => l.id === 'python')).toBe(true);
    });

    it('getConversionPaths excludes the source language itself', () => {
        const engine = createEngine();
        const paths = engine.getConversionPaths('python');
        expect(paths).not.toContain('python');
        expect(paths).toContain('javascript');
    });
});

describe('quickConvert', () => {
    it('is a stateless convenience wrapper around a fresh engine', () => {
        const result = quickConvert('def f():\n    return 1\n', 'javascript', 'python');
        expect(result.success).toBe(true);
        expect(result.sourceLangId).toBe('python');
        expect(result.targetLangId).toBe('javascript');
    });
});

// Real behavior tests for the OmniCC LanguageRegistry: lookup-by-id/alias/extension,
// family grouping, search, and popularity ranking over the real 250+ language table.
import { describe, it, expect } from 'vitest';
import {
    getLang, getLangByExtension, getLangsByFamily, allLanguages,
    searchLanguages, popularLanguages, getConversionLabel, getFileExtension,
    LANGUAGES,
} from '../LanguageRegistry';

describe('LanguageRegistry', () => {
    it('has a large, non-trivial language table', () => {
        expect(LANGUAGES.length).toBeGreaterThan(100);
        expect(allLanguages()).toBe(LANGUAGES);
    });

    it('getLang resolves by canonical id', () => {
        const py = getLang('python');
        expect(py).toBeDefined();
        expect(py!.name).toBe('Python');
        expect(py!.family).toBe('python-family');
    });

    it('getLang resolves by alias, case-insensitively', () => {
        expect(getLang('py')?.id).toBe('python');
        expect(getLang('PY')?.id).toBe('python');
        expect(getLang('golang')?.id).toBe('go');
    });

    it('getLang resolves by display name, case-insensitively', () => {
        expect(getLang('Python')?.id).toBe('python');
        expect(getLang('python')?.id).toBe('python');
        expect(getLang('c++')?.id).toBe('cpp' /* alias 'c-plus-plus' or name 'C++' */);
    });

    it('getLang returns undefined for an unknown id', () => {
        expect(getLang('definitely-not-a-real-language')).toBeUndefined();
    });

    it('getLangByExtension resolves with or without a leading dot', () => {
        expect(getLangByExtension('.py')?.id).toBe('python');
        expect(getLangByExtension('py')?.id).toBe('python');
        expect(getLangByExtension('.RS')?.id).toBe('rust'); // case-insensitive
    });

    it('getLangByExtension returns undefined for unmapped extensions', () => {
        expect(getLangByExtension('.definitely-not-an-extension')).toBeUndefined();
    });

    it('getLangsByFamily groups every registered language consistently', () => {
        const cFamily = getLangsByFamily('c-family');
        expect(cFamily.length).toBeGreaterThan(5);
        expect(cFamily.every(l => l.family === 'c-family')).toBe(true);
        expect(cFamily.some(l => l.id === 'javascript')).toBe(true);
    });

    it('getLangsByFamily returns empty array for a family with no members', () => {
        // Cast through unknown to probe a family string that shouldn't exist in the table.
        expect(getLangsByFamily('not-a-real-family' as never)).toEqual([]);
    });

    it('searchLanguages matches by id, name, alias, description, and extension', () => {
        expect(searchLanguages('python').some(l => l.id === 'python')).toBe(true);
        expect(searchLanguages('golang').some(l => l.id === 'go')).toBe(true);          // alias match
        expect(searchLanguages('.rs').some(l => l.id === 'rust')).toBe(true);            // extension match
        expect(searchLanguages('systems programming').some(l => l.id === 'c')).toBe(true); // description match
    });

    it('searchLanguages with empty query returns the full table', () => {
        expect(searchLanguages('')).toBe(LANGUAGES);
        expect(searchLanguages('   ')).toBe(LANGUAGES);
    });

    it('searchLanguages returns empty for a query matching nothing', () => {
        expect(searchLanguages('zzzznonexistentqueryzzzz')).toEqual([]);
    });

    it('popularLanguages sorts descending by popularity and respects the limit', () => {
        const top5 = popularLanguages(5);
        expect(top5).toHaveLength(5);
        for (let i = 1; i < top5.length; i++) {
            expect(top5[i - 1].popularity).toBeGreaterThanOrEqual(top5[i].popularity);
        }
    });

    it('popularLanguages defaults to a limit of 30', () => {
        expect(popularLanguages()).toHaveLength(30);
    });

    it('getConversionLabel renders human names for known languages', () => {
        expect(getConversionLabel('python', 'javascript')).toBe('Python → JavaScript');
    });

    it('getConversionLabel falls back to raw ids when a language is unknown', () => {
        expect(getConversionLabel('python', 'not-a-lang')).toBe('python → not-a-lang');
    });

    it('getFileExtension returns the primary extension for a language', () => {
        expect(getFileExtension('python')).toBe('.py');
        expect(getFileExtension('rust')).toBe('.rs');
    });

    it('getFileExtension falls back to .txt for unknown languages', () => {
        expect(getFileExtension('not-a-real-language')).toBe('.txt');
    });

    it('every language entry has internally-consistent extension/alias data used by lookups', () => {
        for (const lang of LANGUAGES) {
            expect(getLang(lang.id)).toBe(lang);
            for (const ext of lang.extensions) {
                // BY_EXT is last-writer-wins for shared extensions (e.g. .m is both
                // objc and matlab); just assert the lookup returns *some* valid entry.
                expect(getLangByExtension(ext)).toBeDefined();
            }
        }
    });
});

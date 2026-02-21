import { describe, it, expect } from 'vitest';
import { generateId } from './index';

describe('utils/index', () => {
    describe('generateId', () => {
        it('converts a title to a valid ID', () => {
            expect(generateId('My Awesome Feature')).toBe('my-awesome-feature');
        });

        it('removes special characters', () => {
            expect(generateId('Feature! @With# $Spec1al %Chars^')).toBe('feature-with-spec1al-chars');
        });

        it('handles leading and trailing hyphens correctly', () => {
            expect(generateId('---Feature Name---')).toBe('feature-name');
        });

        it('handles multiple consecutive spaces', () => {
            expect(generateId('Feature    Name')).toBe('feature-name');
        });
    });
});

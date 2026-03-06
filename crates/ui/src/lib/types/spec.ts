import type { Spec as RawSpec, SpecEntry as RawSpecEntry } from '@/bindings';

export interface SpecInfo extends RawSpecEntry { }

export interface SpecDocument extends RawSpec { }

export function toSpecInfo(entry: RawSpecEntry): SpecInfo {
  return entry;
}

export function toSpecDocument(entry: RawSpec): SpecDocument {
  return entry;
}

export function stubSpecDocument(entry: SpecInfo, content = ''): SpecDocument {
  return {
    ...entry.spec,
    body: content,
  };
}

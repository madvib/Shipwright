import type { Spec, SpecEntry as RawSpecEntry } from '@/bindings';

export interface SpecInfo {
  id: string;
  file_name: string;
  title: string;
  path: string;
  status: string;
}

export interface SpecDocument extends SpecInfo {
  content: string;
  spec: Spec;
}

export function toSpecInfo(entry: RawSpecEntry): SpecInfo {
  return {
    id: entry.id,
    file_name: entry.file_name,
    title: entry.spec?.metadata?.title ?? entry.file_name,
    path: entry.path,
    status: entry.status,
  };
}

export function toSpecDocument(entry: RawSpecEntry): SpecDocument {
  return {
    ...toSpecInfo(entry),
    content: entry.spec?.body ?? '',
    spec: entry.spec,
  };
}

export function stubSpecDocument(entry: SpecInfo, content = ''): SpecDocument {
  return {
    ...entry,
    content,
    spec: {
      metadata: {
        id: entry.id,
        title: entry.title,
        created: '',
        updated: '',
      },
      body: content,
    },
  };
}

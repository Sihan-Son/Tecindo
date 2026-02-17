import { create } from 'zustand';
import type { Tag } from '@/lib/types';
import * as tagsApi from '@/api/tags';

const TAG_COLORS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899', '#6b7280'];

interface TagStore {
  tags: Tag[];
  documentTags: Record<string, Tag[]>;
  loading: boolean;
  error: string | null;

  loadTags: () => Promise<void>;
  createTag: (data: { name: string; color?: string }) => Promise<Tag>;
  updateTag: (id: string, data: { name?: string; color?: string }) => Promise<void>;
  deleteTag: (id: string) => Promise<void>;
  loadDocumentTags: (documentId: string) => Promise<void>;
  addTagToDocument: (documentId: string, tagId: string) => Promise<void>;
  removeTagFromDocument: (documentId: string, tagId: string) => Promise<void>;
  findOrCreateTag: (name: string) => Promise<Tag>;
}

export const useTagStore = create<TagStore>((set, get) => ({
  tags: [],
  documentTags: {},
  loading: false,
  error: null,

  loadTags: async () => {
    try {
      set({ loading: true, error: null });
      const tags = await tagsApi.fetchTags();
      set({ tags, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  createTag: async (data) => {
    try {
      set({ loading: true, error: null });
      const tag = await tagsApi.createTag(data);
      set((state) => ({ tags: [...state.tags, tag], loading: false }));
      return tag;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateTag: async (id, data) => {
    try {
      const tag = await tagsApi.updateTag(id, data);
      set((state) => ({
        tags: state.tags.map((t) => (t.id === id ? tag : t)),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  deleteTag: async (id) => {
    try {
      await tagsApi.deleteTag(id);
      set((state) => ({
        tags: state.tags.filter((t) => t.id !== id),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  loadDocumentTags: async (documentId) => {
    try {
      const tags = await tagsApi.fetchDocumentTags(documentId);
      set((state) => ({
        documentTags: { ...state.documentTags, [documentId]: tags },
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  addTagToDocument: async (documentId, tagId) => {
    try {
      await tagsApi.addTagToDocument(documentId, tagId);
      const { tags, documentTags } = get();
      const tag = tags.find((t) => t.id === tagId);
      if (tag) {
        const current = documentTags[documentId] || [];
        set({
          documentTags: { ...documentTags, [documentId]: [...current, tag] },
        });
      }
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  removeTagFromDocument: async (documentId, tagId) => {
    try {
      await tagsApi.removeTagFromDocument(documentId, tagId);
      set((state) => ({
        documentTags: {
          ...state.documentTags,
          [documentId]: (state.documentTags[documentId] || []).filter((t) => t.id !== tagId),
        },
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  findOrCreateTag: async (name: string) => {
    const { tags } = get();
    const existing = tags.find((t) => t.name.toLowerCase() === name.toLowerCase());
    if (existing) return existing;
    const hash = name.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0);
    const color = TAG_COLORS[hash % TAG_COLORS.length];
    return get().createTag({ name, color });
  },
}));

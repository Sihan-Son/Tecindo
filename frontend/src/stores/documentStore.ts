import { create } from 'zustand';
import type { Document, Folder } from '@/lib/types';
import * as api from '@/api/client';

interface DocumentStore {
  documents: Document[];
  currentDocument: Document | null;
  currentContent: string;
  loading: boolean;
  error: string | null;

  folders: Folder[];
  currentFolderId: string | null;

  loadDocuments: () => Promise<void>;
  loadDocument: (id: string) => Promise<void>;
  createDocument: (data?: { title?: string; folder_id?: string }) => Promise<Document>;
  updateDocument: (id: string, data: { title?: string; folder_id?: string | null; is_pinned?: boolean; is_archived?: boolean }) => Promise<void>;
  deleteDocument: (id: string) => Promise<void>;
  loadContent: (id: string) => Promise<void>;
  saveContent: (id: string, content: string) => Promise<void>;

  loadFolders: () => Promise<void>;
  createFolder: (data: { name: string; parent_id?: string }) => Promise<Folder>;
  updateFolder: (id: string, data: { name?: string; parent_id?: string; sort_order?: number }) => Promise<void>;
  deleteFolder: (id: string) => Promise<void>;
  setCurrentFolderId: (id: string | null) => void;
}

export const useDocumentStore = create<DocumentStore>((set, get) => ({
  documents: [],
  currentDocument: null,
  currentContent: '',
  loading: false,
  error: null,

  folders: [],
  currentFolderId: null,

  loadDocuments: async () => {
    try {
      set({ loading: true, error: null });
      const documents = await api.fetchDocuments();
      set({ documents, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  loadDocument: async (id: string) => {
    try {
      set({ loading: true, error: null });
      const document = await api.fetchDocument(id);
      set({ currentDocument: document, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  createDocument: async (data) => {
    try {
      set({ loading: true, error: null });
      const document = await api.createDocument(data);
      set((state) => ({
        documents: [document, ...state.documents],
        currentDocument: document,
        currentContent: '',
        loading: false,
      }));
      return document;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateDocument: async (id, data) => {
    try {
      const document = await api.updateDocument(id, data);
      set((state) => ({
        documents: state.documents.map((d) => (d.id === id ? document : d)),
        currentDocument: state.currentDocument?.id === id ? document : state.currentDocument,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  deleteDocument: async (id) => {
    try {
      await api.deleteDocument(id);
      set((state) => ({
        documents: state.documents.filter((d) => d.id !== id),
        currentDocument: state.currentDocument?.id === id ? null : state.currentDocument,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  loadContent: async (id: string) => {
    try {
      set({ loading: true, error: null });
      const content = await api.fetchDocumentContent(id);
      if (get().currentDocument?.id === id) {
        set({ currentContent: content, loading: false });
      }
    } catch (error) {
      if (get().currentDocument?.id === id) {
        set({ error: (error as Error).message, loading: false });
      }
    }
  },

  saveContent: async (id: string, content: string) => {
    try {
      await api.updateDocumentContent(id, content);
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  loadFolders: async () => {
    try {
      const folders = await api.fetchFolders();
      set({ folders });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  createFolder: async (data) => {
    try {
      const folder = await api.createFolder(data);
      set((state) => ({ folders: [...state.folders, folder] }));
      return folder;
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  updateFolder: async (id, data) => {
    try {
      const folder = await api.updateFolder(id, data);
      set((state) => ({
        folders: state.folders.map((f) => (f.id === id ? folder : f)),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  deleteFolder: async (id) => {
    try {
      await api.deleteFolder(id);
      set((state) => ({
        folders: state.folders.filter((f) => f.id !== id),
        currentFolderId: state.currentFolderId === id ? null : state.currentFolderId,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  setCurrentFolderId: (id) => set({ currentFolderId: id }),
}));

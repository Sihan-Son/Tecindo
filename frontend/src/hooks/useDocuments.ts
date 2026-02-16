import { useEffect } from 'react';
import { useDocumentStore } from '@/stores/documentStore';

export function useDocuments() {
  const {
    documents,
    currentDocument,
    loading,
    error,
    loadDocuments,
    loadDocument,
    createDocument,
    updateDocument,
    deleteDocument,
  } = useDocumentStore();

  useEffect(() => {
    loadDocuments();
  }, []);

  return {
    documents,
    currentDocument,
    loading,
    error,
    loadDocuments,
    loadDocument,
    createDocument,
    updateDocument,
    deleteDocument,
  };
}

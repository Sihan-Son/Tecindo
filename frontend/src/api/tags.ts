import type { Tag } from '@/lib/types';
import { authFetch } from '@/api/client';

const API_BASE = '/api/v1';

export async function fetchTags(): Promise<Tag[]> {
  const response = await authFetch(`${API_BASE}/tags`);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to fetch tags' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  const data = await response.json();
  return data.tags;
}

export async function createTag(data: { name: string; color?: string }): Promise<Tag> {
  const response = await authFetch(`${API_BASE}/tags`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to create tag' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  return response.json();
}

export async function updateTag(id: string, data: { name?: string; color?: string }): Promise<Tag> {
  const response = await authFetch(`${API_BASE}/tags/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to update tag' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  return response.json();
}

export async function deleteTag(id: string): Promise<void> {
  const response = await authFetch(`${API_BASE}/tags/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to delete tag' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
}

export async function addTagToDocument(documentId: string, tagId: string): Promise<void> {
  const response = await authFetch(`${API_BASE}/documents/${documentId}/tags`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tag_id: tagId }),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to add tag' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
}

export async function removeTagFromDocument(documentId: string, tagId: string): Promise<void> {
  const response = await authFetch(`${API_BASE}/documents/${documentId}/tags/${tagId}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to remove tag' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
}

export async function fetchDocumentTags(documentId: string): Promise<Tag[]> {
  const response = await authFetch(`${API_BASE}/documents/${documentId}/tags`);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Failed to fetch document tags' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  const data = await response.json();
  return data.tags;
}

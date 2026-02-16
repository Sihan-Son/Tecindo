import type { Document, Folder } from '@/lib/types';

const API_BASE = '/api/v1';

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Unknown error' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  return response.json();
}

export async function authFetch(url: string, options: RequestInit = {}): Promise<Response> {
  const token = localStorage.getItem('access_token');
  const headers = new Headers(options.headers);
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }
  const response = await fetch(url, { ...options, headers });
  if (response.status === 401 && token) {
    const refreshToken = localStorage.getItem('refresh_token');
    if (refreshToken) {
      try {
        const refreshResponse = await fetch(`${API_BASE}/auth/refresh`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ refresh_token: refreshToken }),
        });
        if (refreshResponse.ok) {
          const data = await refreshResponse.json();
          localStorage.setItem('access_token', data.access_token);
          localStorage.setItem('refresh_token', data.refresh_token);
          headers.set('Authorization', `Bearer ${data.access_token}`);
          return fetch(url, { ...options, headers });
        }
      } catch {
        // refresh failed, fall through
      }
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      window.location.href = '/login';
    }
  }
  return response;
}

export async function fetchDocuments(): Promise<Document[]> {
  const response = await fetch(`${API_BASE}/documents`);
  const data = await handleResponse<{ documents: Document[] }>(response);
  return data.documents;
}

export async function fetchDocument(id: string): Promise<Document> {
  const response = await fetch(`${API_BASE}/documents/${id}`);
  return handleResponse<Document>(response);
}

export async function createDocument(data?: { title?: string; folder_id?: string }): Promise<Document> {
  const response = await fetch(`${API_BASE}/documents`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data || {}),
  });
  return handleResponse<Document>(response);
}

export async function updateDocument(
  id: string,
  data: { title?: string; folder_id?: string; is_pinned?: boolean; is_archived?: boolean }
): Promise<Document> {
  const response = await fetch(`${API_BASE}/documents/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Document>(response);
}

export async function deleteDocument(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/documents/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Unknown error' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
}

export async function fetchDocumentContent(id: string): Promise<string> {
  const response = await fetch(`${API_BASE}/documents/${id}/content`);
  const data = await handleResponse<{ content: string }>(response);
  return data.content;
}

export async function updateDocumentContent(id: string, content: string): Promise<void> {
  const response = await fetch(`${API_BASE}/documents/${id}/content`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Unknown error' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
}

export async function fetchFolders(): Promise<Folder[]> {
  const response = await fetch(`${API_BASE}/folders`);
  const data = await handleResponse<{ folders: Folder[] }>(response);
  return data.folders;
}

export async function createFolder(data: { name: string; parent_id?: string }): Promise<Folder> {
  const response = await fetch(`${API_BASE}/folders`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Folder>(response);
}

export async function updateFolder(
  id: string,
  data: { name?: string; parent_id?: string; sort_order?: number }
): Promise<Folder> {
  const response = await fetch(`${API_BASE}/folders/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Folder>(response);
}

export async function deleteFolder(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/folders/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Unknown error' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
}

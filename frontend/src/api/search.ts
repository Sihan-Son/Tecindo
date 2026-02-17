import type { SearchResult } from '@/lib/types';
import { authFetch } from '@/api/client';

const API_BASE = '/api/v1';

export async function searchDocuments(query: string): Promise<SearchResult[]> {
  const response = await authFetch(`${API_BASE}/search?q=${encodeURIComponent(query)}`);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Search failed' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  const data = await response.json();
  return data.documents;
}

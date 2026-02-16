export interface Document {
  id: string;
  folder_id: string | null;
  title: string;
  slug: string;
  file_path: string;
  word_count: number;
  char_count: number;
  excerpt: string | null;
  is_pinned: boolean;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface Folder {
  id: string;
  parent_id: string | null;
  name: string;
  slug: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string | null;
}

export interface DocumentTag {
  document_id: string;
  tag_id: string;
}

export interface SearchResult {
  id: string;
  title: string;
  excerpt: string;
  rank: number;
}

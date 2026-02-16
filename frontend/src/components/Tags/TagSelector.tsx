import { useState, useEffect } from 'react';
import { useTagStore } from '@/stores/tagStore';
import { TagBadge } from './TagBadge';

interface TagSelectorProps {
  documentId: string;
}

const styles = {
  container: {
    position: 'relative' as const,
  },
  tagList: {
    display: 'flex',
    flexWrap: 'wrap' as const,
    gap: '0.25rem',
    alignItems: 'center',
  },
  addButton: {
    background: 'none',
    border: '1px dashed #ccc',
    borderRadius: '9999px',
    padding: '0.125rem 0.5rem',
    fontSize: '0.75rem',
    color: '#888',
    cursor: 'pointer',
  },
  dropdown: {
    position: 'absolute' as const,
    top: '100%',
    left: 0,
    marginTop: '0.25rem',
    backgroundColor: '#fff',
    border: '1px solid #ddd',
    borderRadius: '6px',
    boxShadow: '0 4px 12px rgba(0,0,0,0.1)',
    zIndex: 50,
    minWidth: '180px',
    padding: '0.25rem',
  },
  dropdownItem: {
    display: 'block',
    width: '100%',
    padding: '0.375rem 0.5rem',
    border: 'none',
    background: 'none',
    textAlign: 'left' as const,
    fontSize: '0.8125rem',
    cursor: 'pointer',
    borderRadius: '4px',
  },
  empty: {
    padding: '0.5rem',
    fontSize: '0.8125rem',
    color: '#999',
  },
};

export function TagSelector({ documentId }: TagSelectorProps) {
  const [open, setOpen] = useState(false);
  const { tags, documentTags, loadTags, loadDocumentTags, addTagToDocument, removeTagFromDocument } = useTagStore();

  const currentTags = documentTags[documentId] || [];
  const availableTags = tags.filter((t) => !currentTags.some((ct) => ct.id === t.id));

  useEffect(() => {
    loadTags();
    loadDocumentTags(documentId);
  }, [documentId, loadTags, loadDocumentTags]);

  return (
    <div style={styles.container}>
      <div style={styles.tagList}>
        {currentTags.map((tag) => (
          <TagBadge
            key={tag.id}
            tag={tag}
            onRemove={() => removeTagFromDocument(documentId, tag.id)}
          />
        ))}
        <button style={styles.addButton} onClick={() => setOpen(!open)}>
          + 태그
        </button>
      </div>
      {open && (
        <div style={styles.dropdown}>
          {availableTags.length === 0 ? (
            <div style={styles.empty}>추가할 수 있는 태그가 없습니다</div>
          ) : (
            availableTags.map((tag) => (
              <button
                key={tag.id}
                style={styles.dropdownItem}
                onClick={() => {
                  addTagToDocument(documentId, tag.id);
                  setOpen(false);
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = '#f3f4f6';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = 'transparent';
                }}
              >
                <TagBadge tag={tag} />
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}

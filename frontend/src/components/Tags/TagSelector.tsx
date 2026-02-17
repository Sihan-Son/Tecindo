import { useState, useEffect, useRef } from 'react';
import { useTagStore } from '@/stores/tagStore';
import { TagBadge } from './TagBadge';

interface TagSelectorProps {
  documentId: string;
}

export function TagSelector({ documentId }: TagSelectorProps) {
  const [input, setInput] = useState('');
  const [showSuggestions, setShowSuggestions] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const addingRef = useRef(false);
  const { tags, documentTags, loadTags, loadDocumentTags, addTagToDocument, removeTagFromDocument, findOrCreateTag } = useTagStore();

  const currentTags = documentTags[documentId] || [];
  const suggestions = input.trim()
    ? tags.filter((t) =>
        t.name.toLowerCase().includes(input.trim().toLowerCase()) &&
        !currentTags.some((ct) => ct.id === t.id)
      )
    : [];

  useEffect(() => {
    loadTags();
    loadDocumentTags(documentId);
  }, [documentId, loadTags, loadDocumentTags]);

  const handleAddTag = async (name: string) => {
    const trimmed = name.trim();
    if (!trimmed || addingRef.current) return;
    if (currentTags.some((t) => t.name.toLowerCase() === trimmed.toLowerCase())) {
      setInput('');
      return;
    }
    addingRef.current = true;
    try {
      const tag = await findOrCreateTag(trimmed);
      await addTagToDocument(documentId, tag.id);
    } finally {
      addingRef.current = false;
    }
    setInput('');
    setShowSuggestions(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      handleAddTag(input);
    }
    if (e.key === 'Backspace' && !input && currentTags.length > 0) {
      const last = currentTags[currentTags.length - 1];
      removeTagFromDocument(documentId, last.id);
    }
  };

  return (
    <div className="tag-selector">
      <span className="tag-selector-hash">#</span>
      <div className="tag-selector-tags">
        {currentTags.map((tag) => (
          <TagBadge
            key={tag.id}
            tag={tag}
            onRemove={() => removeTagFromDocument(documentId, tag.id)}
          />
        ))}
        <input
          ref={inputRef}
          className="tag-selector-input"
          type="text"
          value={input}
          onChange={(e) => {
            setInput(e.target.value);
            setShowSuggestions(true);
          }}
          onKeyDown={handleKeyDown}
          onFocus={() => setShowSuggestions(true)}
          onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
          placeholder={currentTags.length === 0 ? '태그 입력 후 Enter' : ''}
        />
      </div>
      {showSuggestions && input.trim() && suggestions.length > 0 && (
        <div className="tag-selector-dropdown">
          {suggestions.map((tag) => (
            <button
              key={tag.id}
              className="tag-selector-suggestion"
              onMouseDown={(e) => {
                e.preventDefault();
                handleAddTag(tag.name);
              }}
            >
              <TagBadge tag={tag} />
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

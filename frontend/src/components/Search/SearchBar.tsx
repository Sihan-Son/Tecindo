import { useState, useEffect, useRef } from 'react';
import { searchDocuments } from '@/api/search';
import { SearchResults } from './SearchResults';
import type { SearchResult } from '@/lib/types';

const styles = {
  container: {
    position: 'relative' as const,
    width: '100%',
    maxWidth: '400px',
  },
  input: {
    width: '100%',
    padding: '0.5rem 0.75rem 0.5rem 2rem',
    border: '1px solid #ddd',
    borderRadius: '6px',
    fontSize: '0.8125rem',
    outline: 'none',
    boxSizing: 'border-box' as const,
    backgroundColor: '#f9fafb',
  },
  icon: {
    position: 'absolute' as const,
    left: '0.625rem',
    top: '50%',
    transform: 'translateY(-50%)',
    color: '#999',
    fontSize: '0.875rem',
    pointerEvents: 'none' as const,
  },
};

export function SearchBar() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [showResults, setShowResults] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!query.trim()) {
      setResults([]);
      setShowResults(false);
      return;
    }
    debounceRef.current = setTimeout(async () => {
      setLoading(true);
      try {
        const data = await searchDocuments(query);
        setResults(data);
        setShowResults(true);
      } catch {
        setResults([]);
      } finally {
        setLoading(false);
      }
    }, 300);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setShowResults(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div ref={containerRef} style={styles.container}>
      <span style={styles.icon}>&#x1F50D;</span>
      <input
        style={styles.input}
        type="text"
        placeholder="문서 검색..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onFocus={() => {
          if (results.length > 0) setShowResults(true);
        }}
      />
      {showResults && (
        <SearchResults
          results={results}
          loading={loading}
          onSelect={() => {
            setShowResults(false);
            setQuery('');
          }}
        />
      )}
    </div>
  );
}

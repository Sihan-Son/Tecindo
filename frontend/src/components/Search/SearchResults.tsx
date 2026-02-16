import { useNavigate } from 'react-router-dom';
import type { SearchResult } from '@/lib/types';

interface SearchResultsProps {
  results: SearchResult[];
  loading: boolean;
  onSelect: () => void;
}

const styles = {
  container: {
    position: 'absolute' as const,
    top: '100%',
    left: 0,
    right: 0,
    marginTop: '0.25rem',
    backgroundColor: '#fff',
    border: '1px solid #ddd',
    borderRadius: '6px',
    boxShadow: '0 4px 12px rgba(0,0,0,0.1)',
    zIndex: 50,
    maxHeight: '320px',
    overflowY: 'auto' as const,
  },
  item: {
    display: 'block',
    width: '100%',
    padding: '0.625rem 0.75rem',
    border: 'none',
    borderBottom: '1px solid #f3f4f6',
    background: 'none',
    textAlign: 'left' as const,
    cursor: 'pointer',
  },
  title: {
    fontSize: '0.875rem',
    fontWeight: 500,
    color: '#111',
    marginBottom: '0.125rem',
  },
  excerpt: {
    fontSize: '0.75rem',
    color: '#666',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap' as const,
  },
  empty: {
    padding: '1rem',
    textAlign: 'center' as const,
    fontSize: '0.8125rem',
    color: '#999',
  },
  loading: {
    padding: '1rem',
    textAlign: 'center' as const,
    fontSize: '0.8125rem',
    color: '#999',
  },
};

export function SearchResults({ results, loading, onSelect }: SearchResultsProps) {
  const navigate = useNavigate();

  if (loading) {
    return (
      <div style={styles.container}>
        <div style={styles.loading}>검색 중...</div>
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <div style={styles.container}>
        <div style={styles.empty}>검색 결과가 없습니다</div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      {results.map((result) => (
        <button
          key={result.id}
          style={styles.item}
          onClick={() => {
            navigate(`/doc/${result.id}`);
            onSelect();
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#f9fafb';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
          }}
        >
          <div style={styles.title}>{result.title}</div>
          <div style={styles.excerpt}>{result.excerpt}</div>
        </button>
      ))}
    </div>
  );
}

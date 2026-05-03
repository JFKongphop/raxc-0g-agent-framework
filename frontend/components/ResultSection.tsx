'use client';

import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useState, useEffect } from 'react';

interface ResultSectionProps {
  result: any;
}

export function ResultSection({ result }: ResultSectionProps) {
  const [reportContent, setReportContent] = useState<string | null>(null);

  useEffect(() => {
    if (result?.download_url) {
      fetch(`${process.env.NEXT_PUBLIC_API_URL}${result.download_url}`)
        .then(res => res.text())
        .then(setReportContent)
        .catch(console.error);
    }
  }, [result?.download_url]);

  if (result.error) {
    return (
      <div className="card" style={{ background: 'rgba(255,69,58,0.1)', borderColor: 'var(--red)' }}>
        <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--red)', marginBottom: 8 }}>
          ❌ Error
        </div>
        <div style={{ fontSize: 14, color: 'var(--text-muted)' }}>{result.error}</div>
      </div>
    );
  }

  const getRiskBadge = (risk: string) => {
    const lower = risk.toLowerCase();
    if (lower.includes('critical')) return 'badge-critical';
    if (lower.includes('high')) return 'badge-high';
    if (lower.includes('medium')) return 'badge-medium';
    if (lower.includes('low')) return 'badge-low';
    return 'badge-none';
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      {/* Summary Card */}
      <div className="card">
        <div className="card-header">📊 Analysis Results</div>

        <div className="result-grid">
          <div className="stat-card">
            <div className="stat-label">Vulnerability</div>
            <div className="stat-value">{result.vulnerability_found || 'N/A'}</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Risk Level</div>
            <div className="stat-value">
              <span className={`badge ${getRiskBadge(result.risk_level || '')}`}>
                {result.risk_level || 'None'}
              </span>
            </div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Confidence</div>
            <div className="stat-value">{result.confidence || '?'}%</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Type</div>
            <div className="stat-value" style={{ fontSize: 14 }}>
              {result.vulnerability_type || 'N/A'}
            </div>
          </div>
        </div>

        {result.download_url && (
          <a
            href={`${process.env.NEXT_PUBLIC_API_URL}${result.download_url}`}
            download
            className="btn btn-secondary"
            style={{ width: '100%', justifyContent: 'center', marginTop: 12 }}
          >
            📄 Download Full Report
          </a>
        )}
      </div>

      {/* Full Report Card */}
      {reportContent && (
        <div className="card">
          <div className="card-header">📋 Full Security Report</div>
          <div className="markdown-body">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{reportContent}</ReactMarkdown>
          </div>
        </div>
      )}
    </div>
  );
}

import { useState, useEffect } from 'react';
import { getClipboardItemDetail, copyToClipboard } from '@/lib/clipboard-api';
import type { ClipboardItemDetail } from '@/lib/clipboard-api';

interface ClipboardDetailProps {
  itemId: number | null;
  onClose: () => void;
}

export function ClipboardDetail({ itemId, onClose }: ClipboardDetailProps) {
  const [detail, setDetail] = useState<ClipboardItemDetail | null>(null);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!itemId) {
      setDetail(null);
      return;
    }

    setLoading(true);
    getClipboardItemDetail(itemId)
      .then(setDetail)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [itemId]);

  const handleCopy = async () => {
    if (!detail) return;
    if (detail.content_text) {
      await copyToClipboard(detail.content_text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (!itemId) return null;

  const isText = detail?.content_type === 'text';
  const imageBase64 = detail?.content_image
    ? `data:${detail.mime_type};base64,${btoa(
        new Uint8Array(detail.content_image).reduce((data, byte) => data + String.fromCharCode(byte), '')
      )}`
    : null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onClose}>
      <div
        className="bg-card rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-border">
          <h3 className="font-semibold text-foreground">详情</h3>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="flex-1 overflow-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
          ) : detail ? (
            <div className="space-y-4">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <span>{new Date(detail.created_at).toLocaleString('zh-CN')}</span>
                <span>•</span>
                <span>{isText ? '文本' : '图片'}</span>
                {detail.is_pinned && <span>• 📌 已置顶</span>}
              </div>

              {isText ? (
                <pre className="p-4 bg-muted rounded-lg text-sm text-foreground font-mono whitespace-pre-wrap break-all overflow-auto max-h-96">
                  {detail.content_text || '(空)'}
                </pre>
              ) : (
                <div className="flex flex-col items-center gap-4">
                  {imageBase64 && (
                    <img
                      src={imageBase64}
                      alt="Clipboard image"
                      className="max-w-full max-h-96 rounded-lg object-contain"
                    />
                  )}
                  <span className="text-sm text-muted-foreground">
                    {detail.image_width}×{detail.image_height}
                  </span>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center text-muted-foreground py-8">加载失败</div>
          )}
        </div>

        <div className="flex items-center justify-end gap-2 p-4 border-t border-border">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-muted-foreground hover:bg-accent rounded-lg transition-colors"
          >
            关闭
          </button>
          {isText && detail?.content_text && (
            <button
              onClick={handleCopy}
              className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
            >
              {copied ? '已复制!' : '复制内容'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

import type { ClipboardItemPreview } from '@/lib/clipboard-api';

interface ClipboardItemProps {
  item: ClipboardItemPreview;
  isSelected: boolean;
  onSelect: (id: number) => void;
  onCopy: (id: number) => void;
  onPin: (id: number) => void;
  onDelete: (id: number) => void;
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  if (days < 7) return `${days} 天前`;
  
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function truncateText(text: string, maxLength: number = 100): string {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + '...';
}

export function ClipboardItemComponent({
  item,
  isSelected,
  onSelect,
  onCopy,
  onPin,
  onDelete,
}: ClipboardItemProps) {
  const isText = item.content_type === 'text';

  return (
    <div
      onClick={() => onSelect(item.id)}
      className={`group relative p-3 rounded-lg cursor-pointer transition-all ${
        isSelected
          ? 'bg-accent ring-2 ring-ring'
          : 'hover:bg-accent/50'
      }`}
    >
      <div className="flex items-start gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs text-muted-foreground">
              {formatTime(item.created_at)}
            </span>
            {item.is_pinned && (
              <span className="text-xs text-yellow-500">📌</span>
            )}
            <span className={`text-xs px-1.5 py-0.5 rounded ${
              isText ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' 
                     : 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
            }`}>
              {isText ? '文本' : '图片'}
            </span>
          </div>
          
          {isText ? (
            <p className="text-sm text-foreground break-all line-clamp-2">
              {item.content_text ? truncateText(item.content_text) : '(空)'}
            </p>
          ) : (
            <div className="flex items-center gap-2">
              {item.has_image && (
                <div className="w-16 h-16 rounded bg-muted flex items-center justify-center">
                  <span className="text-2xl">🖼️</span>
                </div>
              )}
              <span className="text-xs text-muted-foreground">
                {item.image_width}×{item.image_height}
              </span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={(e) => { e.stopPropagation(); onCopy(item.id); }}
            className="p-1.5 rounded hover:bg-background text-muted-foreground hover:text-foreground"
            title="复制"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); onPin(item.id); }}
            className="p-1.5 rounded hover:bg-background text-muted-foreground hover:text-foreground"
            title={item.is_pinned ? '取消置顶' : '置顶'}
          >
            <span className="text-sm">{item.is_pinned ? '📌' : '📍'}</span>
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); onDelete(item.id); }}
            className="p-1.5 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive"
            title="删除"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}

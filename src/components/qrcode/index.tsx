import { useCallback, useEffect, useRef, useState } from 'react';
import { copyToClipboard } from '@/lib/clipboard-api';
import {
  clearQrDecodeItems,
  clearQrGenerateItems,
  copyQrGenerateToClipboard,
  decodeQrAndSave,
  deleteQrDecodeItem,
  deleteQrGenerateItem,
  formatRelativeTime,
  generateQrAndSave,
  getQrDecodeItem,
  getQrGenerateItem,
  listQrDecodeItems,
  listQrGenerateItems,
  pngBytesToDataUrl,
  saveQrGenerateFile,
  type QrDecodeItem,
  type QrGenerateDetail,
  type QrGeneratePreview,
} from '@/lib/qrcode-api';
import { notify } from '@/lib/toast';
import { toolTabBarClass, toolTabButtonClass, toolContentClass } from '@/lib/tab-styles';

type TabType = 'decode' | 'generate';

const tabs: { id: TabType; name: string; icon: string }[] = [
  { id: 'decode', name: '解析', icon: '🔍' },
  { id: 'generate', name: '生成', icon: '✨' },
];

const ACCEPT_IMAGE = 'image/png,image/jpeg,image/jpg,image/gif,image/webp,image/bmp';

const timeFilters = [
  { value: 'all', label: '全部' },
  { value: 'today', label: '今天' },
  { value: 'yesterday', label: '昨天' },
  { value: 'week', label: '本周' },
  { value: 'month', label: '本月' },
] as const;

function truncate(text: string, max = 80): string {
  if (text.length <= max) return text;
  return `${text.slice(0, max)}…`;
}

export function QrcodeTool() {
  const [tab, setTab] = useState<TabType>('decode');

  return (
    <div className="h-full flex flex-col">
      <div className={toolTabBarClass}>
        <div className="flex gap-2">
          {tabs.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => setTab(t.id)}
              className={toolTabButtonClass(tab === t.id)}
            >
              <span className="mr-1.5">{t.icon}</span>
              {t.name}
            </button>
          ))}
        </div>
      </div>

      <div className={`${toolContentClass} overflow-hidden p-6`}>
        <div className={tab === 'decode' ? 'h-full overflow-y-auto' : 'hidden'}>
          <DecodePanel />
        </div>
        <div className={tab === 'generate' ? 'h-full overflow-y-auto' : 'hidden'}>
          <GeneratePanel />
        </div>
      </div>
    </div>
  );
}

function HistorySection({
  title,
  total,
  onClear,
  show,
}: {
  title: string;
  total: number;
  onClear: () => void;
  show?: boolean;
}) {
  if (!show && total === 0) return null;

  return (
    <div className="flex items-center justify-between pt-2 border-t border-border">
      <h3 className="text-sm font-medium text-foreground">
        {title}
        <span className="ml-2 text-xs font-normal text-muted-foreground">({total})</span>
      </h3>
      <button
        type="button"
        onClick={onClear}
        className="text-xs text-muted-foreground hover:text-destructive transition-colors"
      >
        清空
      </button>
    </div>
  );
}

function HistoryFilterBar({
  timeFilter,
  searchQuery,
  total,
  searchPlaceholder,
  onTimeFilterChange,
  onSearchChange,
}: {
  timeFilter: string;
  searchQuery: string;
  total: number;
  searchPlaceholder: string;
  onTimeFilterChange: (value: string) => void;
  onSearchChange: (value: string) => void;
}) {
  return (
    <div className="space-y-3 pt-2 border-t border-border">
      <div className="relative">
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={searchPlaceholder}
          className="w-full pl-9 pr-3 py-2 text-sm border border-input rounded-lg bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
        />
        <svg
          className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
      </div>

      <div className="flex items-center justify-between gap-2 flex-wrap">
        <div className="flex items-center gap-1 flex-wrap">
          {timeFilters.map((filter) => (
            <button
              key={filter.value}
              type="button"
              onClick={() => onTimeFilterChange(filter.value)}
              className={`px-3 py-1 text-xs rounded-full transition-colors ${
                timeFilter === filter.value
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-primary'
              }`}
            >
              {filter.label}
            </button>
          ))}
        </div>
        <span className="text-xs text-muted-foreground shrink-0">共 {total} 条</span>
      </div>
    </div>
  );
}

function DecodePanel() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [fileName, setFileName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<QrDecodeItem[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [timeFilter, setTimeFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<QrDecodeItem | null>(null);
  const [copiedId, setCopiedId] = useState<number | null>(null);
  const historyInitialized = useRef(false);

  const refreshHistory = useCallback(async () => {
    const result = await listQrDecodeItems({
      time_filter: timeFilter,
      search_query: searchQuery,
    });
    setHistory(result.items);
    setTotalCount(result.total);
    if (!historyInitialized.current && result.items.length > 0) {
      historyInitialized.current = true;
      setSelectedId(result.items[0]!.id);
    }
  }, [timeFilter, searchQuery]);

  useEffect(() => {
    refreshHistory().catch(console.error);
  }, [refreshHistory]);

  useEffect(() => {
    if (selectedId === null) {
      setSelectedDetail(null);
      return;
    }
    getQrDecodeItem(selectedId)
      .then((item) => setSelectedDetail(item))
      .catch(console.error);
  }, [selectedId]);

  useEffect(() => {
    return () => {
      if (previewUrl) URL.revokeObjectURL(previewUrl);
    };
  }, [previewUrl]);

  const decodeFile = useCallback(
    async (file: File) => {
      if (!file.type.startsWith('image/')) {
        setError('请选择图片文件');
        return;
      }

      setLoading(true);
      setError('');
      setFileName(file.name);

      if (previewUrl) URL.revokeObjectURL(previewUrl);
      setPreviewUrl(URL.createObjectURL(file));

      try {
        const buffer = await file.arrayBuffer();
        const record = await decodeQrAndSave(new Uint8Array(buffer), file.name);
        setSelectedDetail(record);
        setSelectedId(record.id);
        historyInitialized.current = true;
        await refreshHistory();
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    },
    [previewUrl, refreshHistory],
  );

  const onFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) void decodeFile(file);
      e.target.value = '';
    },
    [decodeFile],
  );

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const file = e.dataTransfer.files?.[0];
      if (file) void decodeFile(file);
    },
    [decodeFile],
  );

  const handleCopyText = useCallback(async (text: string, id: number) => {
    try {
      await copyToClipboard(text);
      setCopiedId(id);
      notify('已复制');
      setTimeout(() => setCopiedId(null), 1500);
    } catch (e) {
      notify(String(e));
    }
  }, []);

  const handleDelete = useCallback(
    async (id: number) => {
      await deleteQrDecodeItem(id);
      if (selectedId === id) {
        setSelectedId(null);
        setSelectedDetail(null);
        if (previewUrl) URL.revokeObjectURL(previewUrl);
        setPreviewUrl(null);
        setFileName('');
      }
      await refreshHistory();
    },
    [refreshHistory, selectedId, previewUrl],
  );

  const handleClear = useCallback(async () => {
    if (totalCount === 0 && history.length === 0 && !selectedDetail) return;
    if (confirm('确定清空全部解析历史吗？')) {
      await clearQrDecodeItems();
      setSelectedId(null);
      setSelectedDetail(null);
      if (previewUrl) URL.revokeObjectURL(previewUrl);
      setPreviewUrl(null);
      setFileName('');
      historyInitialized.current = false;
      await refreshHistory();
    }
  }, [totalCount, history.length, selectedDetail, previewUrl, refreshHistory]);

  const hasHistoryContext = totalCount > 0 || timeFilter !== 'all' || searchQuery || selectedId !== null;

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <p className="text-sm text-muted-foreground">
        上传或拖入含二维码的图片；历史保存在本地 SQLite（qrcode.db）。
      </p>

      <input ref={inputRef} type="file" accept={ACCEPT_IMAGE} className="hidden" onChange={onFileChange} />

      <div
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === 'Enter' && inputRef.current?.click()}
        onClick={() => inputRef.current?.click()}
        onDragOver={(e) => e.preventDefault()}
        onDrop={onDrop}
        className="flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed border-border bg-muted/30 px-6 py-8 cursor-pointer hover:border-primary/40 hover:bg-muted/50 transition-colors"
      >
        <span className="text-3xl">🖼️</span>
        <p className="text-sm font-medium text-foreground">点击选择图片，或拖拽到此处</p>
      </div>

      {fileName && loading && (
        <p className="text-xs text-muted-foreground truncate">正在识别：{fileName}</p>
      )}

      {previewUrl && (
        <div className="flex justify-center p-3 rounded-lg border border-border bg-card">
          <img src={previewUrl} alt="上传预览" className="max-h-40 max-w-full object-contain rounded" />
        </div>
      )}

      {loading && (
        <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground py-2">
          <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-primary" />
          正在识别…
        </div>
      )}

      {error && <div className="p-3 rounded-lg bg-destructive/10 text-destructive text-sm">{error}</div>}

      {selectedDetail && !loading && (
        <div className="space-y-2 p-4 rounded-lg border border-border bg-card">
          <div className="flex items-center justify-between gap-2">
            <div className="min-w-0">
              <p className="text-xs text-muted-foreground truncate">{selectedDetail.file_name}</p>
              <p className="text-xs text-muted-foreground">{formatRelativeTime(selectedDetail.created_at)}</p>
            </div>
            <button
              type="button"
              onClick={() => handleCopyText(selectedDetail.text, selectedDetail.id)}
              className="shrink-0 px-3 py-1.5 text-xs rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
            >
              {copiedId === selectedDetail.id ? '已复制' : '复制'}
            </button>
          </div>
          <pre className="p-3 rounded-lg bg-muted text-sm font-mono whitespace-pre-wrap break-all overflow-auto max-h-36">
            {selectedDetail.text}
          </pre>
        </div>
      )}

      {hasHistoryContext && (
        <HistoryFilterBar
          timeFilter={timeFilter}
          searchQuery={searchQuery}
          total={totalCount}
          searchPlaceholder="搜索解析内容或文件名..."
          onTimeFilterChange={setTimeFilter}
          onSearchChange={setSearchQuery}
        />
      )}

      <HistorySection
        title="解析历史"
        total={totalCount}
        onClear={handleClear}
        show={hasHistoryContext}
      />

      {history.length > 0 ? (
        <ul className="space-y-2">
          {history.map((item) => (
            <li
              key={item.id}
              onClick={() => setSelectedId(item.id)}
              className={`group flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                selectedId === item.id
                  ? 'border-primary bg-primary/5'
                  : 'border-border bg-card hover:bg-accent/40'
              }`}
            >
              <div className="flex-1 min-w-0">
                <p className="text-sm text-foreground font-mono break-all line-clamp-2">{truncate(item.text, 120)}</p>
                <p className="text-xs text-muted-foreground mt-1 truncate">
                  {item.file_name} · {formatRelativeTime(item.created_at)}
                </p>
              </div>
              <div className="flex shrink-0 gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <IconButton title="复制" onClick={() => handleCopyText(item.text, item.id)}>
                  {copiedId === item.id ? '✓' : '📋'}
                </IconButton>
                <IconButton title="删除" onClick={() => handleDelete(item.id)} danger>
                  🗑
                </IconButton>
              </div>
            </li>
          ))}
        </ul>
      ) : timeFilter !== 'all' || searchQuery ? (
        <p className="text-sm text-muted-foreground py-4 text-center">没有符合筛选条件的历史记录</p>
      ) : null}
    </div>
  );
}

function GeneratePanel() {
  const [text, setText] = useState('https://');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<QrGeneratePreview[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [timeFilter, setTimeFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<QrGenerateDetail | null>(null);
  const [thumbUrls, setThumbUrls] = useState<Record<number, string>>({});
  const [copiedImageId, setCopiedImageId] = useState<number | null>(null);
  const [savingId, setSavingId] = useState<number | null>(null);
  const historyInitialized = useRef(false);

  const refreshHistory = useCallback(async () => {
    const result = await listQrGenerateItems({
      time_filter: timeFilter,
      search_query: searchQuery,
    });
    setHistory(result.items);
    setTotalCount(result.total);
    if (!historyInitialized.current && result.items.length > 0) {
      historyInitialized.current = true;
      setSelectedId(result.items[0]!.id);
    }
  }, [timeFilter, searchQuery]);

  useEffect(() => {
    refreshHistory().catch(console.error);
  }, [refreshHistory]);

  const loadDetail = useCallback(async (id: number) => {
    const detail = await getQrGenerateItem(id);
    if (!detail) return null;
    setThumbUrls((prev) => ({
      ...prev,
      [id]: pngBytesToDataUrl(detail.png_data),
    }));
    return detail;
  }, []);

  useEffect(() => {
    if (selectedId === null) {
      setSelectedDetail(null);
      return;
    }
    loadDetail(selectedId)
      .then((d) => setSelectedDetail(d))
      .catch(console.error);
  }, [selectedId, loadDetail]);

  useEffect(() => {
    history.forEach((item) => {
      if (!thumbUrls[item.id]) {
        loadDetail(item.id).catch(console.error);
      }
    });
  }, [history, thumbUrls, loadDetail]);

  const handleGenerate = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const detail = await generateQrAndSave(text);
      setThumbUrls((prev) => ({
        ...prev,
        [detail.id]: pngBytesToDataUrl(detail.png_data),
      }));
      setSelectedDetail(detail);
      setSelectedId(detail.id);
      historyInitialized.current = true;
      await refreshHistory();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [text, refreshHistory]);

  const handleCopyImage = useCallback(async (id: number) => {
    try {
      await copyQrGenerateToClipboard(id);
      setCopiedImageId(id);
      notify('已复制到剪贴板');
      setTimeout(() => setCopiedImageId(null), 1500);
    } catch (e) {
      notify(String(e));
    }
  }, []);

  const handleCopyText = useCallback(async (value: string) => {
    try {
      await copyToClipboard(value);
      notify('已复制文本');
    } catch (e) {
      notify(String(e));
    }
  }, []);

  const handleSaveImage = useCallback(async (id: number) => {
    setSavingId(id);
    try {
      const saved = await saveQrGenerateFile(id, 'qrcode.png');
      if (saved) notify('已保存');
    } catch (e) {
      notify(String(e));
    } finally {
      setSavingId(null);
    }
  }, []);

  const handleDelete = useCallback(
    async (id: number) => {
      await deleteQrGenerateItem(id);
      setThumbUrls((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      if (selectedId === id) {
        setSelectedId(null);
        setSelectedDetail(null);
      }
      await refreshHistory();
    },
    [refreshHistory, selectedId],
  );

  const handleClear = useCallback(async () => {
    if (totalCount === 0 && history.length === 0 && !selectedDetail) return;
    if (confirm('确定清空全部生成历史吗？')) {
      await clearQrGenerateItems();
      setThumbUrls({});
      setSelectedDetail(null);
      setSelectedId(null);
      historyInitialized.current = false;
      await refreshHistory();
    }
  }, [totalCount, history.length, selectedDetail, refreshHistory]);

  const previewUrl = selectedId !== null ? thumbUrls[selectedId] : null;
  const hasHistoryContext = totalCount > 0 || timeFilter !== 'all' || searchQuery || selectedId !== null;

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <p className="text-sm text-muted-foreground">
        输入文本生成二维码；PNG 存入本地 SQLite，最多保留 30 条。
      </p>

      <div>
        <label className="text-sm font-medium mb-1 block">内容</label>
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="例如：https://example.com 或一段文本"
          rows={3}
          className="w-full px-3 py-2 text-sm border border-border rounded-lg bg-background focus:outline-none focus:ring-2 focus:ring-primary resize-none font-mono"
        />
      </div>

      <button
        type="button"
        onClick={handleGenerate}
        disabled={loading || !text.trim()}
        className="px-4 py-2 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
      >
        {loading ? '生成中…' : '生成二维码'}
      </button>

      {error && <div className="p-3 rounded-lg bg-destructive/10 text-destructive text-sm">{error}</div>}

      {selectedDetail && previewUrl && (
        <div className="space-y-3 p-4 rounded-lg border border-border bg-card">
          <div className="flex justify-center p-4 rounded-xl bg-white">
            <img src={previewUrl} alt="二维码预览" className="w-52 h-52 object-contain" />
          </div>
          <p className="text-sm font-mono text-foreground break-all line-clamp-3">{selectedDetail.text}</p>
          <p className="text-xs text-muted-foreground">{formatRelativeTime(selectedDetail.created_at)}</p>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => handleCopyImage(selectedDetail.id)}
              className="px-3 py-1.5 text-xs rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
            >
              {copiedImageId === selectedDetail.id ? '已复制' : '复制图片'}
            </button>
            <button
              type="button"
              onClick={() => handleSaveImage(selectedDetail.id)}
              disabled={savingId === selectedDetail.id}
              className="px-3 py-1.5 text-xs rounded-lg border border-border hover:bg-accent disabled:opacity-50"
            >
              {savingId === selectedDetail.id ? '保存中…' : '保存图片'}
            </button>
            <button
              type="button"
              onClick={() => handleCopyText(selectedDetail.text)}
              className="px-3 py-1.5 text-xs rounded-lg border border-border hover:bg-accent"
            >
              复制文本
            </button>
            <button
              type="button"
              onClick={() => setText(selectedDetail.text)}
              className="px-3 py-1.5 text-xs rounded-lg border border-border hover:bg-accent"
            >
              填入输入框
            </button>
          </div>
        </div>
      )}

      {hasHistoryContext && (
        <HistoryFilterBar
          timeFilter={timeFilter}
          searchQuery={searchQuery}
          total={totalCount}
          searchPlaceholder="搜索生成内容..."
          onTimeFilterChange={setTimeFilter}
          onSearchChange={setSearchQuery}
        />
      )}

      <HistorySection
        title="生成历史"
        total={totalCount}
        onClear={handleClear}
        show={hasHistoryContext}
      />

      {history.length > 0 ? (
        <ul className="space-y-2 pb-4">
          {history.map((item) => (
            <li
              key={item.id}
              onClick={() => setSelectedId(item.id)}
              className={`group flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                selectedId === item.id
                  ? 'border-primary bg-primary/5'
                  : 'border-border bg-card hover:bg-accent/40'
              }`}
            >
              {thumbUrls[item.id] ? (
                <img
                  src={thumbUrls[item.id]}
                  alt=""
                  className="w-12 h-12 shrink-0 rounded bg-white object-contain p-0.5"
                />
              ) : (
                <div className="w-12 h-12 shrink-0 rounded bg-muted flex items-center justify-center text-lg">
                  🔳
                </div>
              )}
              <div className="flex-1 min-w-0">
                <p className="text-sm font-mono text-foreground truncate">{truncate(item.text, 60)}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{formatRelativeTime(item.created_at)}</p>
              </div>
              <div
                className="flex shrink-0 gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
                onClick={(e) => e.stopPropagation()}
              >
                <IconButton title="复制图片" onClick={() => handleCopyImage(item.id)}>
                  {copiedImageId === item.id ? '✓' : '🖼'}
                </IconButton>
                <IconButton title="保存" onClick={() => handleSaveImage(item.id)}>
                  {savingId === item.id ? '…' : '💾'}
                </IconButton>
                <IconButton title="删除" onClick={() => handleDelete(item.id)} danger>
                  🗑
                </IconButton>
              </div>
            </li>
          ))}
        </ul>
      ) : timeFilter !== 'all' || searchQuery ? (
        <p className="text-sm text-muted-foreground py-4 text-center">没有符合筛选条件的历史记录</p>
      ) : null}
    </div>
  );
}

function IconButton({
  title,
  onClick,
  children,
  danger,
}: {
  title: string;
  onClick: () => void | Promise<void>;
  children: React.ReactNode;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      title={title}
      onClick={(e) => {
        e.stopPropagation();
        void onClick();
      }}
      className={`w-8 h-8 flex items-center justify-center rounded-md text-sm transition-colors ${
        danger ? 'hover:bg-destructive/10 hover:text-destructive' : 'hover:bg-accent'
      }`}
    >
      {children}
    </button>
  );
}

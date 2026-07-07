import { useMemo, useState, useCallback } from 'react'
import { REGIONS, searchRegions, type FlatRegion } from '@/lib/region-codes'
import { getPlateInfo, lookupByPlatePrefix } from '@/lib/plate-codes'
import { getPostalCode, lookupByPostalCode, postalLevelLabel } from '@/lib/postal-codes'
import { getSolarTermStatus, getYearSolarTerms } from '@/lib/solar-terms'
import { Calendar } from '@/components/calendar'
import { formatDateTimeShanghai } from '@/lib/date-time'
import { toolTabBarClass, toolTabButtonClass, toolContentClass } from '@/lib/tab-styles'

type TabType = 'region' | 'solar-term' | 'calendar'

const tabs: { id: TabType; name: string; icon: string }[] = [
  { id: 'region', name: '行政区划', icon: '🚗' },
  { id: 'solar-term', name: '二十四节气', icon: '🌿' },
  { id: 'calendar', name: '日历', icon: '📅' },
]

const selectClass =
  'w-full px-3 py-2 text-sm border border-border rounded-lg bg-background focus:outline-none focus:ring-2 focus:ring-primary'

const CITY_NAME_MAP: Record<string, string> = {}
for (const prov of REGIONS) {
  for (const city of prov.children ?? []) {
    CITY_NAME_MAP[city.code] = city.name
  }
}

export function LifeKnowledgeTool() {
  const [tab, setTab] = useState<TabType>('region')

  return (
    <div className="h-full flex flex-col">
      <div className={toolTabBarClass}>
        <div className="flex gap-2">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={toolTabButtonClass(tab === t.id)}
            >
              <span className="mr-1.5">{t.icon}</span>
              {t.name}
            </button>
          ))}
        </div>
      </div>

      <div
        className={`${toolContentClass} min-h-0 ${
          tab === 'calendar' ? 'overflow-hidden' : 'overflow-y-auto p-6'
        }`}
      >
        {tab === 'region' && <RegionPlateTool />}
        {tab === 'solar-term' && <SolarTermTool />}
        {tab === 'calendar' && <Calendar />}
      </div>
    </div>
  )
}

function RegionPlateTool() {
  const [plateQuery, setPlateQuery] = useState('')
  const [postalQuery, setPostalQuery] = useState('')
  const [provinceCode, setProvinceCode] = useState('')
  const [cityCode, setCityCode] = useState('')
  const [districtCode, setDistrictCode] = useState('')
  const [keyword, setKeyword] = useState('')
  const [picked, setPicked] = useState<FlatRegion | null>(null)

  const province = useMemo(
    () => REGIONS.find((r) => r.code === provinceCode) ?? null,
    [provinceCode],
  )
  const city = useMemo(
    () => province?.children?.find((r) => r.code === cityCode) ?? null,
    [province, cityCode],
  )
  const district = useMemo(
    () => city?.children?.find((r) => r.code === districtCode) ?? null,
    [city, districtCode],
  )

  const plateLookupResults = useMemo(
    () => (plateQuery.trim() ? lookupByPlatePrefix(plateQuery, CITY_NAME_MAP) : []),
    [plateQuery],
  )

  const postalLookupResults = useMemo(
    () => (postalQuery.trim() ? lookupByPostalCode(postalQuery) : []),
    [postalQuery],
  )

  const activeCode = districtCode || cityCode || provinceCode
  const plateInfo = useMemo(() => (activeCode ? getPlateInfo(activeCode) : null), [activeCode])
  const postalInfo = useMemo(() => (activeCode ? getPostalCode(activeCode) : null), [activeCode])

  const searchResults = useMemo(() => searchRegions(keyword), [keyword])

  const selectRegionByCode = useCallback((code: string) => {
    const province = `${code.slice(0, 2)}0000`
    const city = `${code.slice(0, 4)}00`
    setProvinceCode(province)
    setPicked(null)
    if (code.endsWith('0000')) {
      setCityCode('')
      setDistrictCode('')
      const provNode = REGIONS.find((r) => r.code === province)
      if (provNode && ['11', '12', '31', '50'].includes(code.slice(0, 2)) && provNode.children?.[0]) {
        setCityCode(provNode.children[0].code)
      }
    } else if (code.endsWith('00')) {
      setCityCode(city)
      setDistrictCode('')
    } else {
      setCityCode(city)
      setDistrictCode(code)
    }
  }, [])

  const handleProvinceChange = (code: string) => {
    setProvinceCode(code)
    setCityCode('')
    setDistrictCode('')
    setPicked(null)
  }

  const handleCityChange = (code: string) => {
    setCityCode(code)
    setDistrictCode('')
    setPicked(null)
  }

  const handlePickSearch = (item: FlatRegion) => {
    setPicked(item)
    setKeyword(item.name)
    selectRegionByCode(item.code)
  }

  const pathParts = [
    province?.name,
    city && city.name !== province?.name ? city.name : null,
    district?.name,
  ].filter(Boolean)

  return (
    <div className="max-w-5xl mx-auto space-y-5">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div>
          <label className="text-sm font-medium mb-1 block">车牌代号</label>
          <input
            type="text"
            value={plateQuery}
            onChange={(e) => setPlateQuery(e.target.value.toUpperCase())}
            placeholder="冀H、粤B"
            className={selectClass}
          />
        </div>
        <div>
          <label className="text-sm font-medium mb-1 block">邮政编码</label>
          <input
            type="text"
            value={postalQuery}
            onChange={(e) => setPostalQuery(e.target.value.replace(/\D/g, '').slice(0, 6))}
            placeholder="067000"
            className={selectClass}
            inputMode="numeric"
          />
        </div>
        <div>
          <label className="text-sm font-medium mb-1 block">地名搜索</label>
          <input
            type="text"
            value={keyword}
            onChange={(e) => {
              setKeyword(e.target.value)
              setPicked(null)
            }}
            placeholder="省 / 市 / 区县"
            className={selectClass}
          />
        </div>
      </div>

      {plateQuery.trim() && (
        <div>
          {plateLookupResults.length === 0 ? (
            <p className="text-sm text-muted-foreground">车牌代号未找到匹配地区</p>
          ) : (
            <div className="space-y-2">
              {plateLookupResults.map((item) => {
                const regionCode = item.cityCode ?? item.provinceCode
                return (
                <button
                  key={`${item.platePrefix}-${item.cityCode ?? item.provinceCode}`}
                  type="button"
                  onClick={() => selectRegionByCode(regionCode)}
                  className="w-full text-left border border-border rounded-lg p-3 text-sm hover:bg-secondary/60 transition-colors"
                >
                  <div className="font-mono text-base font-semibold text-primary mb-1">
                    {item.platePrefix}
                  </div>
                  <div className="text-foreground">{item.regionPath}</div>
                  <div className="text-xs text-muted-foreground mt-1 flex flex-wrap gap-x-3 gap-y-0.5">
                    <span>{item.provinceName}</span>
                    {item.cityCode && <span>区划代码 {item.cityCode}</span>}
                  </div>
                </button>
                )
              })}
            </div>
          )}
        </div>
      )}

      {postalQuery.trim() && (
        <div>
          {postalLookupResults.length === 0 ? (
            <p className="text-sm text-muted-foreground">邮政编码未找到匹配地区</p>
          ) : (
            <div className="space-y-2">
              {postalLookupResults.map((item) => (
                <button
                  key={`${item.postal}-${item.regionCode}`}
                  type="button"
                  onClick={() => selectRegionByCode(item.regionCode)}
                  className="w-full text-left border border-border rounded-lg p-3 text-sm hover:bg-secondary/60 transition-colors"
                >
                  <div className="font-mono text-base font-semibold text-primary mb-1">
                    {item.postal}
                  </div>
                  <div className="text-foreground">{item.regionPath}</div>
                  <div className="text-xs text-muted-foreground mt-1 flex flex-wrap gap-x-3 gap-y-0.5">
                    <span>{postalLevelLabel(item.level)}</span>
                    <span>区划代码 {item.regionCode}</span>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {keyword && !picked && searchResults.length > 0 && (
        <div className="border border-border rounded-lg overflow-hidden max-h-48 overflow-y-auto">
          {searchResults.map((item) => (
            <button
              key={item.code}
              type="button"
              onClick={() => handlePickSearch(item)}
              className="w-full px-3 py-2 text-left text-sm hover:bg-secondary/60 border-b border-border last:border-b-0"
            >
              <div>{item.name}</div>
              <div className="text-xs text-muted-foreground">{item.path}</div>
            </button>
          ))}
        </div>
      )}

      <div className="relative">
        <div className="absolute inset-0 flex items-center" aria-hidden>
          <div className="w-full border-t border-border" />
        </div>
        <div className="relative flex justify-center">
          <span className="bg-background px-3 text-xs text-muted-foreground">或按行政区划查询</span>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div>
          <label className="text-sm font-medium mb-1 block">省 / 直辖市</label>
          <select
            value={provinceCode}
            onChange={(e) => handleProvinceChange(e.target.value)}
            className={selectClass}
          >
            <option value="">请选择</option>
            {REGIONS.map((r) => (
              <option key={r.code} value={r.code}>{r.name}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="text-sm font-medium mb-1 block">地级市</label>
          <select
            value={cityCode}
            onChange={(e) => handleCityChange(e.target.value)}
            disabled={!province?.children?.length}
            className={selectClass}
          >
            <option value="">请选择</option>
            {(province?.children ?? []).map((r) => (
              <option key={r.code} value={r.code}>{r.name}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="text-sm font-medium mb-1 block">区 / 县</label>
          <select
            value={districtCode}
            onChange={(e) => {
              setDistrictCode(e.target.value)
              setPicked(null)
            }}
            disabled={!city?.children?.length}
            className={selectClass}
          >
            <option value="">请选择</option>
            {(city?.children ?? []).map((r) => (
              <option key={r.code} value={r.code}>{r.name}</option>
            ))}
          </select>
        </div>
      </div>

      {activeCode && (
        <div className="border border-border rounded-lg p-4 space-y-3">
          <div className="text-sm font-medium">查询结果</div>
          <div className="grid grid-cols-2 gap-3 text-sm">
            <InfoRow label="完整路径" value={pathParts.join(' / ') || '—'} />
            <InfoRow label="区划代码" value={activeCode} mono />
            <InfoRow label="省级简称" value={plateInfo ? `${plateInfo.provinceAbbr}（${plateInfo.provinceName}）` : '—'} />
            <InfoRow
              label="车牌代号"
              value={
                plateInfo?.cityLetter
                  ? `${plateInfo.platePrefix}（${plateInfo.provinceAbbr}${plateInfo.cityLetter}）`
                  : plateInfo?.platePrefix ?? '—'
              }
              mono
            />
            <InfoRow label="示例号牌" value={plateInfo?.example ?? '—'} mono />
            <InfoRow
              label="邮政编码"
              value={
                postalInfo
                  ? `${postalInfo.postal}（${postalLevelLabel(postalInfo.level)}）`
                  : '—'
              }
              mono
            />
          </div>
          {plateInfo && !plateInfo.cityLetter && !['11', '12', '31', '50'].includes(activeCode.slice(0, 2)) && (
            <p className="text-xs text-muted-foreground">
              该地市暂无字母代号数据，仅显示省级简称；直辖市使用省级简称发牌。
            </p>
          )}
        </div>
      )}

      <ProvincePlateTable onSelect={selectRegionByCode} />
    </div>
  )
}

function InfoRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <div className="text-xs text-muted-foreground mb-0.5">{label}</div>
      <div className={mono ? 'font-mono text-foreground' : 'text-foreground'}>{value}</div>
    </div>
  )
}

function ProvincePlateTable({ onSelect }: { onSelect: (code: string) => void }) {
  const rows = useMemo(
    () =>
      Object.entries(
        REGIONS.reduce<Record<string, { name: string; abbr: string }>>((acc, r) => {
          const info = getPlateInfo(r.code)
          if (info) acc[r.code] = { name: r.name, abbr: info.provinceAbbr }
          return acc
        }, {}),
      ).map(([code, item]) => ({ code, ...item })),
    [],
  )

  return (
    <div>
      <div className="text-sm font-medium mb-2">各省车牌简称一览</div>
      <div className="border border-border rounded-lg overflow-hidden">
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-px bg-border">
          {rows.map((row) => (
            <button
              key={row.code}
              type="button"
              onClick={() => onSelect(row.code)}
              className="bg-background px-3 py-2 text-sm flex items-center gap-2 hover:bg-secondary/60 transition-colors text-left"
            >
              <span className="font-mono text-base font-semibold text-primary w-6">{row.abbr}</span>
              <span className="text-muted-foreground truncate">{row.name.replace(/(省|市|自治区|壮族|回族|维吾尔)/g, '')}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}

function SolarTermTool() {
  const now = useMemo(() => new Date(), [])
  const [year, setYear] = useState(now.getFullYear())
  const status = useMemo(() => getSolarTermStatus(now), [now])
  const terms = useMemo(() => getYearSolarTerms(year, now), [year, now])

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <div className="border border-border rounded-lg p-4 space-y-2">
        <div className="text-sm text-muted-foreground">
          今天 {formatDateTimeShanghai(now).slice(0, 10)}
        </div>
        {status.todayTerm ? (
          <div className="text-lg font-semibold text-emerald-600 dark:text-emerald-400">
            今日节气：{status.todayTerm}
          </div>
        ) : (
          <div className="text-sm">
            今日无节气交接
            {status.nextTerm && (
              <span className="text-muted-foreground">
                {' '}· 距 {status.nextTerm.name} 还有 {status.nextTerm.daysLeft} 天（{status.nextTerm.dateLabel}）
              </span>
            )}
          </div>
        )}
        {status.prevTerm && !status.todayTerm && (
          <div className="text-xs text-muted-foreground">
            上一节气：{status.prevTerm.name}（{status.prevTerm.dateLabel}）
          </div>
        )}
      </div>

      <div className="flex items-center gap-3">
        <label className="text-sm font-medium">年份</label>
        <select
          value={year}
          onChange={(e) => setYear(Number(e.target.value))}
          className="px-3 py-2 text-sm border border-border rounded-lg bg-background focus:outline-none focus:ring-2 focus:ring-primary"
        >
          {Array.from({ length: 5 }, (_, i) => now.getFullYear() - 2 + i).map((y) => (
            <option key={y} value={y}>{y} 年</option>
          ))}
        </select>
      </div>

      <div className="border border-border rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-secondary/50">
            <tr>
              <th className="px-3 py-2 text-left font-medium w-12">#</th>
              <th className="px-3 py-2 text-left font-medium">节气</th>
              <th className="px-3 py-2 text-left font-medium">公历日期</th>
              <th className="px-3 py-2 text-left font-medium">状态</th>
            </tr>
          </thead>
          <tbody>
            {terms.map((term, i) => (
              <tr
                key={term.name}
                className={`border-t border-border ${term.isToday ? 'bg-emerald-500/10' : ''}`}
              >
                <td className="px-3 py-2 text-muted-foreground">{i + 1}</td>
                <td className={`px-3 py-2 font-medium ${term.isToday ? 'text-emerald-600 dark:text-emerald-400' : ''}`}>
                  {term.name}
                </td>
                <td className="px-3 py-2 font-mono text-xs">{term.dateLabel}</td>
                <td className="px-3 py-2 text-xs text-muted-foreground">
                  {term.isToday ? '今天' : term.isPast ? '已过' : '未到'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="text-xs text-muted-foreground bg-secondary/30 p-3 rounded-lg">
        节气按太阳黄经划分，日期由天文算法计算，每年略有浮动。春分为公历 3 月 20 或 21 日前后。
      </div>
    </div>
  )
}

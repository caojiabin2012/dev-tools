// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import { Lunar } from 'lunar-javascript'

export const SOLAR_TERM_NAMES = [
  '立春', '雨水', '惊蛰', '春分', '清明', '谷雨',
  '立夏', '小满', '芒种', '夏至', '小暑', '大暑',
  '立秋', '处暑', '白露', '秋分', '寒露', '霜降',
  '立冬', '小雪', '大雪', '冬至', '小寒', '大寒',
] as const

export type SolarTermName = (typeof SOLAR_TERM_NAMES)[number]

export interface SolarTermEntry {
  name: SolarTermName
  date: Date
  dateLabel: string
  isPast: boolean
  isToday: boolean
}

export interface SolarTermStatus {
  todayTerm: string | null
  nextTerm: { name: string; date: Date; dateLabel: string; daysLeft: number } | null
  prevTerm: { name: string; date: Date; dateLabel: string } | null
}

interface JieQiSolar {
  getYear(): number
  getMonth(): number
  getDay(): number
}

function formatDateLabel(date: Date): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate())
}

function solarToDate(solar: JieQiSolar): Date {
  return new Date(solar.getYear(), solar.getMonth() - 1, solar.getDay())
}

function getJieQiTableForYear(year: number): Record<string, JieQiSolar> {
  const lunar = Lunar.fromYmd(year, 6, 1)
  return lunar.getJieQiTable() as Record<string, JieQiSolar>
}

/** 获取指定公历年的 24 节气（含当年小寒、大寒及次年交接项，按名称去重取该年日期） */
export function getYearSolarTerms(year: number, refDate = new Date()): SolarTermEntry[] {
  const table = getJieQiTableForYear(year)
  const today = startOfDay(refDate)

  return SOLAR_TERM_NAMES.map((name) => {
    const solar = table[name]
    if (!solar) return null
    const date = solarToDate(solar)
    if (date.getFullYear() !== year) return null
    return {
      name,
      date,
      dateLabel: formatDateLabel(date),
      isPast: date < today,
      isToday: date.getTime() === today.getTime(),
    }
  }).filter((item): item is SolarTermEntry => item !== null)
    .sort((a, b) => a.date.getTime() - b.date.getTime())
}

export function getSolarTermStatus(refDate = new Date()): SolarTermStatus {
  const year = refDate.getFullYear()
  const terms = [
    ...getYearSolarTerms(year - 1, refDate),
    ...getYearSolarTerms(year, refDate),
    ...getYearSolarTerms(year + 1, refDate),
  ].sort((a, b) => a.date.getTime() - b.date.getTime())

  const today = startOfDay(refDate)
  const todayTerm = terms.find((t) => t.isToday)?.name ?? null

  let prevTerm: SolarTermStatus['prevTerm'] = null
  let nextTerm: SolarTermStatus['nextTerm'] = null

  for (const term of terms) {
    if (term.date <= today) {
      prevTerm = { name: term.name, date: term.date, dateLabel: term.dateLabel }
    } else if (!nextTerm) {
      const daysLeft = Math.round((term.date.getTime() - today.getTime()) / 86400000)
      nextTerm = { name: term.name, date: term.date, dateLabel: term.dateLabel, daysLeft }
      break
    }
  }

  return { todayTerm, nextTerm, prevTerm }
}

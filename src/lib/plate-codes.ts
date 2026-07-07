/** 省级行政区车牌简称（单字） */
export const PROVINCE_PLATE_CHARS =
  '京津冀沪渝豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼'

export const PROVINCE_PLATE: Record<string, { abbr: string; name: string }> = {
  '11': { abbr: '京', name: '北京' },
  '12': { abbr: '津', name: '天津' },
  '13': { abbr: '冀', name: '河北' },
  '14': { abbr: '晋', name: '山西' },
  '15': { abbr: '蒙', name: '内蒙古' },
  '21': { abbr: '辽', name: '辽宁' },
  '22': { abbr: '吉', name: '吉林' },
  '23': { abbr: '黑', name: '黑龙江' },
  '31': { abbr: '沪', name: '上海' },
  '32': { abbr: '苏', name: '江苏' },
  '33': { abbr: '浙', name: '浙江' },
  '34': { abbr: '皖', name: '安徽' },
  '35': { abbr: '闽', name: '福建' },
  '36': { abbr: '赣', name: '江西' },
  '37': { abbr: '鲁', name: '山东' },
  '41': { abbr: '豫', name: '河南' },
  '42': { abbr: '鄂', name: '湖北' },
  '43': { abbr: '湘', name: '湖南' },
  '44': { abbr: '粤', name: '广东' },
  '45': { abbr: '桂', name: '广西' },
  '46': { abbr: '琼', name: '海南' },
  '50': { abbr: '渝', name: '重庆' },
  '51': { abbr: '川', name: '四川' },
  '52': { abbr: '贵', name: '贵州' },
  '53': { abbr: '云', name: '云南' },
  '54': { abbr: '藏', name: '西藏' },
  '61': { abbr: '陕', name: '陕西' },
  '62': { abbr: '甘', name: '甘肃' },
  '63': { abbr: '青', name: '青海' },
  '64': { abbr: '宁', name: '宁夏' },
  '65': { abbr: '新', name: '新疆' },
}

/** 地级行政区车牌字母代号（国标行政区划代码 → 字母） */
export const CITY_PLATE_LETTER: Record<string, string> = {
  // 河北
  '130100': 'A', '130200': 'B', '130300': 'C', '130400': 'D', '130500': 'E',
  '130600': 'F', '130700': 'G', '130800': 'H', '130900': 'J', '131000': 'R', '131100': 'T',
  '133100': 'X', // 雄安新区（2023 年起启用冀X）
  // 山西
  '140100': 'A', '140200': 'B', '140300': 'C', '140400': 'D', '140500': 'E',
  '140600': 'F', '140700': 'H', '140800': 'J', '140900': 'K', '141000': 'L', '141100': 'M',
  // 内蒙古
  '150100': 'A', '150200': 'B', '150300': 'C', '150400': 'D', '150500': 'E',
  '150600': 'F', '150700': 'E', '150800': 'K', '150900': 'J',
  // 辽宁
  '210100': 'A', '210200': 'B', '210300': 'C', '210400': 'D', '210500': 'E',
  '210600': 'F', '210700': 'G', '210800': 'H', '210900': 'J', '211000': 'K',
  '211100': 'L', '211200': 'M', '211300': 'N', '211400': 'P',
  // 吉林
  '220100': 'A', '220200': 'B', '220300': 'C', '220400': 'D', '220500': 'E',
  '220600': 'F', '220700': 'G', '220800': 'H',
  // 黑龙江
  '230100': 'A', '230200': 'B', '230300': 'C', '230400': 'D', '230500': 'E',
  '230600': 'F', '230700': 'G', '230800': 'H', '230900': 'J', '231000': 'K',
  '231100': 'L', '231200': 'M',
  // 江苏
  '320100': 'A', '320200': 'B', '320300': 'C', '320400': 'D', '320500': 'E',
  '320600': 'F', '320700': 'G', '320800': 'H', '320900': 'J', '321000': 'K',
  '321100': 'L', '321200': 'M', '321300': 'N',
  // 浙江
  '330100': 'A', '330200': 'B', '330300': 'C', '330400': 'D', '330500': 'E',
  '330600': 'F', '330700': 'G', '330800': 'H', '330900': 'J', '331000': 'K',
  '331100': 'L',
  // 安徽
  '340100': 'A', '340200': 'B', '340300': 'C', '340400': 'D', '340500': 'E',
  '340600': 'F', '340700': 'G', '340800': 'H', '341000': 'J', '341100': 'M',
  '341200': 'K', '341300': 'L', '341500': 'N', '341600': 'S', '341700': 'R',
  '341800': 'P',
  // 福建
  '350100': 'A', '350200': 'B', '350300': 'C', '350400': 'D', '350500': 'E',
  '350600': 'F', '350700': 'H', '350800': 'J', '350900': 'K',
  // 江西
  '360100': 'A', '360200': 'H', '360300': 'J', '360400': 'G', '360500': 'K',
  '360600': 'L', '360700': 'B', '360800': 'D', '360900': 'C', '361000': 'F',
  '361100': 'E',
  // 山东
  '370100': 'A', '370200': 'B', '370300': 'C', '370400': 'D', '370500': 'E',
  '370600': 'F', '370700': 'G', '370800': 'H', '370900': 'J', '371000': 'K',
  '371100': 'L', '371300': 'Q', '371400': 'N', '371500': 'P', '371600': 'M',
  '371700': 'R',
  // 河南
  '410100': 'A', '410200': 'B', '410300': 'C', '410400': 'D', '410500': 'E',
  '410600': 'F', '410700': 'G', '410800': 'H', '410900': 'J', '411000': 'K',
  '411100': 'L', '411200': 'M', '411300': 'R', '411400': 'N', '411500': 'S',
  '411600': 'P', '411700': 'Q',
  // 湖北
  '420100': 'A', '420200': 'B', '420300': 'C', '420500': 'E', '420600': 'F',
  '420700': 'G', '420800': 'H', '420900': 'J', '421000': 'D', '421100': 'J',
  '421200': 'L', '421300': 'S',
  // 湖南
  '430100': 'A', '430200': 'B', '430300': 'C', '430400': 'D', '430500': 'E',
  '430600': 'F', '430700': 'G', '430800': 'H', '430900': 'J', '431000': 'K',
  '431100': 'M', '431200': 'N', '431300': 'L',
  // 广东
  '440100': 'A', '440200': 'F', '440300': 'B', '440400': 'C', '440500': 'D',
  '440600': 'E', '440700': 'F', '440800': 'G', '440900': 'H', '441200': 'J',
  '441300': 'K', '441400': 'L', '441500': 'M', '441600': 'N', '441700': 'P',
  '441800': 'Q', '441900': 'S', '442000': 'T', '445100': 'U', '445200': 'V',
  '445300': 'W',
  // 广西
  '450100': 'A', '450200': 'B', '450300': 'C', '450400': 'D', '450500': 'E',
  '450600': 'F', '450700': 'G', '450800': 'H', '450900': 'J', '451000': 'K',
  '451100': 'J', '451200': 'M', '451300': 'G', '451400': 'F',
  // 海南
  '460100': 'A', '460200': 'B',
  // 四川
  '510100': 'A', '510300': 'C', '510400': 'D', '510500': 'E', '510600': 'F',
  '510700': 'B', '510800': 'H', '510900': 'J', '511000': 'K', '511100': 'L',
  '511300': 'R', '511400': 'M', '511500': 'Q', '511600': 'N', '511700': 'S',
  '511800': 'T', '511900': 'Y', '512000': 'M', '513200': 'U', '513300': 'V',
  '513400': 'W',
  // 贵州
  '520100': 'A', '520200': 'B', '520300': 'C', '520400': 'G', '520500': 'F',
  '520600': 'D', '522300': 'E', '522600': 'H', '522700': 'J',
  // 云南
  '530100': 'A', '530300': 'C', '530400': 'D', '530500': 'E', '530600': 'F',
  '530700': 'G', '530800': 'H', '530900': 'J', '532300': 'E', '532500': 'G',
  '532600': 'H', '532800': 'K', '532900': 'L', '533100': 'M', '533300': 'Q',
  '533400': 'P',
  // 西藏
  '540100': 'A', '540200': 'D', '540300': 'B', '540400': 'C', '540500': 'E',
  '540600': 'F',
  // 陕西
  '610100': 'A', '610200': 'B', '610300': 'C', '610400': 'D', '610500': 'E',
  '610600': 'J', '610700': 'F', '610800': 'K', '610900': 'G', '611000': 'H',
  // 甘肃
  '620100': 'A', '620200': 'B', '620300': 'C', '620400': 'D', '620500': 'E',
  '620600': 'H', '620700': 'G', '620800': 'K', '620900': 'J', '621000': 'M',
  '621100': 'J', '621200': 'L',
  // 青海
  '630100': 'A', '630200': 'B',
  // 宁夏
  '640100': 'A', '640200': 'B', '640300': 'C', '640400': 'D', '640500': 'E',
  // 新疆
  '650100': 'A', '650200': 'B', '650400': 'K', '650500': 'L', '652300': 'B',
  '652700': 'E', '652800': 'M', '652900': 'N', '653000': 'P', '653100': 'Q',
  '653200': 'R', '654000': 'F', '654200': 'G', '654300': 'H',
}

const MUNICIPALITY_PREFIXES = new Set(['11', '12', '31', '50'])

export interface PlateInfo {
  provinceAbbr: string
  provinceName: string
  cityLetter: string | null
  platePrefix: string
  example: string
}

function cityCodeFromRegionCode(code: string): string {
  if (code.length < 6) return code.padEnd(6, '0')
  return `${code.slice(0, 4)}00`
}

export function getPlateInfo(regionCode: string): PlateInfo | null {
  if (!regionCode || regionCode.length < 2) return null
  const prefix = regionCode.slice(0, 2)
  const province = PROVINCE_PLATE[prefix]
  if (!province) return null

  const cityCode = cityCodeFromRegionCode(regionCode)
  const cityLetter = CITY_PLATE_LETTER[cityCode] ?? null
  const isMunicipality = MUNICIPALITY_PREFIXES.has(prefix)

  let platePrefix = province.abbr
  if (cityLetter && !isMunicipality) {
    platePrefix = `${province.abbr}${cityLetter}`
  }

  return {
    provinceAbbr: province.abbr,
    provinceName: province.name,
    cityLetter: isMunicipality ? null : cityLetter,
    platePrefix,
    example: `${platePrefix}·12345`,
  }
}

export interface PlateLookupResult {
  platePrefix: string
  provinceAbbr: string
  provinceName: string
  provinceCode: string
  cityLetter: string | null
  cityName: string | null
  cityCode: string | null
  regionPath: string
}

/** 从号牌或代号中解析省级简称 + 地市字母，如「冀H12345」「冀 H」→ 冀 + H */
export function parsePlatePrefix(input: string): { abbr: string; letter: string | null } | null {
  const trimmed = input.trim().replace(/[·.\s]/g, '').toUpperCase()
  if (!trimmed) return null

  const abbr = trimmed[0]
  if (!PROVINCE_PLATE_CHARS.includes(abbr)) return null

  const rest = trimmed.slice(1).replace(/[^A-HJ-NP-Z0-9]/g, '')
  const letterMatch = rest.match(/^[A-HJ-NP-Z]/)
  return { abbr, letter: letterMatch?.[0] ?? null }
}

function findProvinceByAbbr(abbr: string): { code: string; name: string; abbr: string } | null {
  for (const [code, info] of Object.entries(PROVINCE_PLATE)) {
    if (info.abbr === abbr) {
      return { code, name: info.name, abbr: info.abbr }
    }
  }
  return null
}

/** 车牌代号反查地区，如「冀H」→ 河北省承德市 */
export function lookupByPlatePrefix(
  input: string,
  cityNames: Record<string, string>,
): PlateLookupResult[] {
  const parsed = parsePlatePrefix(input)
  if (!parsed) return []

  const province = findProvinceByAbbr(parsed.abbr)
  if (!province) return []

  const provinceCode = `${province.code}0000`

  if (!parsed.letter) {
    return [{
      platePrefix: province.abbr,
      provinceAbbr: province.abbr,
      provinceName: province.name,
      provinceCode,
      cityLetter: null,
      cityName: null,
      cityCode: null,
      regionPath: province.name,
    }]
  }

  if (MUNICIPALITY_PREFIXES.has(province.code)) {
    return [{
      platePrefix: `${province.abbr}${parsed.letter}`,
      provinceAbbr: province.abbr,
      provinceName: province.name,
      provinceCode,
      cityLetter: parsed.letter,
      cityName: `${province.name}（${province.abbr}${parsed.letter} 为市区号牌）`,
      cityCode: null,
      regionPath: province.name,
    }]
  }

  const matches: PlateLookupResult[] = []
  for (const [cityCode, letter] of Object.entries(CITY_PLATE_LETTER)) {
    if (letter !== parsed.letter || !cityCode.startsWith(province.code)) continue
    const cityName = cityNames[cityCode]
    if (!cityName) continue
    matches.push({
      platePrefix: `${province.abbr}${parsed.letter}`,
      provinceAbbr: province.abbr,
      provinceName: province.name,
      provinceCode,
      cityLetter: parsed.letter,
      cityName,
      cityCode,
      regionPath: `${province.name}/${cityName}`,
    })
  }

  return matches.sort((a, b) => (a.cityCode ?? '').localeCompare(b.cityCode ?? ''))
}

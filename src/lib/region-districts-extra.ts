interface DistrictNode {
  code: string
  name: string
}

/** 补全 REGIONS 中仅有地市、缺少区县的条目（国家统计局区划代码） */
export const EXTRA_CITY_DISTRICTS: Record<string, DistrictNode[]> = {
  '130500': [
    { code: '130502', name: '襄都区' }, { code: '130503', name: '信都区' },
    { code: '130505', name: '任泽区' }, { code: '130506', name: '南和区' },
    { code: '130522', name: '临城县' }, { code: '130523', name: '内丘县' },
    { code: '130524', name: '柏乡县' }, { code: '130525', name: '隆尧县' },
    { code: '130528', name: '宁晋县' }, { code: '130529', name: '巨鹿县' },
    { code: '130530', name: '新河县' }, { code: '130531', name: '广宗县' },
    { code: '130532', name: '平乡县' }, { code: '130533', name: '威县' },
    { code: '130534', name: '清河县' }, { code: '130535', name: '临西县' },
    { code: '130581', name: '南宫市' }, { code: '130582', name: '沙河市' },
  ],
  '130600': [
    { code: '130602', name: '竞秀区' }, { code: '130606', name: '莲池区' },
    { code: '130607', name: '满城区' }, { code: '130608', name: '清苑区' },
    { code: '130609', name: '徐水区' }, { code: '130623', name: '涞水县' },
    { code: '130624', name: '阜平县' }, { code: '130626', name: '定兴县' },
    { code: '130627', name: '唐县' }, { code: '130628', name: '高阳县' },
    { code: '130629', name: '容城县' }, { code: '130630', name: '涞源县' },
    { code: '130631', name: '望都县' }, { code: '130632', name: '安新县' },
    { code: '130633', name: '易县' }, { code: '130634', name: '曲阳县' },
    { code: '130635', name: '蠡县' }, { code: '130636', name: '顺平县' },
    { code: '130637', name: '博野县' }, { code: '130638', name: '雄县' },
    { code: '130681', name: '涿州市' }, { code: '130682', name: '定州市' },
    { code: '130683', name: '安国市' }, { code: '130684', name: '高碑店市' },
  ],
  '130700': [
    { code: '130702', name: '桥东区' }, { code: '130703', name: '桥西区' },
    { code: '130705', name: '宣化区' }, { code: '130706', name: '下花园区' },
    { code: '130708', name: '万全区' }, { code: '130709', name: '崇礼区' },
    { code: '130722', name: '张北县' }, { code: '130723', name: '康保县' },
    { code: '130724', name: '沽源县' }, { code: '130725', name: '尚义县' },
    { code: '130726', name: '蔚县' }, { code: '130727', name: '阳原县' },
    { code: '130728', name: '怀安县' }, { code: '130730', name: '怀来县' },
    { code: '130731', name: '涿鹿县' }, { code: '130732', name: '赤城县' },
  ],
  '130800': [
    { code: '130802', name: '双桥区' }, { code: '130803', name: '双滦区' },
    { code: '130804', name: '鹰手营子矿区' }, { code: '130821', name: '承德县' },
    { code: '130822', name: '兴隆县' }, { code: '130824', name: '滦平县' },
    { code: '130825', name: '隆化县' }, { code: '130826', name: '丰宁满族自治县' },
    { code: '130827', name: '宽城满族自治县' }, { code: '130828', name: '围场满族蒙古族自治县' },
    { code: '130881', name: '平泉市' },
  ],
  '130900': [
    { code: '130902', name: '新华区' }, { code: '130903', name: '运河区' },
    { code: '130921', name: '沧县' }, { code: '130922', name: '青县' },
    { code: '130923', name: '东光县' }, { code: '130924', name: '海兴县' },
    { code: '130925', name: '盐山县' }, { code: '130926', name: '肃宁县' },
    { code: '130927', name: '南皮县' }, { code: '130928', name: '吴桥县' },
    { code: '130929', name: '献县' }, { code: '130930', name: '孟村回族自治县' },
    { code: '130981', name: '泊头市' }, { code: '130982', name: '任丘市' },
    { code: '130983', name: '黄骅市' }, { code: '130984', name: '河间市' },
  ],
  '131000': [
    { code: '131002', name: '安次区' }, { code: '131003', name: '广阳区' },
    { code: '131022', name: '固安县' }, { code: '131023', name: '永清县' },
    { code: '131024', name: '香河县' }, { code: '131025', name: '大城县' },
    { code: '131026', name: '文安县' }, { code: '131028', name: '大厂回族自治县' },
    { code: '131081', name: '霸州市' }, { code: '131082', name: '三河市' },
  ],
  '131100': [
    { code: '131102', name: '桃城区' }, { code: '131103', name: '冀州区' },
    { code: '131121', name: '枣强县' }, { code: '131122', name: '武邑县' },
    { code: '131123', name: '武强县' }, { code: '131124', name: '饶阳县' },
    { code: '131125', name: '安平县' }, { code: '131126', name: '故城县' },
    { code: '131127', name: '景县' }, { code: '131128', name: '阜城县' },
    { code: '131182', name: '深州市' },
  ],
}

export function mergeExtraDistricts<T extends { code: string; name: string; children?: T[] }>(
  nodes: T[],
): T[] {
  return nodes.map((node) => {
    if (!node.children) return node
    const children = node.children.map((city) => {
      if (city.children?.length) return city
      const extra = EXTRA_CITY_DISTRICTS[city.code]
      return extra ? { ...city, children: extra as T[] } : city
    })
    return { ...node, children }
  })
}

# -*- coding: utf-8 -*-
"""
2026학년도 전 자료 → 성적분석ON .db (스키마 v3).

기존 etl_2026_g1.py(고1 3·6·9월 모의고사, 입학)를 재사용하고 아래를 더한다.
  · 전학년 명렬표(1·2·3학년) + 학적변동
  · 나이스 교과학습발달상황 download-*.xlsx (1·2학년 반별 20파일) — Rust 파서와 같은 규칙
  · 고2 학력평가 3·6월 (2022개정 파일, 학번 5자리)
  · 편제표 (성적데이터.xlsx 편제 시트)
  · 수시 진학결과 2023~2025 (이름 제외)
  · 강원 지역 대학별 전형별 지원·합격 범위 (4) 분석자료)
사용: python tools/etl_2026_all.py [--src 폴더] [--out 폴더]
"""
import argparse, glob, io, json, os, re, sqlite3, sys, warnings
from datetime import datetime

warnings.filterwarnings('ignore')
import pandas as pd

sys.path.insert(0, os.path.dirname(__file__))
import etl_2026_g1 as g1

if not isinstance(sys.stdout, io.TextIOWrapper) or sys.stdout.encoding.lower() != 'utf-8':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

DEFAULT_SRC = r"C:/Users/User/AppData/Local/Temp/BNZ.6aa764dd16db6f68/"
DEFAULT_OUT = g1.DEFAULT_OUT
SCHEMA = g1.SCHEMA
GRADE_YEAR = 2026
UNIV_SEED = [
    ('서울권', '서울대', '인문/자연', '지역균형(교과)', 1.5), ('서울권', '연세대', '인문/자연', '추천형(교과)', 1.6), ('서울권', '고려대', '인문/자연', '학교추천(교과 80 + 서류 20)', 1.7),
    ('서울권', '한양대', '인문/자연', '추천형(교과)', 1.8), ('서울권', '성균관대', '인문/자연', '학교장추천(교과)', 1.9), ('서울권', '서강대', '인문/자연', '지역균형(교과)', 2.0),
    ('서울권', '중앙대', '인문/자연', '지역균형(교과)', 2.1), ('서울권', '서울시립대', '인문/자연', '지역균형(교과)', 2.2), ('서울권', '한국외대', '인문', '학교장추천(교과)', 2.3),
    ('서울권', '건국대', '인문/자연', 'KU지역균형(교과)', 2.4), ('서울권', '숙명여대', '인문/자연', '지역균형(교과)', 2.6), ('서울권', '세종대', '인문/자연', '지역균형(교과)', 2.8),
    ('서울권', '국민대', '인문/자연', '교과성적우수자', 3.0), ('서울권', '숭실대', '인문/자연', '학생부우수자(교과)', 3.0), ('지방거점국립', '부산대', '인문/자연', '학생부교과', 2.6),
    ('지방거점국립', '경북대', '인문/자연', '교과우수자', 2.7), ('지방거점국립', '충남대', '인문/자연', '일반전형(교과)', 3.0), ('지방거점국립', '전남대', '인문/자연', '학생부교과(일반)', 3.2),
    ('지방거점국립', '충북대', '인문/자연', '학생부교과', 3.3), ('지방거점국립', '전북대', '인문/자연', '일반학생(교과)', 3.4), ('지방거점국립', '제주대', '인문/자연', '일반학생(교과)', 3.7),
    ('지방거점국립', '경상국립대', '인문/자연', '일반전형(교과)', 3.8), ('강원권', '강원대(춘천)', '의예과', '지역인재(교과)', 1.2), ('강원권', '한림대', '의예과', '지역인재(교과)', 1.2),
    ('강원권', '강원대(춘천)', '간호학과', '일반전형(교과)', 2.4), ('강원권', '한림대', '간호학과', '학생부교과(일반)', 2.5), ('강원권', '연세대 미래캠퍼스', '인문/자연', '교과우수자', 3.2),
    ('강원권', '강원대(춘천)', '인문/자연', '일반전형(교과)', 3.6), ('강원권', '한림대', '인문/자연', '학생부교과(일반)', 3.7), ('강원권', '강릉원주대', '인문/자연', '교과일반', 4.4),
    ('강원권', '가톨릭관동대', '인문/자연', '교과일반', 4.6), ('강원권', '한라대', '인문/자연', '일반전형(교과)', 5.0),
]


def s(v):
    if v is None or (isinstance(v, float) and pd.isna(v)):
        return ''
    return str(v).strip()


def num(v):
    return g1.num(v)


def norm_area(a):
    t = s(a).replace(' ', '')
    if not t:
        return ''
    if t.startswith('사회'):
        return '사회'
    if t.startswith('기술') or '정보' in t:
        return '기술·가정/정보'
    if t.startswith('제2외국어') or '한문' in t:
        return '제2외국어/한문'
    return t


# ── 명렬표 ──────────────────────────────────────────────────

def read_roster(src):
    found = glob.glob(src + '*명렬표*.xlsx')
    if not found:
        return {}, []
    f = found[0]
    df = pd.read_excel(f, sheet_name='Sheet2', header=None).iloc[1:]
    df = df[df[2].notna()]
    students = {}
    for _, r in df.iterrows():
        hk = s(r[2])
        if not re.fullmatch(r'\d{5}', hk):
            continue
        g, c, n = int(hk[0]), int(hk[1:3]), int(hk[3:5])
        students[(g, c, n)] = s(r[1])
    changes = []
    try:
        ch = pd.read_excel(f, sheet_name='Sheet3', header=None).iloc[1:]
        for _, r in ch.iterrows():
            if pd.isna(r[2]):
                continue
            changes.append(dict(name=s(r[2]), grade=int(num(r[4]) or 0), class_no=int(num(r[5]) or 0), number=int(num(r[6]) or 0), kind=s(r[7]), date=s(r[1]), school=s(r[8])))
    except Exception:
        pass
    return students, changes


# ── 나이스 교과학습발달상황 ───────────────────────────────────

def parse_neis(path):
    x = pd.read_excel(path, header=None)
    grade = class_no = None
    for r in range(min(6, len(x))):
        for c in range(min(6, x.shape[1])):
            m = re.search(r'(\d)학년\s*(\d+)반', s(x.iat[r, c]))
            if m:
                grade, class_no = int(m.group(1)), int(m.group(2))
    if grade is None:
        raise ValueError(f'반 정보 없음: {path}')
    rows, layout, number, name, g, sem = [], None, None, '', None, None
    for r in range(len(x)):
        c0 = s(x.iat[r, 0]).replace(' ', '')
        c5 = s(x.iat[r, 5]) if x.shape[1] > 5 else ''
        c7 = s(x.iat[r, 7]) if x.shape[1] > 7 else ''
        if c0 == '번호':
            layout = 'five' if '원점수' in c7 else 'three'
            continue
        if c0.startswith('이수학점') or layout is None or c5 in ('', '·', 'nan'):
            continue
        n0 = num(x.iat[r, 0])
        if n0 is not None:
            number, name, g, sem = int(n0), s(x.iat[r, 1]), None, None
        if number is None:
            continue
        if num(x.iat[r, 2]) is not None:
            g = int(num(x.iat[r, 2]))
        if num(x.iat[r, 3]) is not None:
            sem = int(num(x.iat[r, 3]))
        if g is None or sem is None:
            continue
        area = norm_area(x.iat[r, 4])
        credits = num(x.iat[r, 6]) or 0
        base = dict(class_no=class_no, number=number, name=name, grade=g, semester=sem, area=area, subject=c5, credits=credits)
        if layout == 'five':
            raw = mean = None
            if '/' in c7:
                a, b = c7.split('/', 1)
                raw, mean = num(a), num(b)
            ach = s(x.iat[r, 8])
            ach_v = ach if ach in list('ABCDE') else None
            dist = [None] * 5
            for i, L in enumerate('ABCDE'):
                m = re.search(L + r'\(([\d.]+)\)', s(x.iat[r, 9]))
                if m:
                    dist[i] = float(m.group(1))
            note = s(x.iat[r, 12]) if x.shape[1] > 12 else ''
            rows.append(dict(base, raw=raw, subject_mean=mean, achievement=ach_v, ach_scale=5 if ach_v else 0, dist=dist,
                             grade5=int(num(x.iat[r, 10])) if num(x.iat[r, 10]) is not None else None,
                             enrolled_n=int(num(x.iat[r, 11])) if num(x.iat[r, 11]) is not None else None, note=note or None))
        else:
            ach = c7
            ach_v, scale = (ach, 3) if ach in ('A', 'B', 'C') else (('P', 0) if ach in ('P', '이수') else (None, 0))
            note = s(x.iat[r, 8]) if x.shape[1] > 8 else ''
            rows.append(dict(base, raw=None, subject_mean=None, achievement=ach_v, ach_scale=scale, dist=[None] * 5, grade5=None, enrolled_n=None, note=note or None))
    return grade, class_no, rows


# ── 고2 모의고사 (2022개정 파일) ─────────────────────────────

def read_g2_mock(src):
    found = [p for p in glob.glob(src + '*고2 학력평가*2022개정*.xlsb')]
    out, roster = [], {}
    if not found:
        return roster, out
    xl = pd.ExcelFile(found[0], engine='pyxlsb')
    cols = {'국어': (7, 8, 9), '수학': (10, 11, 12), '영어': (None, None, 13), '한국사': (None, None, 6), '통합사회': (14, 15, 16), '통합과학': (17, 18, 19), '제2외국어': (None, None, 20)}
    for sh, rd in [('3월', 3), ('6월', 6)]:
        df = xl.parse(sh, header=None).iloc[4:]
        df = df[df[4].notna() & df[5].notna()]
        df = df[df[5].astype(str).str.strip() != '0']
        for _, r in df.iterrows():
            g, c, n = g1.parse_hakbun(r[4])
            roster[(g, c, n)] = g1.norm_name(r[5])
            whole_absent = g1.is_absent(r[6])
            for subj, (std, pct, grd) in cols.items():
                vals = [num(r[i]) if i is not None else None for i in (std, pct, grd)]
                absent = whole_absent or any(g1.is_absent(r[i]) for i in (std, pct, grd) if i is not None)
                if subj == '제2외국어' and all(v is None for v in vals):
                    continue
                if all(v is None for v in vals) and not absent:
                    continue
                out.append(dict(grade=g, class_no=c, number=n, round=rd, subject=subj, raw=None, std=vals[0], pct=vals[1],
                                grade9=int(vals[2]) if vals[2] is not None else None, absent=int(absent)))
    return roster, out


# ── 편제 ──────────────────────────────────────────────────────

def read_catalog(src):
    f = src + '_대시보드초안/성적데이터.xlsx'
    if not os.path.exists(f):
        return []
    df = pd.read_excel(f, sheet_name='편제')
    return [(s(r['과목']), s(r['구분'])) for _, r in df.iterrows() if s(r['과목'])]


# ── 진학결과 ──────────────────────────────────────────────────

def read_admissions(src):
    rows = []
    for f in sorted(glob.glob(src + '*학년도 수시 진학결과*.xlsx')):
        yr = int(re.search(r'(\d{4})학년도', os.path.basename(f)).group(1))
        x = pd.read_excel(f, header=None)
        h = [s(v) for v in x.iloc[0]]
        body = x.iloc[2:]
        col = {name: i for i, name in enumerate(h) if name}
        for _, r in body.iterrows():
            if not s(r[col.get('대학명', 6)]):
                continue
            def get(k, default=None):
                return r[col[k]] if k in col else default
            fin = s(get('최종단계'))
            rows.append(dict(
                adm_year=yr, class_no=int(num(get('반')) or 0) or None, number=int(num(get('번호')) or 0) or None,
                region=s(get('지역')), univ=s(get('대학명')), adm_type=s(get('전형유형')), adm_name=s(get('전형명(대)', get('전형'))), detail=s(get('세부유형')),
                track=s(get('계열')), unit=s(get('모집단위')), quota=int(num(get('모집인원')) or 0) or None, select_type=s(get('선발유형')), min_req=s(get('최저학력기준')),
                stage1=s(get('1단계')), final=fin, registered=s(get('등록여부')),
                grade_all=num(r[col['전교과']]) if '전교과' in col else None,
                grade_main=num(r[col['국영수사/과']]) if '국영수사/과' in col else None,
                grade_kem=num(r[col['국영수']]) if '국영수' in col else None,
                grade_kor=num(r[col['국어']]) if '국어' in col and col['국어'] < 40 else None,
                grade_eng=num(r[col['영어']]) if '영어' in col and col['영어'] < 40 else None,
                grade_math=num(r[col['수학']]) if '수학' in col and col['수학'] < 40 else None,
                grade_soc=num(r[col['사회']]) if '사회' in col else None,
                grade_sci=num(r[col['과학']]) if '과학' in col else None,
                csat_note=s(get('국수영(탐1탐2)')),
            ))
    return rows


def read_regional(src):
    f = glob.glob(src + '4) 분석자료*.xlsx')
    if not f:
        return []
    xl = pd.ExcelFile(f[0])
    out = []
    for sh in xl.sheet_names:
        track = '인문' if sh.startswith('인문') else '자연' if sh.startswith('자연') else None
        cat = '교과' if '교과' in sh else '종합' if '종합' in sh else None
        if not track or not cat:
            continue
        df = xl.parse(sh, header=None).iloc[3:]
        for _, r in df.iterrows():
            if not s(r[0]) or s(r[0]).startswith('“'):
                continue
            out.append(dict(source=os.path.basename(f[0]), track=track, category=cat, univ=s(r[0]), adm_name=s(r[1]),
                            apply_n=int(num(r[2]) or 0), apply_best=num(r[3]), apply_worst=num(r[4]), pass_n=int(num(r[5]) or 0),
                            pass_best=num(r[6]), pass_worst=num(r[7]), pass_rate=num(r[8]), cut70=num(r[9])))
    return out


# ── DB ─────────────────────────────────────────────────────────

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--src', default=DEFAULT_SRC)
    ap.add_argument('--out', default=DEFAULT_OUT)
    a = ap.parse_args()
    src = a.src if a.src.endswith('/') else a.src + '/'
    os.makedirs(a.out, exist_ok=True)
    report = {'generated': datetime.now().isoformat(timespec='seconds'), 'warnings': []}

    # 1) 명단: 명렬표 우선, 고1 파일 로스터로 보강
    roster, changes = read_roster(src)
    if not roster:
        report['warnings'].append('명렬표 파일 없음 — 모의고사·나이스 파일의 로스터로 명단을 만듭니다.')
    g1.FILES = {k: v if os.path.exists(v) else src + os.path.basename(v) for k, v in g1.FILES.items()}
    missing = [k for k, v in g1.FILES.items() if not os.path.exists(v)]
    if missing:
        raise SystemExit(f'고1 파일이 없습니다: {missing} (--src 폴더 또는 G:\...\2026년 에 있어야 합니다)')
    r3, rows3 = g1.read_march()
    r6, rows6 = g1.read_june()
    r9, rows9, notes9 = g1.read_sept()
    for rr in (r3, r6, r9):
        for (c, n), nm in rr.items():
            roster.setdefault((1, c, n), nm)
    g2roster, g2mock = read_g2_mock(src)
    for k, nm in g2roster.items():
        roster.setdefault(k, nm)
    report['students'] = {g: sum(1 for k in roster if k[0] == g) for g in (1, 2, 3)}

    # 2) NEIS
    neis_rows, neis_files = [], []
    for f in sorted(glob.glob(src + 'download-*.xlsx')):
        try:
            g, c, rows = parse_neis(f)
        except Exception as e:
            report['warnings'].append(f'{os.path.basename(f)}: {e}')
            continue
        neis_files.append((os.path.basename(f), g, c, len(rows), len({r["number"] for r in rows})))
        for r in rows:
            r['cur_grade'] = g
            roster.setdefault((g, c, r['number']), r['name'])
        neis_rows.extend(rows)
    report['neis_files'] = neis_files
    catalog = read_catalog(src)
    admissions = read_admissions(src)
    regional = read_regional(src)

    # 3) DB
    db_path = os.path.join(a.out, '2026_춘천고_성적분석.db')
    if os.path.exists(db_path):
        os.remove(db_path)
    con = sqlite3.connect(db_path)
    con.execute('PRAGMA foreign_keys = ON')
    with open(SCHEMA, encoding='utf-8') as f:
        con.executescript(f.read())
    con.execute('PRAGMA user_version = 3')
    con.execute("INSERT INTO APP_CONFIGS (config_key, config_value) VALUES ('app_version', ?)", (g1.APP_VERSION,))
    for i, (region, univ, unit, adm, cut) in enumerate(UNIV_SEED):
        con.execute("INSERT INTO UnivReference (region, univ, unit, adm_type, cut, note, sort_order) VALUES (?,?,?,?,?,'예시값 — 학교 입결로 교체',?)", (region, univ, unit, adm, cut, i))
    for subj, kind in catalog:
        con.execute('INSERT OR REPLACE INTO SubjectCatalog (subject, area, kind) VALUES (?, NULL, ?)', (subj, kind))

    ids = {}
    for i, ((g, c, n), nm) in enumerate(sorted(roster.items()), start=1):
        con.execute('INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?,?,?,?,?,?)', (f'S{i:05d}', GRADE_YEAR - g + 1, g, c, n, nm))
        ids[(g, c, n)] = i

    # 학적변동 → 제외 (전출·자퇴만; 전입·재입학은 활성)
    excluded = []
    for ch in changes:
        k = (ch['grade'], ch['class_no'], ch['number'])
        if k in ids and any(w in ch['kind'] for w in ('전출', '자퇴', '제적', '휴학')):
            reason = f"{ch['kind']} {ch['date']} {ch['school']}".strip()
            con.execute("UPDATE Student SET status='excluded', exclude_reason=? WHERE id=?", (reason, ids[k]))
            con.execute("INSERT INTO ExclusionLog (student_id, action, reason) VALUES (?, 'exclude', ?)", (ids[k], reason))
            excluded.append((f"{k[0]}학년 {k[1]}반 {k[2]}번 {roster[k]}", reason))
    for (lbl, t) in notes9:
        if '학적 변동' in t:
            for k in ids:
                if k[0] == 1 and f'{k[1]}반 {k[2]}번' == lbl:
                    con.execute("UPDATE Student SET status='excluded', exclude_reason=? WHERE id=? AND status='active'", (t + ' (9월 가채점 표기)', ids[k]))
                    con.execute("INSERT INTO ExclusionLog (student_id, action, reason) VALUES (?, 'exclude', ?)", (ids[k], t))
                    excluded.append((f"1학년 {k[1]}반 {k[2]}번 {roster[k]}", t))
    report['excluded'] = excluded

    # 모의고사
    n_mock = 0
    for r in rows3 + rows6 + rows9:
        sid = ids[(1, r['class_no'], r['number'])]
        con.execute('INSERT OR REPLACE INTO MockScore (student_id, grade_year, grade, round, subject, raw, std, pct, grade9, absent) VALUES (?,?,?,?,?,?,?,?,?,?)',
                    (sid, GRADE_YEAR, 1, r['round'], r['subject'], r['raw'], r['std'], r['pct'], r['grade9'], r['absent']))
        n_mock += 1
    for r in g2mock:
        sid = ids[(r['grade'], r['class_no'], r['number'])]
        con.execute('INSERT OR REPLACE INTO MockScore (student_id, grade_year, grade, round, subject, raw, std, pct, grade9, absent) VALUES (?,?,?,?,?,?,?,?,?,?)',
                    (sid, GRADE_YEAR, r['grade'], r['round'], r['subject'], r['raw'], r['std'], r['pct'], r['grade9'], r['absent']))
        n_mock += 1
    report['mock_rows'] = n_mock

    # 입학 (고1)
    by_name = {}
    for (g, c, n), nm in roster.items():
        if g == 1:
            by_name.setdefault(nm, []).append((c, n))
    entry_rows, ent_unmatched, ent_ambiguous, ent_total = g1.read_entry(by_name)
    for e in entry_rows:
        sid = ids[(1, e['class_no'], e['number'])]
        con.execute('INSERT INTO EntryScore (student_id, mid_school_score, placement_rank, placement_n) VALUES (?,?,?,?)', (sid, e['mid_school_score'], e['placement_rank'], e['placement_n']))
        for sb, v in e['sub'].items():
            if v is not None:
                con.execute('INSERT INTO EntrySubjectScore (student_id, subject, raw) VALUES (?,?,?)', (sid, sb, v))
    report['entry'] = {'total': ent_total, 'matched': len(entry_rows), 'unmatched': ent_unmatched, 'ambiguous': ent_ambiguous}

    # NEIS 내신
    n_school, n_unmatched = 0, set()
    for r in neis_rows:
        k = (r['cur_grade'], r['class_no'], r['number'])
        if k not in ids:
            n_unmatched.add(k)
            continue
        gy = GRADE_YEAR - (r['cur_grade'] - r['grade'])
        con.execute('''INSERT OR REPLACE INTO SchoolScore (student_id, grade_year, grade, semester, subject, achievement, grade5, raw, subject_mean, enrolled_n, credits, area,
                       kind, ach_scale, dist_a, dist_b, dist_c, dist_d, dist_e, note)
                       VALUES (?,?,?,?,?,?,?,?,?,?,?,?,(SELECT kind FROM SubjectCatalog WHERE subject=?),?,?,?,?,?,?,?)''',
                    (ids[k], gy, r['grade'], r['semester'], r['subject'], r['achievement'], r['grade5'], r['raw'], r['subject_mean'], r['enrolled_n'], int(r['credits']), r['area'],
                     r['subject'], r['ach_scale'], *r['dist'], r['note']))
        n_school += 1
    report['school_rows'] = n_school
    report['school_unmatched'] = sorted(n_unmatched)

    # 진학결과 / 강원 자료
    for r in admissions:
        con.execute('''INSERT INTO AdmissionResult (adm_year, class_no, number, region, univ, adm_type, adm_name, detail, track, unit, quota, select_type, min_req, stage1, final, registered,
                       grade_all, grade_main, grade_kem, grade_kor, grade_eng, grade_math, grade_soc, grade_sci, csat_note)
                       VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)''',
                    (r['adm_year'], r['class_no'], r['number'], r['region'], r['univ'], r['adm_type'], r['adm_name'], r['detail'], r['track'], r['unit'], r['quota'], r['select_type'], r['min_req'],
                     r['stage1'], r['final'], r['registered'], r['grade_all'], r['grade_main'], r['grade_kem'], r['grade_kor'], r['grade_eng'], r['grade_math'], r['grade_soc'], r['grade_sci'], r['csat_note']))
    for r in regional:
        con.execute('INSERT INTO RegionalAdmission (source, track, category, univ, adm_name, apply_n, apply_best, apply_worst, pass_n, pass_best, pass_worst, pass_rate, cut70) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)',
                    (r['source'], r['track'], r['category'], r['univ'], r['adm_name'], r['apply_n'], r['apply_best'], r['apply_worst'], r['pass_n'], r['pass_best'], r['pass_worst'], r['pass_rate'], r['cut70']))
    report['admissions'] = {'rows': len(admissions), 'years': sorted({r['adm_year'] for r in admissions})}
    report['regional_rows'] = len(regional)
    report['catalog'] = len(catalog)

    for ds, fn, n in [('students', '명렬표+모의고사 로스터', len(roster)), ('mock', '고1 3·6·9월 + 고2 3·6월', n_mock), ('neis_school', f'download-*.xlsx {len(neis_files)}개', n_school),
                      ('entry', os.path.basename(g1.FILES['entry']), len(entry_rows)), ('admission', '수시 진학결과 2023~2025', len(admissions)), ('regional', '강원 대학별 범위', len(regional))]:
        con.execute("INSERT INTO ImportLog (dataset, file_name, row_count, inserted, mapping_json) VALUES (?,?,?,?,'{\"note\":\"ETL\"}')", (ds, fn, n, n))
    con.commit()
    con.close()
    report['db'] = db_path
    os.makedirs(os.path.join(a.out, '추출데이터'), exist_ok=True)
    with open(os.path.join(a.out, '추출데이터', 'ETL_전체_요약.json'), 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2, default=str)
    print(json.dumps(report, ensure_ascii=False, indent=1, default=str))


if __name__ == '__main__':
    main()

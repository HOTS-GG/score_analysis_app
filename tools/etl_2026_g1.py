# -*- coding: utf-8 -*-
"""
2026학년도 1학년 실제 파일 4종 → 성적분석ON .db 생성 + 정제 CSV + 분석 요약.

입력 (G:\...\2026년\):
  1) 2026 신입생 고입내신+반배치고사 성적.xlsx            → EntryScore / EntrySubjectScore
  2) 2026학년도 고1 3월 학력평가 성적 종합 분석.xlsb ('3월' 시트) → MockScore round=3
  3) 2026학년도 고1 6월 학력평가 성적 종합 분석.xlsb ('6월' 시트) → MockScore round=6
  4) 2026년 9월 학력평가 학급 가채점(1학년 전체반).xlsx     → MockScore round=9 (원점수만, 가채점)

원칙: 미응시는 0이 아니라 결측(absent=1). 실명은 Student 테이블에만. 전화번호·생년월일은 저장하지 않음.
사용: python tools/etl_2026_g1.py [--out 폴더]
"""
import argparse, io, json, math, os, re, sqlite3, sys
from datetime import datetime
import numpy as np
import pandas as pd
from scipy import stats

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

SRC = r"G:/내 드라이브/[2026.03.~2030.02.] 춘천고등학교/2026년/"
DEFAULT_OUT = r"G:/내 드라이브/[2026.03.~2030.02.] 춘천고등학교/2026년/71_춘천고등학교 성적분석/"
SCHEMA = os.path.join(os.path.dirname(__file__), '..', 'src-tauri', 'src', 'schema.sql')
APP_VERSION = '0.1.0'
GRADE_YEAR, GRADE, COHORT = 2026, 1, 2026
CLIP = (0.5, 99.5)

FILES = {
    'entry': SRC + "2026 신입생 고입내신+반배치고사 성적.xlsx",
    'm3': SRC + "2026학년도 고1 3월 학력평가 성적 종합 분석.xlsb",
    'm6': SRC + "2026학년도 고1 6월 학력평가 성적 종합 분석.xlsb",
    'm9': SRC + "2026년 9월 학력평가 학급 가채점(1학년 전체반).xlsx",
}
ABSENT_TOKENS = ('미응시', '결시', '병결', '학적 변동', '도움반')
PENDING_TOKENS = ('취합중', '~')  # 점수 미확정 → 기록 없음

# 동명이인 수동 매칭: (입학 파일 성명, 출신중학교) → (반, 번호). 담임 확인 후 채워 넣고 다시 실행.
MANUAL_ENTRY_MATCH = {
    # ('홍길동', '○○중학교'): (3, 8),
}


def num(v):
    """셀 → float 또는 None. '미응시', '0xf'(수식 오류), 빈칸은 None."""
    if v is None or (isinstance(v, float) and math.isnan(v)):
        return None
    if isinstance(v, (int, float, np.integer, np.floating)):
        return float(v)
    s = str(v).strip().replace(',', '')
    if not s or s.startswith('0x'):
        return None
    try:
        return float(s)
    except ValueError:
        return None


def is_absent(v):
    return isinstance(v, str) and any(t in v for t in ABSENT_TOKENS)


def norm_name(s):
    s = str(s).strip()
    s = re.sub(r'\(.*?\)', '', s)  # '홍길동(일문)' → '홍길동'
    return re.sub(r'\s+', ' ', s).strip()


def parse_hakbun(v):
    s = str(int(float(v))).zfill(5)
    return int(s[0]), int(s[1:3]), int(s[3:5])


# ── 1. 로스터 + 모의고사 ─────────────────────────────────────────

def read_march():
    xl = pd.ExcelFile(FILES['m3'], engine='pyxlsb')
    df = xl.parse('3월', header=None).iloc[4:]
    df = df[df[1].notna()]
    rows, roster = [], {}
    cols = {  # subject: (raw, std, pct, grade)
        '국어': (10, 11, 12, 13), '수학': (15, 16, 17, 18), '영어': (19, None, None, 20),
        '한국사': (7, None, None, 8), '통합사회': (23, 24, 25, 26), '통합과학': (28, 29, 30, 31),
        '제2외국어': (32, 33, 34, 35),
    }
    for _, r in df.iterrows():
        g, c, n = parse_hakbun(r[1])
        name = norm_name(r[6])
        roster[(c, n)] = name
        for subj, (raw, std, pct, grd) in cols.items():
            vals = [num(r[i]) if i is not None else None for i in (raw, std, pct, grd)]
            absent = any(is_absent(r[i]) for i in (raw, std, pct, grd) if i is not None)
            if all(v is None for v in vals) and not absent:
                continue
            rows.append(dict(class_no=c, number=n, round=3, subject=subj, raw=vals[0], std=vals[1], pct=vals[2],
                             grade9=int(vals[3]) if vals[3] is not None else None, absent=int(absent)))
    return roster, rows


def read_june():
    xl = pd.ExcelFile(FILES['m6'], engine='pyxlsb')
    df = xl.parse('6월', header=None).iloc[4:]
    df = df[df[4].notna() & df[5].notna()]
    df = df[df[5].astype(str).str.strip() != '0']
    rows, roster = [], {}
    cols = {'국어': (None, 7, 8, 9), '수학': (None, 10, 11, 12), '영어': (None, None, None, 13), '한국사': (None, None, None, 6),
            '통합사회': (None, 14, 15, 16), '통합과학': (None, 17, 18, 19), '제2외국어': (None, None, None, 20)}
    for _, r in df.iterrows():
        g, c, n = parse_hakbun(r[4])
        roster[(c, n)] = norm_name(r[5])
        whole_absent = is_absent(r[6])
        for subj, (raw, std, pct, grd) in cols.items():
            vals = [num(r[i]) if i is not None else None for i in (raw, std, pct, grd)]
            absent = whole_absent or any(is_absent(r[i]) for i in (std, pct, grd) if i is not None)
            if subj == '제2외국어' and all(v is None for v in vals):
                continue  # 선택 응시 과목: 값이 있을 때만 기록
            if all(v is None for v in vals) and not absent:
                continue
            rows.append(dict(class_no=c, number=n, round=6, subject=subj, raw=vals[0], std=vals[1], pct=vals[2],
                             grade9=int(vals[3]) if vals[3] is not None else None, absent=int(absent)))
    return roster, rows


def read_sept():
    xl = pd.ExcelFile(FILES['m9'], engine='openpyxl')
    rows, roster, notes = [], {}, []
    subj_cols = {'국어': 3, '수학': 4, '영어': 5, '한국사': 6, '통합사회': 7, '통합과학': 8}
    for sh in xl.sheet_names:
        df = xl.parse(sh, header=None).iloc[2:]
        df = df[df[1].notna()]
        for _, r in df.iterrows():
            g, c, n = parse_hakbun(r[1])
            roster[(c, n)] = norm_name(r[2])
            first = r[3]
            row_absent = is_absent(first)
            row_pending = isinstance(first, str) and any(t in first for t in PENDING_TOKENS)
            if isinstance(first, str):
                notes.append((f'{c}반 {n}번', first.strip()))
            for subj, i in subj_cols.items():
                v = r[i]
                x = num(v)
                absent = row_absent or is_absent(v)
                if x is None and not absent:
                    continue  # 취합중·조퇴 이후 과목 등 → 기록 없음
                rows.append(dict(class_no=c, number=n, round=9, subject=subj, raw=x, std=None, pct=None, grade9=None, absent=int(absent)))
            if row_pending:
                pass
    return roster, rows, notes


# ── 2. 입학 데이터 ───────────────────────────────────────────────

def read_entry(roster_by_name):
    xl = pd.ExcelFile(FILES['entry'], engine='openpyxl')
    df = xl.parse(xl.sheet_names[1], header=None).iloc[1:]
    df = df[df[3].notna()]
    n_placement = int(pd.to_numeric(df[9], errors='coerce').notna().sum())
    matched, unmatched, ambiguous = [], [], []
    entry_names = df[3].map(norm_name)
    entry_dups = set(entry_names[entry_names.duplicated(keep=False)])
    for _, r in df.iterrows():
        name = norm_name(r[3])
        school = str(r[7]).strip()
        cands = roster_by_name.get(name, [])
        if (name, school) in MANUAL_ENTRY_MATCH:
            cands = [MANUAL_ENTRY_MATCH[(name, school)]]
        elif name in entry_dups or len(cands) > 1:
            ambiguous.append((name, school, [f'{c}반 {n}번' for c, n in cands]))
            continue
        if not cands:
            unmatched.append((name, school))
            continue
        c, n = cands[0]
        matched.append(dict(class_no=c, number=n, mid_school_score=num(r[8]), placement_rank=num(r[9]),
                            placement_n=n_placement, sub={'국어': num(r[15]), '수학': num(r[16]), '영어': num(r[17])},
                            placement_total=num(r[13])))
    return matched, unmatched, ambiguous, len(df)


# ── 3. DB 쓰기 ───────────────────────────────────────────────────

def build_db(path, students, mock_rows, entry_rows, exclusions, logs):
    if os.path.exists(path):
        os.remove(path)
    con = sqlite3.connect(path)
    con.execute('PRAGMA foreign_keys = ON')
    with open(SCHEMA, encoding='utf-8') as f:
        con.executescript(f.read())
    con.execute('PRAGMA user_version = 2')  # src-tauri/src/db.rs SCHEMA_VERSION 과 맞출 것
    con.execute("INSERT INTO APP_CONFIGS (config_key, config_value) VALUES ('app_version', ?)", (APP_VERSION,))
    ids = {}
    for i, ((c, n), name) in enumerate(sorted(students.items()), start=1):
        con.execute('INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?,?,?,?,?,?)',
                    (f'S{i:05d}', COHORT, GRADE, c, n, name))
        ids[(c, n)] = i
    for r in mock_rows:
        con.execute('''INSERT INTO MockScore (student_id, grade_year, grade, round, subject, raw, std, pct, grade9, absent, blank_n)
                       VALUES (?,?,?,?,?,?,?,?,?,?,NULL)''',
                    (ids[(r['class_no'], r['number'])], GRADE_YEAR, GRADE, r['round'], r['subject'], r['raw'], r['std'], r['pct'], r['grade9'], r['absent']))
    for e in entry_rows:
        sid = ids[(e['class_no'], e['number'])]
        con.execute('INSERT INTO EntryScore (student_id, mid_school_score, placement_rank, placement_n) VALUES (?,?,?,?)',
                    (sid, e['mid_school_score'], e['placement_rank'], e['placement_n']))
        for s, v in e['sub'].items():
            if v is not None:
                con.execute('INSERT INTO EntrySubjectScore (student_id, subject, raw) VALUES (?,?,?)', (sid, s, v))
    for (c, n), reason in exclusions:
        sid = ids[(c, n)]
        con.execute("UPDATE Student SET status='excluded', exclude_reason=? WHERE id=?", (reason, sid))
        con.execute("INSERT INTO ExclusionLog (student_id, action, reason) VALUES (?, 'exclude', ?)", (sid, reason))
    for l in logs:
        con.execute('INSERT INTO ImportLog (dataset, file_name, row_count, inserted, updated, skipped, mapping_json) VALUES (?,?,?,?,?,?,?)',
                    (l['dataset'], l['file'], l['rows'], l['inserted'], 0, l['skipped'], json.dumps(l.get('mapping', {}), ensure_ascii=False)))
    con.commit()
    con.close()


# ── 4. 분석 요약 ─────────────────────────────────────────────────

def z_of(p):
    return stats.norm.ppf(np.clip(p, *CLIP) / 100)


def analyze(students, mock, entry_rows):
    m = pd.DataFrame(mock)
    out = {'rounds': {}, 'delta': {}, 'entry_corr': {}, 'subject_gap': {}}
    for (rd, subj), g in m.groupby(['round', 'subject']):
        d = {'N_rows': int(len(g)), 'absent': int(g.absent.sum())}
        p = g.pct.dropna()
        if len(p) >= 5:
            z = z_of(p.values)
            d.update(N_pct=int(len(p)), mu=round(float(z.mean()), 3), sigma=round(float(z.std(ddof=1)), 3),
                     skew=round(float(stats.skew(z)), 3), pct_median=round(float(p.median()), 1))
        gr = g.grade9.dropna()
        if len(gr):
            d.update(N_grade=int(len(gr)), top12=round(float((gr <= 2).mean() * 100), 1), bottom4=round(float((gr >= 4).mean() * 100), 1))
        rw = g.raw.dropna()
        if len(rw):
            d.update(N_raw=int(len(rw)), raw_mean=round(float(rw.mean()), 1), raw_sd=round(float(rw.std(ddof=1)), 1))
        out['rounds'][f'{rd}월 {subj}'] = d
    # Δ(3→6) 백분위
    for subj in ['국어', '수학']:
        a = m[(m['round'] == 3) & (m.subject == subj)].set_index(['class_no', 'number']).pct
        b = m[(m['round'] == 6) & (m.subject == subj)].set_index(['class_no', 'number']).pct
        d = (b - a).dropna()
        out['delta'][subj] = {'N': int(len(d)), 'mean': round(float(d.mean()), 2), 'sd': round(float(d.std(ddof=1)), 2),
                              'le_-15': int((d <= -15).sum()), 'le_-7': int((d <= -7).sum()), 'ge_+7': int((d >= 7).sum()), 'ge_+15': int((d >= 15).sum())}
    # 과목 불균형 (국어 vs 수학 백분위)
    for rd in [3, 6]:
        a = m[(m['round'] == rd) & (m.subject == '국어')].set_index(['class_no', 'number']).pct
        b = m[(m['round'] == rd) & (m.subject == '수학')].set_index(['class_no', 'number']).pct
        gap = (a - b).dropna().abs()
        out['subject_gap'][f'{rd}월'] = {'N': int(len(gap)), 'mean': round(float(gap.mean()), 1), 'ge_30': int((gap >= 30).sum())}
    # 입학 vs 3월 (부가가치 회귀 가드용 r)
    if entry_rows:
        e = pd.DataFrame([{'class_no': r['class_no'], 'number': r['number'], 'mid': r['mid_school_score'], 'prank': r['placement_rank'], 'pn': r['placement_n']} for r in entry_rows]).set_index(['class_no', 'number'])
        e['x_place'] = 100 - e.prank / e.pn * 100
        e['x_mid'] = 100 - e.mid
        for rd in [3, 6]:
            k = m[(m['round'] == rd) & (m.subject.isin(['국어', '수학']))].pivot_table(index=['class_no', 'number'], columns='subject', values='pct')
            if k.empty:
                continue
            y = z_of(k.mean(axis=1).dropna().values)
            yy = pd.Series(y, index=k.mean(axis=1).dropna().index)
            j = e.join(yy.rename('z'), how='inner').dropna(subset=['z'])
            res = {}
            for xc in ['x_place', 'x_mid']:
                jj = j.dropna(subset=[xc])
                if len(jj) > 10:
                    r_, p_ = stats.pearsonr(jj[xc], jj.z)
                    res[xc] = {'N': int(len(jj)), 'r': round(float(r_), 3), 'r2': round(float(r_ ** 2), 3)}
            out['entry_corr'][f'{rd}월'] = res
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--out', default=DEFAULT_OUT)
    a = ap.parse_args()
    os.makedirs(os.path.join(a.out, '추출데이터'), exist_ok=True)

    roster3, rows3 = read_march()
    roster6, rows6 = read_june()
    roster9, rows9, notes9 = read_sept()
    students = dict(roster3)
    for r in (roster6, roster9):
        for k, v in r.items():
            students.setdefault(k, v)
    name_conflicts = [(k, roster3.get(k), roster6.get(k), roster9.get(k)) for k in students
                      if len({x for x in (roster3.get(k), roster6.get(k), roster9.get(k)) if x}) > 1]

    by_name = {}
    for k, v in students.items():
        by_name.setdefault(v, []).append(k)
    entry_rows, ent_unmatched, ent_ambiguous, ent_total = read_entry(by_name)

    exclusions = [(k, f'{t} (9월 가채점 표기)') for (lbl, t) in notes9 if '학적 변동' in t
                  for k in students if f'{k[0]}반 {k[1]}번' == lbl]

    mock = rows3 + rows6 + rows9
    logs = [
        {'dataset': 'students', 'file': '3월·6월·9월 파일 로스터 합성', 'rows': len(students), 'inserted': len(students), 'skipped': 0},
        {'dataset': 'mock', 'file': os.path.basename(FILES['m3']) + ' [3월]', 'rows': len(roster3), 'inserted': len(rows3), 'skipped': 0, 'mapping': {'context': {'grade_year': GRADE_YEAR, 'grade': GRADE, 'round': 3}, 'note': 'ETL'}},
        {'dataset': 'mock', 'file': os.path.basename(FILES['m6']) + ' [6월]', 'rows': len(roster6), 'inserted': len(rows6), 'skipped': 0, 'mapping': {'context': {'grade_year': GRADE_YEAR, 'grade': GRADE, 'round': 6}, 'note': 'ETL'}},
        {'dataset': 'mock', 'file': os.path.basename(FILES['m9']), 'rows': len(roster9), 'inserted': len(rows9), 'skipped': 0, 'mapping': {'context': {'grade_year': GRADE_YEAR, 'grade': GRADE, 'round': 9}, 'note': 'ETL 가채점 원점수만'}},
        {'dataset': 'entry', 'file': os.path.basename(FILES['entry']), 'rows': ent_total, 'inserted': len(entry_rows), 'skipped': len(ent_unmatched) + len(ent_ambiguous), 'mapping': {'note': 'ETL 이름 매칭'}},
    ]
    db_path = os.path.join(a.out, '2026_1학년_성적분석.db')
    build_db(db_path, students, mock, entry_rows, exclusions, logs)

    # CSV (앱 가져오기 화면으로 재현 가능한 세로형)
    pd.DataFrame([{'학년': GRADE, '반': c, '번호': n, '이름': nm} for (c, n), nm in sorted(students.items())]).to_csv(os.path.join(a.out, '추출데이터', '학생명단.csv'), index=False, encoding='utf-8-sig')
    mdf = pd.DataFrame(mock).rename(columns={'class_no': '반', 'number': '번호', 'round': '회차', 'subject': '과목', 'raw': '원점수', 'std': '표준점수', 'pct': '백분위', 'grade9': '등급', 'absent': '결시'})
    for rd in [3, 6, 9]:
        mdf[mdf['회차'] == rd].drop(columns=['회차']).to_csv(os.path.join(a.out, '추출데이터', f'모의고사_{rd}월_세로형.csv'), index=False, encoding='utf-8-sig')
    pd.DataFrame([{'반': e['class_no'], '번호': e['number'], '고입내신 석차백분율': e['mid_school_score'], '반배치 석차': e['placement_rank'], '반배치 응시인원': e['placement_n'],
                   '반배치 국어': e['sub']['국어'], '반배치 수학': e['sub']['수학'], '반배치 영어': e['sub']['영어'], '반배치 총점': e['placement_total']} for e in entry_rows]).to_csv(os.path.join(a.out, '추출데이터', '입학시점.csv'), index=False, encoding='utf-8-sig')

    an = analyze(students, mock, entry_rows)
    report = {
        'generated': datetime.now().isoformat(timespec='seconds'), 'db': db_path,
        'students': len(students), 'name_conflicts': name_conflicts, 'excluded': [(f'{k[0]}반 {k[1]}번 {students[k]}', r) for k, r in exclusions],
        'mock_rows': {'3월': len(rows3), '6월': len(rows6), '9월': len(rows9)},
        'sept_notes': notes9,
        'entry': {'total': ent_total, 'matched': len(entry_rows), 'unmatched': ent_unmatched, 'ambiguous': ent_ambiguous},
        'analysis': an,
    }
    with open(os.path.join(a.out, '추출데이터', '분석요약.json'), 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2, default=str)
    write_markdown(report, os.path.join(a.out, '분석요약_2026_1학년.md'))
    print(json.dumps(report, ensure_ascii=False, indent=1, default=str))


def write_markdown(rep, path):
    an = rep['analysis']
    L = []
    L.append(f"# 2026학년도 1학년 성적 데이터 추출·분석 요약\n\n생성: {rep['generated']}  \n앱 파일: `{rep['db']}`\n")
    L.append("## 1. 추출 결과\n")
    L.append(f"- 학생 명단: **{rep['students']}명** (3월·6월·9월 파일의 학번·이름 합성, 이름 충돌 {len(rep['name_conflicts'])}건)")
    for lbl, r in rep['excluded']:
        L.append(f"- 제외 처리: {lbl} — {r}")
    L.append(f"- 모의고사 행: 3월 {rep['mock_rows']['3월']} / 6월 {rep['mock_rows']['6월']} / 9월 {rep['mock_rows']['9월']} (학생×과목)")
    e = rep['entry']
    L.append(f"- 입학 시점: 파일 {e['total']}명 중 **{e['matched']}명 매칭**, 명단에 없음 {len(e['unmatched'])}명, 동명이인 보류 {len(e['ambiguous'])}명\n")
    L.append("### 파일별 특징\n")
    L.append("| 파일 | 들어온 것 | 못 들어온 것 |\n|---|---|---|")
    L.append("| 3월 종합분석(xlsb) | 국어·수학 표준점수/백분위/등급, 영어·한국사·통합사회·통합과학 등급 | 원점수 없음, 통사·통과 백분위 없음 |")
    L.append("| 6월 종합분석(xlsb) | 국어·수학·통합사회·통합과학 표준점수/백분위/등급, 영어·한국사 등급 | 원점수 없음 |")
    L.append("| 9월 학급 가채점(xlsx) | 6과목 원점수 (한국사·통사·통과는 50점 만점) | 표준점수·백분위·등급 없음 → 회차 비교 불가, 정식 성적표 나오면 교체 |")
    L.append("| 신입생 고입내신+반배치(xlsx) | 고입내신 석차백분율, 반배치 석차·응시인원, 국·수·영 원점수 | 전화번호·생년월일은 저장하지 않음. 임시학번은 실제 학번과 달라 이름으로 매칭 |\n")
    L.append("### 9월 가채점 특이 표기\n")
    for lbl, t in rep['sept_notes']:
        L.append(f"- {lbl}: {t}")
    L.append("\n### 입학 데이터 매칭 보류 (담임 확인 필요)\n")
    L.append("동명이인은 자동으로 붙이지 않았습니다. `tools/etl_2026_g1.py`의 `MANUAL_ENTRY_MATCH`에 (성명, 출신중) → (반, 번호)를 넣고 다시 실행하면 됩니다.\n")
    L.append("| 성명 | 출신중학교 | 명단의 후보 |\n|---|---|---|")
    for item in e['ambiguous']:
        name, school, cands = (item + [[]])[:3] if len(item) == 2 else item
        L.append(f"| {name} | {school} | {', '.join(cands) if cands else '-'} |")
    for name, school in e['unmatched']:
        L.append(f"| {name} | {school} | 명단에 없음 (전출·미등록?) |")
    L.append("\n## 2. 학교 레벨 (L0) — 모의고사 전국 백분위만 사용\n")
    L.append("z = Φ⁻¹(백분위/100), 백분위 0.5~99.5 클리핑. mu = 학년의 전국 레벨(0 = 전국 평균), sigma = 밀집도(1 = 전국과 같은 퍼짐).\n")
    L.append("| 회차·과목 | N | mu | sigma | 왜도 | 백분위 중앙값 | 1~2등급 % | 4등급↓ % |\n|---|---|---|---|---|---|---|---|")
    for k, d in an['rounds'].items():
        if 'mu' in d:
            L.append(f"| {k} | {d['N_pct']} | {d['mu']:+.3f} | {d['sigma']:.3f} | {d['skew']:+.2f} | {d['pct_median']} | {d.get('top12','')} | {d.get('bottom4','')} |")
    L.append("\n등급만 있는 과목 (표시용):\n")
    L.append("| 회차·과목 | N | 1~2등급 % | 4등급↓ % |\n|---|---|---|---|")
    for k, d in an['rounds'].items():
        if 'mu' not in d and 'top12' in d:
            L.append(f"| {k} | {d['N_grade']} | {d['top12']} | {d['bottom4']} |")
    L.append("\n9월 가채점 원점수 (회차 간 비교 금지 — 난이도가 다름):\n")
    L.append("| 과목 | N | 평균 | 표준편차 |\n|---|---|---|---|")
    for k, d in an['rounds'].items():
        if k.startswith('9월') and 'raw_mean' in d:
            L.append(f"| {k[3:]} | {d['N_raw']} | {d['raw_mean']} | {d['raw_sd']} |")
    L.append("\n## 3. 개인 변화 Δ(3→6) — 전국 백분위 차\n")
    L.append("| 과목 | N | 평균 Δ | SD | Δ ≤ −15 (급락) | Δ ≤ −7 | Δ ≥ +7 | Δ ≥ +15 |\n|---|---|---|---|---|---|---|---|")
    for s, d in an['delta'].items():
        L.append(f"| {s} | {d['N']} | {d['mean']:+.2f} | {d['sd']} | {d['le_-15']} | {d['le_-7']} | {d['ge_+7']} | {d['ge_+15']} |")
    L.append("\n두 회차뿐이라 '연속 하락'(3→6, 6→9 모두 ≤ −7) 판정은 9월 정식 백분위가 들어와야 가능합니다.\n")
    L.append("## 4. 과목 불균형 (|국어 − 수학| 백분위)\n")
    L.append("| 회차 | N | 평균 격차 | 격차 ≥ 30 (플래그) |\n|---|---|---|---|")
    for k, d in an['subject_gap'].items():
        L.append(f"| {k} | {d['N']} | {d['mean']} | {d['ge_30']}명 |")
    L.append("\n## 5. 입학 시점 ↔ 모의고사 상관 (부가가치 회귀 가드)\n")
    L.append("x = 100 − 석차백분율(반배치 / 고입내신), y = 국·수 백분위 평균의 z. 회귀는 r ≥ 0.4, N ≥ 100일 때만 허용.\n")
    L.append("| 회차 | 반배치 r (N) | r² | 고입내신 r (N) | r² |\n|---|---|---|---|---|")
    for k, d in an['entry_corr'].items():
        p, m = d.get('x_place', {}), d.get('x_mid', {})
        L.append(f"| {k} | {p.get('r','-')} ({p.get('N','-')}) | {p.get('r2','-')} | {m.get('r','-')} ({m.get('N','-')}) | {m.get('r2','-')} |")
    L.append("\n## 6. 읽을 때 주의\n")
    L.append("- 이 요약은 집계일 뿐, 인과를 말하지 않습니다. mu·sigma의 변화는 '학년 전체가 전국 대비 어디에 있는가'입니다.")
    L.append("- 개인의 전국 위치는 3월·6월 백분위로만 봅니다. 9월은 가채점 원점수라 개인 추이에 쓰지 않습니다.")
    L.append("- 잔차·회귀 예측값은 교사용 스크리닝 지표이며 학생 카드에 나오지 않습니다.")
    with open(path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(L) + '\n')


if __name__ == '__main__':
    main()

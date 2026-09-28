# -*- coding: utf-8 -*-
"""
성적분석ON 가져오기 양식 생성.
  자동 인식 양식: 01_나이스_교과학습발달상황(반별).xlsx, 02_학력평가_종합분석(회차시트).xlsx, 03_학력평가_가채점(반별).xlsx
  직접 매핑 양식: 04_학생명단.xlsx, 05_입학시점_고입내신_반배치.xlsx, 06_담임용_내신입력.xlsx
각 파일에 '작성요령' 시트를 넣는다. 함께 templates/*.json(셀 격자)도 만들어 Rust 파서 테스트에 쓴다.
사용: python tools/make_templates.py [--out 폴더]
"""
import argparse, io, json, os, sys
from openpyxl import Workbook
from openpyxl.styles import Alignment, Border, Font, PatternFill, Side
from openpyxl.utils import get_column_letter

if not isinstance(sys.stdout, io.TextIOWrapper) or sys.stdout.encoding.lower() != 'utf-8':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_OUT = r"G:/내 드라이브/[2026.03.~2030.02.] 춘천고등학교/2026년/71_춘천고등학교 성적분석/양식/"
JSON_OUT = os.path.join(HERE, '..', 'templates')

HEAD_FILL = PatternFill('solid', fgColor='DDEBF7')
NOTE_FILL = PatternFill('solid', fgColor='FFF2CC')
BOLD = Font(bold=True)
THIN = Side(style='thin', color='BBBBBB')
BORDER = Border(left=THIN, right=THIN, top=THIN, bottom=THIN)


def style_header(ws, row, ncol):
    for c in range(1, ncol + 1):
        cell = ws.cell(row=row, column=c)
        cell.font = BOLD
        cell.fill = HEAD_FILL
        cell.alignment = Alignment(horizontal='center', vertical='center', wrap_text=True)
        cell.border = BORDER


def autosize(ws, widths=None):
    for i, col in enumerate(ws.columns, start=1):
        w = (widths or {}).get(i)
        if w is None:
            w = max((len(str(c.value)) if c.value is not None else 0) for c in col)
            w = min(40, max(8, int(w * 1.6)))
        ws.column_dimensions[get_column_letter(i)].width = w


def guide_sheet(wb, lines, title='작성요령'):
    ws = wb.create_sheet(title)
    ws.column_dimensions['A'].width = 110
    for i, line in enumerate(lines, start=1):
        c = ws.cell(row=i, column=1, value=line)
        c.alignment = Alignment(wrap_text=True, vertical='top')
        if line.startswith('■'):
            c.font = BOLD
            c.fill = NOTE_FILL
    return ws


def grid_of(ws):
    out = []
    for row in ws.iter_rows(values_only=True):
        out.append([None if v is None else v for v in row])
    return out


# ── 01 나이스 교과학습발달상황 ────────────────────────────────

def make_neis(out):
    wb = Workbook()
    ws = wb.active
    ws.title = 'sheet1'
    ws['R1'] = '2026.09.07.'
    ws['E2'] = '학교생활기록부 교과학습발달상황'
    ws['E2'].font = Font(bold=True, size=13)
    ws['A3'] = '2학년 1반'
    ws['A3'].font = BOLD
    ws['R3'] = '사용자명 : 홍길동'
    H5 = ['번 호', '성 명', '학 년', '학 기', '교 과', '과 목', '학 점', '원점수/과목평균', '성취도', '성취도별 분포비율', '석차등급', '수강자수', '비 고']
    H3 = ['번 호', '성 명', '학 년', '학 기', '교 과', '과 목', '학 점', '성취도', '비 고']
    dist = 'A(8.5) B(22.0) C(30.2)D(24.1) E(15.2)'
    students = [(1, '김서준'), (2, '이하윤'), (3, '박지호')]
    five = [  # (학년, 학기, 교과, 과목, 학점, 원점수, 평균, 성취도, 석차등급, 수강자)
        (1, 1, '국어', '공통국어1', 4, 82, 63.8, 'B', 2, 265), (1, 1, '수학', '공통수학1', 4, 71, 61.0, 'C', 3, 265),
        (1, 1, '영어', '공통영어1', 4, 90, 60.8, 'A', 1, 265), (1, 1, '사회(역사/도덕포함)', '통합사회1', 4, 77, 70.7, 'B', 2, 265),
        (1, 1, '과학', '통합과학1', 4, 66, 68.0, 'C', 3, 265),
        (1, 2, '국어', '공통국어2', 4, 79, 63.2, 'B', 2, 259), (1, 2, '수학', '공통수학2', 4, 68, 59.7, 'C', 3, 259),
        (2, 1, '국어', '문학', 4, 86, 67.2, 'B', 2, 258), (2, 1, '수학', '대수', 4, 87, 54.9, 'A', 1, 258),
        (2, 1, '영어', '영어Ⅰ', 4, 94, 50.7, 'A', 1, 258), (2, 1, '사회(역사/도덕포함)', '사회문제 탐구', 3, 100, 97.3, 'A', '·', 80),
        (2, 1, '기술·가정/정보', '정보', 3, 90, 70.4, 'A', 1, 258),
    ]
    three = [(1, 1, '과학', '과학탐구실험1', 1, 'B'), (1, 1, '체육', '체육1', 2, 'A'), (1, 1, '예술', '음악', 3, 'A'), (2, 1, '체육', '스포츠 생활1', 2, 'A')]
    r = 4
    ws.append([])
    # 5단계 블록: 학생마다 머리글 반복 (나이스 출력물과 같은 모양)
    for si, (num, name) in enumerate(students):
        for j, h in enumerate(H5, start=1):
            ws.cell(row=r, column=j, value=h)
        style_header(ws, r, len(H5))
        r += 1
        last_g, last_s = None, None
        for k, (g, sem, area, subj, cred, raw, mean, ach, rank, n) in enumerate(five):
            raw2 = raw - si * 6 if raw - si * 6 > 0 else raw
            ws.cell(row=r, column=1, value=num if k == 0 else None)
            ws.cell(row=r, column=2, value=name if k == 0 else None)
            ws.cell(row=r, column=3, value=g if (k == 0 or g != last_g) else None)
            ws.cell(row=r, column=4, value=sem if (k == 0 or g != last_g or sem != last_s) else None)
            ws.cell(row=r, column=5, value=area)
            ws.cell(row=r, column=6, value=subj)
            ws.cell(row=r, column=7, value=cred)
            ws.cell(row=r, column=8, value=f'{raw2}/{mean}')
            ws.cell(row=r, column=9, value=ach)
            ws.cell(row=r, column=10, value=dist if ach != 'A' or rank != '·' else 'A(93.8) B(5.0) C(0.0)D(1.3) E(0.0)')
            ws.cell(row=r, column=11, value=rank)
            ws.cell(row=r, column=12, value=n)
            last_g, last_s = g, sem
            r += 1
        ws.cell(row=r, column=1, value='이수학점 합계')
        ws.cell(row=r, column=7, value=sum(x[4] for x in five))
        r += 1
    # 3단계 블록
    for j, h in enumerate(H3, start=1):
        ws.cell(row=r, column=j, value=h)
    style_header(ws, r, len(H3))
    r += 1
    for num, name in students:
        last_g, last_s = None, None
        for k, (g, sem, area, subj, cred, ach) in enumerate(three):
            ws.cell(row=r, column=1, value=num if k == 0 else None)
            ws.cell(row=r, column=2, value=name if k == 0 else None)
            ws.cell(row=r, column=3, value=g if (k == 0 or g != last_g) else None)
            ws.cell(row=r, column=4, value=sem if (k == 0 or g != last_g or sem != last_s) else None)
            ws.cell(row=r, column=5, value=area)
            ws.cell(row=r, column=6, value=subj)
            ws.cell(row=r, column=7, value=cred)
            ws.cell(row=r, column=8, value=ach)
            last_g, last_s = g, sem
            r += 1
    autosize(ws, {5: 20, 6: 18, 8: 16, 10: 40})
    guide_sheet(wb, [
        '■ 나이스 교과학습발달상황 양식 (데이터 → 나이스 내신 가져오기)',
        '이 시트는 나이스에서 [학교생활기록부 > 교과학습발달상황]을 반별로 엑셀 저장한 파일과 같은 모양입니다. 실제 파일은 그대로 넣으면 되고, 이 양식은 손으로 채울 때 참고용입니다.',
        '',
        '■ 프로그램이 읽는 규칙',
        '1. 위쪽 어딘가에 "2학년 1반"처럼 「N학년 M반」이 적힌 칸이 있어야 반을 알 수 있습니다.',
        '2. "번 호 / 성 명 / 학 년 / 학 기 / 교 과 / 과 목 / 학 점 / 원점수/과목평균 / 성취도 / 성취도별 분포비율 / 석차등급 / 수강자수 / 비 고" 머리글 행 아래가 성적입니다. 머리글은 여러 번 반복돼도 됩니다.',
        '3. 학생의 첫 줄에만 번호·성명을 적고, 이어지는 과목 줄은 비워 둡니다. 학년·학기도 바뀔 때만 적습니다.',
        '4. 원점수/과목평균은 "82/63.8" 처럼 슬래시로 붙여 씁니다. 분포비율은 "A(8.5) B(22.0) C(30.2)D(24.1) E(15.2)" 형식.',
        '5. 석차등급이 없는 과목(성취도만 있는 과목)은 석차등급 칸에 "·" 을 둡니다.',
        '6. 체육·예술·과학탐구실험처럼 3단계(A·B·C) 과목은 "번 호 / 성 명 / 학 년 / 학 기 / 교 과 / 과 목 / 학 점 / 성취도 / 비 고" 머리글의 별도 블록에 적습니다.',
        '7. 2학년 파일에 들어 있는 1학년 성적은 자동으로 한 해 전 학년도로 저장됩니다.',
        '',
        '■ 주의',
        '· 이름은 명단 매칭에만 쓰이고 분석 표에는 저장되지 않습니다.',
        '· 미이수·미응시 과목은 줄을 넣지 않거나 값을 비워 두세요. 0점을 쓰지 마세요.',
    ])
    path = os.path.join(out, '01_나이스_교과학습발달상황(반별).xlsx')
    wb.save(path)
    return path, grid_of(ws)


# ── 02 학력평가 종합분석 회차 시트 ─────────────────────────────

def make_analysis(out):
    wb = Workbook()
    ws = wb.active
    ws.title = '6월'
    ws['C2'] = '2026학년도'
    ws['F2'] = '6월 학력평가'
    ws['F2'].font = BOLD
    H = ['', '학년', '반', '번호', '학번', '이름', '한국사\n등급', '국어\n표준점수', '국어\n백분위', '국어\n등급', '수학\n표준점수', '수학\n백분위', '수학\n등급', '영어\n등급',
         '통합사회\n표준점수', '통사\n백분위', '통사\n등급', '통합과학\n표준점수', '통과\n백분위', '통과\n등급', '제2외국어/한문 등급', '국영수\n등급평균', '국영수탐\n등급평균']
    ws.append([])
    ws.append([])
    for j, h in enumerate(H, start=1):
        ws.cell(row=4, column=j, value=h)
    style_header(ws, 4, len(H))
    ws.row_dimensions[4].height = 34
    rows = [
        (1, '01', 1, 10101, '김서준', 1, 123, 85.83, 3, 104, 57.73, 5, 3, 56, 65.47, 4, 60, 79.82, 3, None, 3.7, 3.8),
        (1, '01', 2, 10102, '이하윤', 2, 108, 63.49, 4, 128, 90.5, 2, 3, 60, 84.16, 3, 67, 96.11, 1, None, 3.0, 2.8),
        (1, '01', 3, 10103, '박지호', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', '미응시', None, None, None),
    ]
    for i, row in enumerate(rows, start=5):
        for j, v in enumerate(row, start=2):
            ws.cell(row=i, column=j, value=v)
    autosize(ws, {6: 10, 21: 16})
    guide_sheet(wb, [
        '■ 학력평가 성적 종합 분석 양식 (데이터 → 모의고사 성적 가져오기)',
        '교육청 학력평가 종합분석 파일의 회차 시트(3월·6월…)와 같은 모양입니다. 실제 파일이 .xlsb 이면 엑셀에서 "다른 이름으로 저장 → Excel 통합 문서(*.xlsx)"로 저장한 뒤 그대로 넣으면 됩니다.',
        '',
        '■ 프로그램이 읽는 규칙',
        '1. "학번"과 "이름"이 있는 머리글 행을 찾습니다. 그 행에서 "국어 표준점수", "국어 백분위", "국어 등급" 처럼 「과목 + 항목」으로 된 열을 과목별로 읽습니다. (줄바꿈·공백 상관없음)',
        '2. 학번은 5자리(학년1 + 반2 + 번호2, 예 10103 = 1학년 1반 3번)여야 합니다. 이름이 비었거나 0인 줄은 건너뜁니다.',
        '3. "통사·통과"는 통합사회·통합과학으로, "제2외국어/한문 등급"은 제2외국어로 저장됩니다.',
        '4. 등급평균·순위·석차·괄호가 들어간 집계 열(국영수 등급평균, 국수탐(표준) 등)은 읽지 않습니다.',
        '5. 회차(월)는 시트 이름이나 위쪽 제목("6월 학력평가")에서 읽고, 화면에서 바꿀 수 있습니다. 학년도는 화면에서 지정합니다.',
        '6. 미응시 학생은 칸에 "미응시"라고 쓰거나 비워 두세요. 0점을 쓰면 안 됩니다.',
        '',
        '■ 이 양식으로 넣으면 학생 화면의 "내신 vs 모의고사", 학년·학교 화면의 전국 백분위 분석이 채워집니다.',
    ])
    path = os.path.join(out, '02_학력평가_종합분석(회차시트).xlsx')
    wb.save(path)
    return path, grid_of(ws)


# ── 03 가채점 ─────────────────────────────────────────────────

def make_raw(out):
    wb = Workbook()
    ws = wb.active
    ws.title = '1-1'
    H = ['', '학번', '이름', '국어', '수학', '영어', '한국사', '통사', '통과', '합계']
    ws.append([])
    for j, h in enumerate(H, start=1):
        ws.cell(row=2, column=j, value=h)
    style_header(ws, 2, len(H))
    rows = [(10101, '김서준', 72, 61, 80, 40, 31.5, 47), (10102, '이하윤', 59, 34, 67, 32, 42, 32), (10103, '박지호', '미응시', None, None, None, None, None)]
    for i, (hk, nm, *sc) in enumerate(rows, start=3):
        ws.cell(row=i, column=2, value=hk)
        ws.cell(row=i, column=3, value=nm)
        for j, v in enumerate(sc, start=4):
            ws.cell(row=i, column=j, value=v)
        if all(isinstance(v, (int, float)) for v in sc):
            ws.cell(row=i, column=10, value=f'=SUM(D{i}:I{i})')
    autosize(ws)
    ws2 = wb.copy_worksheet(ws)
    ws2.title = '1-2'
    ws2['B3'], ws2['C3'] = 10201, '정도윤'
    ws2['B4'], ws2['C4'] = 10202, '강지우'
    ws2['B5'], ws2['C5'] = 10203, '조예린'
    guide_sheet(wb, [
        '■ 학력평가 학급 가채점 양식 (데이터 → 모의고사 성적 가져오기)',
        '반마다 시트 하나("1-1", "1-2" …). 정식 성적표가 나오기 전 원점수만 모아 둘 때 씁니다.',
        '',
        '■ 프로그램이 읽는 규칙',
        '1. "학번 / 이름 / 국어 / 수학 / 영어 / 한국사 / 통사 / 통과 / 합계" 머리글. 합계는 있어도 없어도 됩니다.',
        '2. 학번은 5자리. 회차(월)는 화면에서 고릅니다(가채점은 보통 9월).',
        '3. 미응시·도움반·결시는 국어 칸에 "미응시" 처럼 적으면 전 과목 결측으로 처리됩니다. "취합중"이라 적힌 학생은 건너뜁니다.',
        '4. 원점수만 있으므로 회차 간 비교에는 쓰지 않습니다. 정식 성적표(02 양식)를 같은 회차로 다시 넣으면 백분위·등급이 채워집니다.',
    ])
    path = os.path.join(out, '03_학력평가_가채점(반별).xlsx')
    wb.save(path)
    return path, grid_of(ws)


# ── 04 학생 명단 ──────────────────────────────────────────────

def make_roster(out):
    wb = Workbook()
    ws = wb.active
    ws.title = '명단'
    H = ['학년', '반', '번호', '이름']
    ws.append(H)
    style_header(ws, 1, len(H))
    for row in [(1, 1, 1, '김서준'), (1, 1, 2, '이하윤'), (1, 1, 3, '박지호'), (1, 2, 1, '정도윤')]:
        ws.append(row)
    autosize(ws)
    guide_sheet(wb, [
        '■ 학생 명단 양식 (데이터 → 모의고사 성적 가져오기 → 직접 매핑 → "학생 명단", 또는 학생 명단 탭의 엑셀 가져오기)',
        '학년·반·번호·이름 네 열이면 됩니다. 입학연도는 화면의 공통값에서 한 번만 적습니다 (예: 2026학년도 1학년 → 2026).',
        '',
        '보통은 이 파일이 필요 없습니다. 나이스 내신 파일이나 모의고사 파일을 넣을 때 "명단에 없는 학생 자동 생성"이 켜져 있으면 명단이 자동으로 만들어집니다.',
        '전학·자퇴 학생은 삭제하지 말고 학생 명단 탭에서 "제외"하세요. 통계에서 빠지고 기록은 남습니다.',
    ])
    path = os.path.join(out, '04_학생명단.xlsx')
    wb.save(path)
    return path, grid_of(ws)


# ── 05 입학 시점 ─────────────────────────────────────────────

def make_entry(out):
    wb = Workbook()
    ws = wb.active
    ws.title = '입학'
    H = ['학년', '반', '번호', '이름', '고입내신 석차백분율', '반배치 석차', '반배치 응시인원', '반배치 국어', '반배치 수학', '반배치 영어']
    ws.append(H)
    style_header(ws, 1, len(H))
    for row in [(1, 1, 1, '김서준', 12.3, 24, 262, 88, 100, 96), (1, 1, 2, '이하윤', 35.3, 69, 262, 80, 60, 96), (1, 1, 3, '박지호', 57.4, 150, 262, 56, 44, 40)]:
        ws.append(row)
    autosize(ws, {5: 18, 6: 12, 7: 14})
    guide_sheet(wb, [
        '■ 입학 시점 양식 (데이터 → 모의고사 성적 가져오기 → "직접 매핑" 펼치기 → 데이터셋 "입학 시점")',
        '입학 당시 위치를 기록해 두면 학급·학생 화면에 "입학 때보다 올라옴/내려감"이 표시됩니다.',
        '',
        '■ 열 설명',
        '· 고입내신 석차백분율: 중학교 내신 석차 백분율(%). 작을수록 상위.',
        '· 반배치 석차 / 반배치 응시인원: 반배치고사 교내 석차와 응시 인원. 위치 계산은 이 둘을 우선 씁니다.',
        '· 반배치 국어·수학·영어: 과목별 원점수 (선택).',
        '',
        '직접 매핑 화면에서 파일을 열면 열 이름을 보고 자동으로 짝을 맞춥니다. 확인만 하고 가져오기를 누르면 됩니다. 학년은 공통값에 적습니다.',
        '신입생 파일의 임시학번은 실제 학번과 다를 수 있으니, 반·번호는 배정 후 실제 값으로 적어 주세요.',
    ])
    path = os.path.join(out, '05_입학시점_고입내신_반배치.xlsx')
    wb.save(path)
    return path, grid_of(ws)


# ── 06 담임용 내신 (표준편차·석차 포함) ──────────────────────

def make_homeroom(out):
    wb = Workbook()
    ws = wb.active
    ws.title = '내신입력'
    H = ['학년', '반', '번호', '이름', '과목', '단위수', '원점수', '성취도', '5등급', '과목평균', '표준편차', '수강자수', '석차']
    ws.append(H)
    style_header(ws, 1, len(H))
    for row in [(2, 5, 18, '김서준', '문학', 4, 86.77, 'B', 2, 67.2, 17.8, 258, 34), (2, 5, 18, '김서준', '대수', 4, 87.19, 'A', 1, 54.9, 23.4, 258, 10),
                (2, 5, 18, '김서준', '사회문제 탐구', 3, 100, 'A', None, 97.3, 5.7, 80, 1), (2, 5, 19, '이하윤', '문학', 4, 71.5, 'C', 3, 67.2, 17.8, 258, 110)]:
        ws.append(row)
    autosize(ws, {4: 10, 5: 16})
    guide_sheet(wb, [
        '■ 담임용 내신 입력 양식 (데이터 → 모의고사 성적 가져오기 → "직접 매핑" 펼치기 → 데이터셋 "내신")',
        '나이스 출력물에는 없는 표준편차와 석차를 넣을 때 씁니다. 이 두 값이 있으면 학생 화면에 "2등급 내 8/62 위치", "3.2점 올리면 1등급 진입권" 같은 세부 위치가 나옵니다.',
        '',
        '■ 열 설명',
        '· 한 줄 = 학생 1명 × 과목 1개. 과목 수만큼 줄을 추가합니다.',
        '· 성취도만 있는 과목(A·B·C 과목, 이수 과목)은 5등급 칸을 비웁니다. 등급 평균에서 자동 제외됩니다.',
        '· 석차가 없으면 비워도 됩니다. 과목평균·표준편차로 위치를 추정하고 "추정" 표시가 붙습니다.',
        '· 과목명 뒤에 (4) 처럼 단위수를 붙여도 인식합니다.',
        '· 학년도·학기는 화면의 공통값에 적습니다.',
        '',
        '이 양식은 앱의 직접 매핑 화면 → 내신 탭 → "담임용 입력 양식 내려받기"로도 받을 수 있으며, 그때는 반 학생이 미리 채워집니다.',
    ])
    path = os.path.join(out, '06_담임용_내신입력.xlsx')
    wb.save(path)
    return path, grid_of(ws)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--out', default=DEFAULT_OUT)
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    os.makedirs(JSON_OUT, exist_ok=True)
    made = []
    for fn, key in [(make_neis, 'neis'), (make_analysis, 'mock_analysis'), (make_raw, 'mock_raw'), (make_roster, 'roster'), (make_entry, 'entry'), (make_homeroom, 'homeroom')]:
        path, grid = fn(a.out)
        with open(os.path.join(JSON_OUT, f'{key}.json'), 'w', encoding='utf-8') as f:
            json.dump(grid, f, ensure_ascii=False)
        made.append(path)
    # 안내 README
    readme = os.path.join(a.out, '읽어주세요.txt')
    with open(readme, 'w', encoding='utf-8') as f:
        f.write('성적분석ON 가져오기 양식\n\n'
                '[자동 인식 — 파일을 그대로 넣으면 됩니다]\n'
                ' 01_나이스_교과학습발달상황(반별).xlsx  → 데이터 › 나이스 내신 가져오기\n'
                ' 02_학력평가_종합분석(회차시트).xlsx     → 데이터 › 모의고사 성적 가져오기  (실제 파일이 .xlsb면 .xlsx로 저장)\n'
                ' 03_학력평가_가채점(반별).xlsx          → 데이터 › 모의고사 성적 가져오기\n\n'
                '[직접 매핑 — 열 이름을 보고 자동으로 짝을 맞춥니다]\n'
                ' 04_학생명단.xlsx                       → 학생 명단 (보통은 자동 생성되므로 불필요)\n'
                ' 05_입학시점_고입내신_반배치.xlsx        → 모의고사 성적 가져오기 › 직접 매핑 › 입학 시점\n'
                ' 06_담임용_내신입력.xlsx                 → 모의고사 성적 가져오기 › 직접 매핑 › 내신 (표준편차·석차 보완용)\n\n'
                '각 파일의 "작성요령" 시트에 규칙이 있습니다. 예시 학생 줄은 지우고 쓰세요.\n')
    made.append(readme)
    for p in made:
        print('made', p)


if __name__ == '__main__':
    main()

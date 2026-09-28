# 성적분석ON — 진행 상태

최종 갱신: 2026-09-14 (데이터 탭 재구성·설정 슬라이더 포함)

## 구조 (2026-09-14 UI 재구성)

생기부ON의 "사이드바 순서대로" 파이프라인을 버리고 **사람 중심 6개 화면**으로 재구성했다 (`docs/UI_REDESIGN.md`).
상단 바: 학년도·학년 / 반 / 학기 / 주요교과·전과목 + 학생 검색(Ctrl+K). 왼쪽 레일: 학급 · 학생 · 과목 · 학년·학교 · 진학 · 데이터.

| 화면 | 파일 | 상태 |
|---|---|---|
| 학급 한눈에 | `pages/ClassPage.vue` ← `get_class_report` | 동작 (타일·정렬 표·등급 분포·교과 레이더·먼저 만나 볼 학생) |
| 학생 상담 리포트 | `pages/StudentPage.vue` ← `get_student_bundle` | 동작 (태그·KPI·레이더·추이·과목표·내신 vs 모의·가채점·입학 대비·4대학 산식·메모·해석용 JSON·PDF/엑셀) |
| 과목별 성취 | `pages/SubjectPage.vue` ← `get_subject_options/report` | 동작 |
| 학년·학교 | `pages/SchoolPage.vue` (기존 L0~L7 섹션을 탭으로) | 칸 (계산 엔진 2단계) |
| 진학 | `pages/AdmissionPage.vue` ← 입결 집계·강원 범위·참고선 | 동작 (데이터가 있으면) |
| 데이터 | `pages/DataPage.vue` — 나이스 내신 가져오기 / 모의고사 성적 가져오기(`MockImport.vue`, 종합분석·가채점 자동 인식, 직접 매핑은 접힘) / 학생 명단 / 데이터 점검(`DataCheck.vue`) / 내보내기 / 설명서 | 동작 |
| 설정 | `sections/SettingsSection.vue` + `SliderField.vue` — 판단 기준을 슬라이더·숫자·쉬운 설명·예시로, 고급 항목은 접힘 | 동작 |

### Rust `analysis/`
- `data.rs` 학년 단위 적재(GradeData), 교과 정규화, 공통과목 판정
- `neis.rs` 나이스 교과학습발달상황 파서(5단계·3단계 블록, 학기 forward-fill, 분포비율) + 가져오기(이전 학년 기록의 학년도 자동 계산)
- `class_report.rs` gpa(표시용)·교내 위치(석차백분율 환산: 5등급 컷 구간 중앙값 학점 가중)·직전 대비·입학 대비·모의 대비·태그·학급 요약
- `student_bundle.rs` 상담 리포트 번들(추이·모의·입학·메모·익명 AI 프로필) + 학생 검색
- `subject_report.rs` 과목별 분포·반별 비교
- `mock_files.rs` 모의고사 파일 자동 인식(종합분석 시트 머리글 '국어|표준점수' 파싱, 가채점 원점수), 미응시·취합중 처리
- `univ.rs` 강원대·한림대·경희대·동국대 산식(대시보드 초안 이식), 참고선 seed/편집, 3개년 입결 집계, 9등급 연속 환산
- `school_report.rs` 담임 양식(sd·석차) 경로의 등급 내 위치 피드백 (나이스 경로는 분포·편차로 대체)
- 스키마 v3: SchoolScore 나이스 필드, SubjectCatalog, AdmissionResult, RegionalAdmission, UnivReference. 테스트 32개.

### 가져오기 양식
- `tools/make_templates.py` → `G:\...9_춘천고등학교 성적분석\양식\` 에 01~06 양식 xlsx + 읽어주세요.txt. 같은 셀 격자를 `templates/*.json` 으로 저장해 `tests/template_tests.rs` 가 파서·매핑 가져오기 통과를 검증한다 (양식을 바꾸면 스크립트 재실행).

### 데이터 이식
- `tools/etl_2026_all.py --src <원본 폴더>`: 명렬표·나이스 20파일·고1 3/6/9월·고2 3/6월·입학·편제·수시 진학결과 2023~2025·강원 대학별 범위 → `2026_춘천고_성적분석.db`
- **원본 폴더(첨부 임시 폴더)가 세션 중 사라져** 2026-09-14 현재 DB에는 고1 모의고사·입학만 들어 있다 (학생 280, 모의 5015행). 원본 파일을 안정된 폴더(예: `G:\...\71_춘천고등학교 성적분석\원본자료\`)에 두고 `--src` 로 다시 실행하면 나머지가 채워진다.
- 앱에서는 데이터 → 나이스 내신 가져오기로 download-*.xlsx 를 직접 넣을 수 있다 (ETL 없이).

## 다음
1. 원본 파일 재실행 → 2학년 나이스 내신·입결로 학급/학생/진학 화면을 실데이터로 검증
2. 학년·학교(L0~L7) 계산 엔진: z 변환·mu/sigma·환율표·스크리닝 플래그 12개
3. 학년·학교 화면의 필터를 상단 바 맥락과 통합
4. 반 전체 카드 일괄 PDF(학생당 1쪽) — 이전 `StudentCardSection` 기능 복원
5. AI 해석(B 프롬프트) 연동 — 현재는 해석용 JSON 미리보기까지

## 실행
```bash
npm run dev            # 브라우저 미리보기 (mock — src/lib/mockAnalysis.js)
npm run tauri dev      # 데스크톱 앱
npm run test:rust      # Rust 테스트
SCORE_DB=경로 cargo test realdb -- --ignored --nocapture   # 실제 DB 점검
```

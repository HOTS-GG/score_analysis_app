# GLOBAL RULES

## ARCHITECTURE
- Component MUST NOT call invoke() directly — go through `src/lib/ipc.js` inside a Store
- ALWAYS use Store
- Rust handles ALL DB, statistics and core logic

## PRINCIPLES
- Store = single source of truth
- Frontend = UI + state only
- No duplicated logic

## DOMAIN RULES (성적 분석 — 위반 시 결과 무효)
- 등급끼리 직접 빼는 연산 금지. 비교·연산은 백분위 / 교내 석차백분율로만, 등급은 표시용
- 모평 전국 백분위만 절대적 성장 지표. 내신 등급 시계열·원점수 회차 비교를 성장으로 쓰지 않음
- 서로 다른 준거(전국 vs 교내)를 비교할 때는 교내 석차백분율로 통일
- 잔차·z_resid·회귀 예측값·전이행렬 확률은 교사용. 학생 카드에 절대 출력하지 않음
- 인과 표현 금지("~때문에", "학교 교육의 효과로")
- 교사·학급 평가 목적 집계(반별 평균 잔차 순위 등) 기능 금지
- 모든 지표에 산출 N 병기. N<30 경고

## CONVENTIONS
- Rust commands: snake_case
- MUST handle errors explicitly
- 플래그 기준값은 코드에 하드코딩하지 않고 thresholds(설정)에서 읽는다

## PROHIBITED
- Silent failures
- Business logic in frontend

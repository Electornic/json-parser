# Project context for AI assistants

## Build / Test commands

- 빌드: `cargo build`
- 테스트 전체: `cargo test`
- 테스트 단위만: `cargo test --lib`
- 픽스처 검증: `cargo test --test conformance`
- 공개 API 통합: `cargo test --test public_api`
- 데모 실행: `echo '{"a":1}' | cargo run -q`
- 린트: `cargo clippy`
- 릴리스 빌드: `cargo build --release`

## Architecture

레이어가 단방향으로 의존합니다: `error → number, value → lexer → string → parser → ser → lib`.

- `src/error.rs` — `ParseError` (line/col/byte_offset), `ErrorKind`
- `src/number.rs` — `Number(String)` raw lexeme + lazy `as_i64`/`as_u64`/`as_f64`
- `src/value.rs` — `JsonValue` enum, 객체는 `Vec<(String, JsonValue)>`로 순서 보존
- `src/lexer.rs` — 토큰 스트림 생산. **재귀 없음**, byte 단위로 진행하지만 UTF-8 leading byte로 col 카운트
- `src/string.rs` — JSON string unescape (surrogate pair 처리). 입력은 lexer가 추출한 raw content
- `src/parser.rs` — recursive descent. `parse_value` + `parse_array_body` + `parse_object_body`, depth limit 128
- `src/ser.rs` — `to_string` (compact) / `to_string_pretty(indent)`. 라운드트립 보장
- `src/main.rs` — stdin → pretty-print 데모. exit 0/1/2
- `src/lib.rs` — 공개 API re-export

## Conventions

- **의존성 0** — `Cargo.toml`에 외부 크레이트 추가 금지. 표준 라이브러리만.
- **Number는 원본 lexeme 보존** — 정밀도 손실 없이 라운드트립. 변환은 호출자가 명시적으로.
- **객체 키 순서 보존** — `HashMap` 쓰지 말 것. `Vec<(String, JsonValue)>` 유지.
- **커밋은 작은 단위** — 한 모듈 또는 하나의 논리 단위 = 한 커밋. 작업 끝나기 전에 자주 커밋.
- **테스트는 모듈마다** — 각 `src/*.rs` 내부에 `#[cfg(test)] mod tests`. 통합은 `tests/`에.
- **에러는 위치 정보 포함** — `ParseError::new(kind, line, col, byte_offset)`로만 생성. 위치 없는 에러 금지.

## Gotchas

- Lexer는 문자열 토큰을 **decode하지 않음**. raw escaped content를 `Token::String(String)`에 넣고, parser가 `string::unescape`로 디코드. 두 레이어 분리.
- Lexer의 col 카운트는 UTF-8 leading byte 기준 (`(b & 0xC0) != 0x80`). bytes != codepoints.
- `read_string`은 escape sequence 형식만 검증 (`\u` 다음 4 hex 등). 의미 검증(surrogate pair, lone surrogate)은 `string::unescape`에서.
- `depth -= 1`은 에러 path에선 실행되지 않음 — 파서 인스턴스가 에러 후 폐기되므로 OK. RAII guard로 바꾸려면 `Drop` 사용.
- Number 토큰의 leading zero는 lexer가 `0`까지만 소비. `01`은 `Number("0"), Number("1")` 두 토큰이 되고, parser가 trailing data로 거부.
- 중복 객체 키는 last-wins 아니라 **둘 다 보존**됨. `JsonValue::get(key)`는 첫 매치 반환.

## Test layout

- `src/<module>.rs::tests` — 각 모듈 내부 unit 테스트 (총 ~103개)
- `tests/public_api.rs` — 공개 API 입장에서의 통합 테스트
- `tests/conformance.rs` — `tests/fixtures/` 디렉터리를 자동으로 walk
- `tests/fixtures/y_*.json` — 반드시 accept 되어야 하는 입력
- `tests/fixtures/n_*.json` — 반드시 reject 되어야 하는 입력
- 새 엣지케이스는 fixtures에 파일 추가만 하면 됨 (러너가 자동 검출)

## Out of scope (v1)

다음은 의도적으로 구현하지 않음. 추가 요청 시 명시적 승인 필요:

- 스트리밍/SAX 파서
- JSON5 (주석, 트레일링 콤마, 작은따옴표)
- async I/O
- serde 호환 (`Serialize`/`Deserialize` derive)
- 중복 키 strict 모드
- zero-copy 파싱 (`JsonValue<'a>` with borrowed strings)

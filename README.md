# json-parser

RFC 8259 준수 JSON 파서. 의존성 없이 Rust 표준 라이브러리만 사용.

## 특징

- **RFC 8259 엄격 준수** — 선행 0, 트레일링 콤마, lone surrogate 등 모두 거부
- **위치 정보 포함 에러** — 줄/열/바이트 오프셋
- **숫자 원본 보존** — 임의 정밀도 lexeme 그대로 저장, `as_i64`/`as_u64`/`as_f64` 지연 변환
- **객체 키 순서 보존** — `Vec<(String, JsonValue)>` 사용
- **재귀 깊이 제한** — 기본 128 단계 (스택 오버플로 방지)
- **의존성 0** — `Cargo.toml`에 외부 크레이트 없음

## Quick start

```rust
use json_parser::{parse, to_string_pretty};

let input = r#"{"name":"json-parser","ok":true}"#;
let value = parse(input)?;

println!("{}", to_string_pretty(&value, 2));
// {
//   "name": "json-parser",
//   "ok": true
// }
```

## API

```rust
pub fn parse(input: &str) -> Result<JsonValue, ParseError>;
pub fn to_string(value: &JsonValue) -> String;
pub fn to_string_pretty(value: &JsonValue, indent: usize) -> String;

pub enum JsonValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

pub struct Number(/* raw lexeme */);
impl Number {
    pub fn as_str(&self) -> &str;
    pub fn as_i64(&self) -> Option<i64>;
    pub fn as_u64(&self) -> Option<u64>;
    pub fn as_f64(&self) -> Option<f64>;
}

pub struct ParseError {
    pub kind: ErrorKind,
    pub line: usize,        // 1-based
    pub col: usize,         // 1-based, 코드포인트 단위
    pub byte_offset: usize,
}
```

`JsonValue`는 편의 접근자(`as_bool`, `as_str`, `as_array`, `as_object`, `get(key)`)도 제공합니다.

## 데모 바이너리

```bash
echo '{"a":1,"b":[true,null]}' | cargo run -q
# {
#   "a": 1,
#   "b": [
#     true,
#     null
#   ]
# }
```

- 정상: stdout으로 pretty-print, exit 0
- 파싱 실패: stderr로 에러 메시지, exit 1
- stdin 읽기 실패: exit 2

## 에러 예시

```text
$ echo '{"a":}' | cargo run -q
parse error: expected a JSON value at line 1, column 6
```

## 테스트

```bash
cargo test                       # 전체 (unit + integration + conformance)
cargo test --lib                 # 모듈 unit 테스트
cargo test --test conformance    # JSONTestSuite 스타일 픽스처
cargo test --test public_api     # 공개 API 통합 테스트
```

`tests/fixtures/y_*.json`는 반드시 통과해야 하고, `n_*.json`은 반드시 거부돼야 합니다. 새 케이스는 파일만 추가하면 자동으로 검증됩니다.

## 비범위 (v1)

다음은 의도적으로 구현하지 않았습니다:

- 스트리밍/SAX 파서 (입력 전체를 한 번에 읽음)
- JSON5 확장 (주석, 트레일링 콤마, 작은따옴표 등)
- async I/O
- serde 호환 (`Serialize`/`Deserialize` derive)
- 중복 키 strict 거부 모드 (현재는 last-wins로 보존)

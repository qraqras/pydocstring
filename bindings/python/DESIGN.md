# Python バインディング設計方針

## 概要

Python バインディング (`pydocstring-rs`) の walk インターフェースを、コアクレートの `DocstringVisitor` トレイトを直接使う設計に移行する。
セクションオブジェクトは **thin wrapper**（子を持たない）とし、子へのアクセスは walk 経由のみとする。

---

## 現状の問題点

### Eager 構築

現在の `PyGoogleSection` は構築時に全ての子を `Py::new` で確保する:

```rust
struct PyGoogleSection {
    args: Vec<Py<PyGoogleArg>>,
    returns: Option<Py<PyGoogleReturn>>,
    // ...
}
```

- `visit_google_arg` が inactive であっても全 arg に対して `Py::new` が走る
- `walk_section` という複雑な関数が必要になる

### 手書き walk ロジックの重複

コアクレートに `DocstringVisitor` + `walk_node` があるにもかかわらず、バインディング独自の `walk_cst` / `walk_node` / `walk_section` を手書きしている。

---

## 設計方針

### 原則

1. **Walk の唯一の実装はコアの `DocstringVisitor`**
   バインディング独自の walk 関数は持たない。

2. **セクションオブジェクトは thin**
   `PyGoogleSection` / `PyNumPySection` は位置情報と種別のみ保持する。子は持たない。

3. **子へのアクセスは walk 経由**
   `section.args` 等の直接アクセスは提供しない。`visit_google_arg` で受け取る。

4. **セクション内の全エントリ収集は leave で完結**
   セクション開始 (`visit_google_section`) 〜 終了 (`leave_google_section`) の間に子が dispatch される順序を保証する。

---

## 変更内容

### Step 1: コア — `leave_*` を `DocstringVisitor` に追加（非破壊的）

`walk_node` が各ノードの dispatch 後に `leave_*` を呼ぶ:

```rust
// walk_node 内
visitor.visit_google_section(source, sec)?;   // enter
// (デフォルト実装内で walk_node → 子が dispatch される)
visitor.leave_google_section(source, sec)?;   // leave
```

全 `leave_*` メソッドのデフォルト実装は `Ok(())` を返すため、既存コードへの影響はない。

### Step 2: バインディング — struct を thin に変更

```rust
// After
#[pyclass]
struct PyGoogleSection {
    range: TextRange,
    section_kind: PyGoogleSectionKind,
    header_name: Py<PyToken>,
    // children フィールドなし
}

#[pyclass]
struct PyNumPySection {
    range: TextRange,
    section_kind: PyNumPySectionKind,
    header_name: Py<PyToken>,
    // children フィールドなし
}
```

### Step 3: バインディング — `PyDispatcher` を実装

```rust
struct PyDispatcher<'py> {
    py: Python<'py>,
    arc: Arc<Parsed>,
    visitor: &'py Py<PyAny>,
    active: &'py ActiveMethods,
    ctx: Py<PyWalkContext>,
}

impl DocstringVisitor for PyDispatcher<'_> {
    type Error = PyErr;

    fn visit_google_section(
        &mut self, source: &str, sec: &GoogleSection<'_>
    ) -> Result<(), PyErr> {
        if self.active.google_section {
            let obj = Py::new(self.py, PyGoogleSection {
                range: *sec.syntax().range(),
                section_kind: google_section_kind_to_py(sec.section_kind(source)),
                header_name: mk_token(self.py, sec.header().name(), source)?,
            })?;
            self.visitor.call_method1(self.py, "visit_google_section", (obj, &self.ctx))?;
        }
        walk_node(source, sec.syntax(), self)
    }

    fn leave_google_section(
        &mut self, source: &str, sec: &GoogleSection<'_>
    ) -> Result<(), PyErr> {
        if self.active.leave_google_section {
            let obj = Py::new(self.py, PyGoogleSection { ... })?;
            self.visitor.call_method1(self.py, "leave_google_section", (obj, &self.ctx))?;
        }
        Ok(())
    }

    // 全ノード種別で同様のパターン
}
```

### Step 4: バインディング — 旧 walk 関数を削除

削除対象:
- `walk_cst`
- `walk_node`（バインディング側）
- `walk_section`

`walk()` Python 関数は `PyDispatcher` を使うように変更する。

### Step 5: テストを walk ベースに書き直し

```python
# Before（直接アクセス — 廃止）
args = doc.sections[0].args
assert args[0].name.text == "x"

# After（walk ベース）
class Collector:
    def __init__(self): self.args = []
    def visit_google_arg(self, arg, ctx): self.args.append(arg)

c = Collector()
pydocstring.walk(doc, c)
assert c.args[0].name.text == "x"
```

---

## Python API の変化

### 維持されるもの

- `doc.sections` — セクションリスト（thin オブジェクト）
- `section.section_kind` — セクション種別
- `section.header_name` — セクションヘッダーテキスト
- `section.range` — バイト位置
- `walk(doc, visitor)` — walk インターフェース全体
- `visit_google_arg` / `visit_numpy_parameter` 等の visit メソッド

### 追加されるもの

- `leave_google_section` / `leave_google_arg` 等の leave メソッド（全ノード種別）

### 削除されるもの

- `section.args` / `section.returns` / `section.exceptions` 等、セクション経由の子への直接アクセス

---

## ユースケース例

### Args セクション内の引数順序チェック

```python
class ArgOrderLinter:
    def __init__(self, expected_order):
        self.expected = expected_order
        self._in_args = False
        self._seen = []
        self.violations = []

    def visit_google_section(self, section, ctx):
        self._in_args = (section.section_kind == pydocstring.GoogleSectionKind.ARGS)
        self._seen = []

    def visit_google_arg(self, arg, ctx):
        if self._in_args:
            self._seen.append(arg.name.text)

    def leave_google_section(self, section, ctx):
        if self._in_args:
            expected = [n for n in self.expected if n in self._seen]
            if self._seen != expected:
                self.violations.append(
                    f"Wrong order: {self._seen}, expected {expected}"
                )

linter = ArgOrderLinter(["x", "y", "z"])
pydocstring.walk(doc, linter)
print(linter.violations)
```

### 型アノテーション欠落チェック

```python
class MissingTypeLinter:
    def __init__(self):
        self.missing = []

    def visit_google_arg(self, arg, ctx):
        if arg.type is None:
            self.missing.append(arg.name.text)
```

---

## 実装順序

1. コア `visitor.rs`: `leave_*` メソッドと `walk_node` の呼び出し追加
2. バインディング: `PyGoogleSection` / `PyNumPySection` を thin に
3. バインディング: `ActiveMethods` に `leave_*` フラグ追加
4. バインディング: `PyDispatcher` 実装
5. バインディング: 旧 `walk_cst` / `walk_node` / `walk_section` 削除
6. テスト書き直し
7. ビルド・全テスト通過確認
8. コミット

# pydocstringについて
- pydocstringはPythonのdocstringをパースするライブラリです
- pydocstringはNumPy/Google/Sphinxスタイルをサポートします
- 将来的にリンタやフォーマッタで使用されることを想定しています
- Rustで実装します
- Rustの外部クレートは使用しません

# 実装方針
- NumPy->Google->Sphinxの順で実装します
- リンタ向けにパース結果には位置情報を含めます
- 設計方針はBiomeを参考にします
- 内部実装はスタイルごとに具体的な構造体を定義します
- 公開APIは抽象化された構造体を返すようにします

# docstringのスタイルガイド
- NumPy: https://numpydoc.readthedocs.io/en/latest/format.html

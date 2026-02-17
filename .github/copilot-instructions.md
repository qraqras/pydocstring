# pydocstringについて
- pydocstringはPythonのdocstringをパースするライブラリです
- pydocstringはNumPy/Google/Sphinxスタイルをサポートします
- Rustで実装します
- Rustの外部クレートは使用しません

# 実装方針
- NumPy->Google->Sphinxの順で実装します
- 各スタイルのパーサーは独立して実装します

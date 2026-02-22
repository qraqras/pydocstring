use pydocstring::google::parse_google;
use pydocstring::GoogleSectionBody;

fn main() {
    let input = "\
Returns:
  A dict mapping keys to the corresponding table row data
  fetched. Each row is represented as a tuple of strings. For
  example:

  {b'Serak': ('Rigel VII', 'Preparer'),
        b'Zim': ('Irk', 'Invader'),
        b'Lrrr': ('Omicron Persei 8', 'Emperor')}

  Returned keys are always bytes.  If a key from the keys argument is
  missing from the dictionary, then that row was not found in the
  table (and require_all_keys must have been False).
";
    let doc = parse_google(input);
    println!("Summary: {:?}", doc.summary.value);
    println!("Items: {}", doc.items.len());
    for (idx, item) in doc.items.iter().enumerate() {
        match item {
            pydocstring::GoogleDocstringItem::Section(s) => {
                println!("Item {}: Section {:?}", idx, s.header.name.value);
                if let GoogleSectionBody::Returns(ref ret) = s.body {
                    let type_str = ret.return_type.as_ref().map(|t| &t.value);
                    let d = &ret.description.value;
                    println!("  type: {:?}", type_str);
                    println!("  desc: {:?}", if d.len() > 80 { &d[..80] } else { d });
                }
            }
            pydocstring::GoogleDocstringItem::StrayLine(s) => {
                println!("Item {}: StrayLine {:?}", idx, s.value);
            }
        }
    }
}

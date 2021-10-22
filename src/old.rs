use editor_input::input_from_editor;
use indicium::simple::{Indexable, SearchIndex};
use std::io::Result;

struct TestString {
    string: String,
}


impl Indexable for TestString {
    fn strings(&self) -> Vec<String> {
        vec![self.string.clone()]
    }
}

fn main() -> Result<()> {
    let stuffs = &[
        TestString {
            string: "hej".to_string(),
        },
        TestString {
            string: "the old man and the sea".to_string(),
        },
        TestString {
            string: "the invisible man".to_string(),
        },
        TestString {
            string: "the big book of java".to_string(),
        },
        TestString {
            string: "hej kaj".to_string(),
        },
        TestString {
            string: "hallå boje".to_string(),
        },
    ];
    let mut search_index: SearchIndex<usize> = SearchIndex::default();
    stuffs
        .iter()
        .enumerate()
        .for_each(|(i, e)| search_index.insert(&i, e));

    let input = input_from_editor()?;
    dbg!(search_index.search(&input));
    Ok(())
}

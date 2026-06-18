use anyhow::Result;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase,
};

use crate::{
    cli::{CaseArgs, CaseStyle, GlobalOptions},
    output,
};

pub fn run(args: &CaseArgs, global: &GlobalOptions) -> Result<()> {
    let converted = convert(args.style, &args.text);
    output::write_or_json(
        global,
        || {
            println!("{converted}");
            Ok(())
        },
        &serde_json::json!({ "style": format!("{:?}", args.style), "output": converted }),
    )
}

pub fn convert(style: CaseStyle, input: &str) -> String {
    match style {
        CaseStyle::Camel => input.to_lower_camel_case(),
        CaseStyle::Pascal => input.to_pascal_case(),
        CaseStyle::Snake => input.to_snake_case(),
        CaseStyle::Kebab => input.to_kebab_case(),
        CaseStyle::ScreamingSnake => input.to_shouty_snake_case(),
        CaseStyle::Title => input.to_title_case(),
        CaseStyle::Sentence => sentence_case(input),
    }
}

fn sentence_case(input: &str) -> String {
    let mut words = input.to_kebab_case().replace('-', " ");
    if let Some(first) = words.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_snake() {
        assert_eq!(convert(CaseStyle::Snake, "Hello world"), "hello_world");
    }
}

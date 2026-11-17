use notra_ui::Theme;
use notra_ui::components::code_block::demo;
use notra_ui::rune::data_parts::{TabOption, Tabs};
use notra_ui::rune::{CodeBlock, Language};
use rune::{El, col};

use crate::{State, section};

#[derive(Default)]
pub struct Demo {
    pub framework: usize,
    pub copies: [u32; 5],
}

const JSX: &str = "export function Hello({ name }: { name: string }) {\n  // Greets the reader by name.\n  return <p className=\"text-sm\">Hello, {name}!</p>;\n}";
const JSON: &str =
    "{\n  \"name\": \"@usenotra/geo\",\n  \"version\": \"1.4.0\",\n  \"private\": false\n}";
const BASH: &str =
    "# Install and run\nexport NOTRA_GEO_TOKEN=ntr_geo_123\nbun add @usenotra/geo && bun run dev";

fn copied(i: usize) -> impl Fn(&mut State) + 'static {
    move |s: &mut State| s.code_block.copies[i] += 1
}

pub fn view(s: &State, t: &Theme) -> El {
    let demo_state = &s.code_block;
    let (_, file, code) = demo::FRAMEWORKS[demo_state.framework];
    let tabs = Tabs::new(
        demo::FRAMEWORKS
            .iter()
            .map(|(label, _, _)| TabOption::new(*label))
            .collect(),
        demo_state.framework,
    )
    .label("Framework")
    .on_select(|s: &mut State, i| s.code_block.framework = i)
    .build(t);
    let column = |children: Vec<El>| col().gap(16.0).max_w(528.0).w_full().children(children);
    col()
        .child(section(
            "Demo",
            t,
            column(vec![
                CodeBlock::new(code)
                    .filename(file)
                    .tabs(tabs)
                    .copies(demo_state.copies[0])
                    .on_copy(copied(0))
                    .build(t)
                    .key("snippet"),
                CodeBlock::new(demo::COMMAND)
                    .command()
                    .copies(demo_state.copies[1])
                    .on_copy(copied(1))
                    .build(t)
                    .key("token"),
            ]),
        ))
        .child(section(
            "Languages",
            t,
            column(vec![
                CodeBlock::new(JSX)
                    .language(Language::TypeScript)
                    .filename("hello.tsx")
                    .copies(demo_state.copies[2])
                    .on_copy(copied(2))
                    .build(t)
                    .key("tsx"),
                CodeBlock::new(JSON)
                    .language(Language::Json)
                    .filename("package.json")
                    .copies(demo_state.copies[3])
                    .on_copy(copied(3))
                    .build(t)
                    .key("json"),
                CodeBlock::new(BASH)
                    .language(Language::Bash)
                    .copies(demo_state.copies[4])
                    .on_copy(copied(4))
                    .build(t)
                    .key("bash"),
            ]),
        ))
}

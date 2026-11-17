use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use notra_ui::components::code_block::{Language, demo};
use notra_ui::gpui::CodeBlock;
use notra_ui::gpui::data_parts::{TabOption, Tabs};

use crate::{Page, section};

pub fn page(_window: &mut Window, cx: &mut App) -> Page {
    Page {
        slug: "code-block",
        title: "Code Block",
        view: cx.new(|_| CodeBlockPage::default()).into(),
    }
}

#[derive(Default)]
struct CodeBlockPage {
    framework: usize,
}

const JSX: &str = "export function Hello({ name }: { name: string }) {\n  // Greets the reader by name.\n  return <p className=\"text-sm\">Hello, {name}!</p>;\n}";
const JSON: &str =
    "{\n  \"name\": \"@usenotra/geo\",\n  \"version\": \"1.4.0\",\n  \"private\": false\n}";
const BASH: &str =
    "# Install and run\nexport NOTRA_GEO_TOKEN=ntr_geo_123\nbun add @usenotra/geo && bun run dev";

impl Render for CodeBlockPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (_, file, code) = demo::FRAMEWORKS[self.framework];
        let options = demo::FRAMEWORKS
            .iter()
            .map(|(label, _, _)| TabOption::new(*label))
            .collect();
        let tabs = Tabs::new("framework", options, self.framework).on_change(cx.listener(
            |this, i: &usize, _, cx| {
                this.framework = *i;
                cx.notify();
            },
        ));
        div()
            .flex()
            .flex_col()
            .child(
                section("Demo", cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .max_w(px(528.0))
                        .w_full()
                        .child(CodeBlock::new("snippet", code).filename(file).tabs(tabs))
                        .child(CodeBlock::new("token", demo::COMMAND).command()),
                ),
            )
            .child(
                section("Languages", cx).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .max_w(px(528.0))
                        .w_full()
                        .child(
                            CodeBlock::new("tsx", JSX)
                                .language(Language::TypeScript)
                                .filename("hello.tsx"),
                        )
                        .child(
                            CodeBlock::new("json", JSON)
                                .language(Language::Json)
                                .filename("package.json"),
                        )
                        .child(CodeBlock::new("bash", BASH).language(Language::Bash)),
                ),
            )
    }
}

use leptos::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="header-content">
                <div class="logo">
                    <h1>"🔄 Kairos Gateway"</h1>
                    <span class="version">"v0.2.6"</span>
                </div>
                <nav class="nav">
                    <div class="nav-links">
                        <a href="https://github.com/DanielSarmiento04/kairos-rs" target="_blank">
                            "📚 Docs"
                        </a>
                        <a href="https://github.com/DanielSarmiento04/kairos-rs" target="_blank">
                            "⭐ GitHub"
                        </a>
                    </div>
                </nav>
            </div>
        </header>
    }
}
use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="sidebar">
            <div class="nav-brand">
                <h1>"🔄 Kairos"</h1>
                <span class="version">"v0.2.6"</span>
            </div>
            
            <nav class="sidebar-nav">
                <ul class="nav-menu">
                    <li class="nav-item">
                        <A href="" class="nav-link">
                            <span class="nav-icon">"📊"</span>
                            <span class="nav-label">"Dashboard"</span>
                        </A>
                    </li>
                    <li class="nav-item">
                        <A href="/routes" class="nav-link">
                            <span class="nav-icon">"🛤️"</span>
                            <span class="nav-label">"Routes"</span>
                        </A>
                    </li>
                    <li class="nav-item">
                        <A href="/metrics" class="nav-link">
                            <span class="nav-icon">"📈"</span>
                            <span class="nav-label">"Metrics"</span>
                        </A>
                    </li>
                    <li class="nav-item">
                        <A href="/config" class="nav-link">
                            <span class="nav-icon">"⚙️"</span>
                            <span class="nav-label">"Configuration"</span>
                        </A>
                    </li>
                    <li class="nav-item">
                        <A href="/health" class="nav-link">
                            <span class="nav-icon">"💚"</span>
                            <span class="nav-label">"Health"</span>
                        </A>
                    </li>
                </ul>
            </nav>
            
            <div class="sidebar-footer">
                <div class="footer-links">
                    <a href="https://github.com/DanielSarmiento04/kairos-rs" target="_blank" class="footer-link">
                        <span class="nav-icon">"📚"</span>
                        <span class="nav-label">"Documentation"</span>
                    </a>
                    <a href="https://github.com/DanielSarmiento04/kairos-rs" target="_blank" class="footer-link">
                        <span class="nav-icon">"⭐"</span>
                        <span class="nav-label">"GitHub"</span>
                    </a>
                </div>
            </div>
        </aside>
    }
}
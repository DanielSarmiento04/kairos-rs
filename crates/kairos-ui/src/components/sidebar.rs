use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="sidebar">
            <nav class="sidebar-nav">
                <ul>
                    <li>
                        <A href="" class="nav-link">
                            <span class="icon">"📊"</span>
                            <span class="label">"Dashboard"</span>
                        </A>
                    </li>
                    <li>
                        <A href="/routes" class="nav-link">
                            <span class="icon">"🛤️"</span>
                            <span class="label">"Routes"</span>
                        </A>
                    </li>
                    <li>
                        <A href="/metrics" class="nav-link">
                            <span class="icon">"📈"</span>
                            <span class="label">"Metrics"</span>
                        </A>
                    </li>
                    <li>
                        <A href="/config" class="nav-link">
                            <span class="icon">"⚙️"</span>
                            <span class="label">"Configuration"</span>
                        </A>
                    </li>
                    <li>
                        <A href="/health" class="nav-link">
                            <span class="icon">"💚"</span>
                            <span class="label">"Health"</span>
                        </A>
                    </li>
                </ul>
            </nav>
        </aside>
    }
}
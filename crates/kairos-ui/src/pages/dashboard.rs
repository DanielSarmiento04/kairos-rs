use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="dashboard">
            <div class="page-header">
                <h1>"Dashboard"</h1>
                <p>"Overview of your Kairos API Gateway"</p>
            </div>
            
            <div class="dashboard-grid">
                <div class="metric-card">
                    <h3>"🚀 Requests/sec"</h3>
                    <div class="metric-value">"1,234"</div>
                </div>
                
                <div class="metric-card">
                    <h3>"✅ Success Rate"</h3>
                    <div class="metric-value">"99.5%"</div>
                </div>
                
                <div class="metric-card">
                    <h3>"⚡ Avg Latency"</h3>
                    <div class="metric-value">"12ms"</div>
                </div>
                
                <div class="metric-card">
                    <h3>"🔗 Active Routes"</h3>
                    <div class="metric-value">"8"</div>
                </div>
            </div>
            
            <div class="dashboard-sections">
                <section class="recent-activity">
                    <h2>"Recent Activity"</h2>
                    <div class="activity-list">
                        <div class="activity-item">
                            <span class="time">"2 min ago"</span>
                            <span class="event">"Route /api/users/{id} added"</span>
                        </div>
                        <div class="activity-item">
                            <span class="time">"5 min ago"</span>
                            <span class="event">"Rate limit updated to 1000 req/min"</span>
                        </div>
                        <div class="activity-item">
                            <span class="time">"10 min ago"</span>
                            <span class="event">"Circuit breaker opened for user-service"</span>
                        </div>
                    </div>
                </section>
                
                <section class="system-status">
                    <h2>"System Status"</h2>
                    <div class="status-grid">
                        <div class="status-item">
                            <span class="status-label">"Gateway"</span>
                            <span class="status-indicator healthy">"Healthy"</span>
                        </div>
                        <div class="status-item">
                            <span class="status-label">"Auth Service"</span>
                            <span class="status-indicator healthy">"Healthy"</span>
                        </div>
                        <div class="status-item">
                            <span class="status-label">"User Service"</span>
                            <span class="status-indicator warning">"Degraded"</span>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
}
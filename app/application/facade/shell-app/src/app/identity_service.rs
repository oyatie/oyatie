use super::*;

pub(super) fn identity_workforce_service() -> impl IntoView {
    view! {
        <section
            id="identity-workforce-service"
            class="identity-workforce-service panel"
            data-identity-service="true"
            aria-labelledby="identity-service-title"
        >
            <div class="identity-service-head">
                <div>
                    <p class="screen-anchor">"SETTINGS · WORKFORCE"</p>
                    <h3 id="identity-service-title">"Identity, organization profile, onboarding, and employees"</h3>
                </div>
                <div class="identity-service-actions">
                    <span class="status-chip success" data-identity-status="true">"local profile ready"</span>
                    <button type="button" data-identity-action="open-audit">"Audit log"</button>
                </div>
            </div>

            <div class="identity-service-shell">
                <aside class="identity-settings-rail" aria-label="Identity and organization sections">
                    <div class="identity-person">
                        <span aria-hidden="true">"최"</span>
                        <div><strong>"최유나 · Choi Yu-na"</strong><small>"Admin · Oyatie Corp."</small></div>
                    </div>
                    <p class="screen-anchor">"ACCOUNT"</p>
                    <button type="button" class="active" data-identity-tab="auth">"패스키 · MFA"</button>
                    <button type="button" data-identity-tab="sessions">"세션 · 기기"</button>
                    <button type="button" data-identity-tab="roles">"역할 · 권한"</button>
                    <p class="screen-anchor">"WORKSPACE"</p>
                    <button type="button" data-identity-tab="org">"조직 프로필"</button>
                    <button type="button" data-identity-tab="employees">"구성원"</button>
                    <button type="button" data-identity-tab="onboarding">"워크스페이스 설정"</button>
                    <p class="identity-chain">"Oyatie v0.1 · chain 0x4f81 · last sync 09:14 KST"</p>
                </aside>

                <div class="identity-service-main">
                    // A-2: identity tablist — aria-orientation + id/aria-controls + role=tabpanel
                    <div class="identity-tabs" role="tablist" aria-label="Identity service views" aria-orientation="horizontal">
                        <button type="button" id="identity-tab-auth" class="active" data-identity-tab="auth" role="tab" aria-selected="true" aria-controls="identity-panel-auth">"Auth"</button>
                        <button type="button" id="identity-tab-sessions" data-identity-tab="sessions" role="tab" aria-selected="false" aria-controls="identity-panel-sessions">"Sessions"</button>
                        <button type="button" id="identity-tab-roles" data-identity-tab="roles" role="tab" aria-selected="false" aria-controls="identity-panel-roles">"Roles"</button>
                        <button type="button" id="identity-tab-org" data-identity-tab="org" role="tab" aria-selected="false" aria-controls="identity-panel-org">"Org profile"</button>
                        <button type="button" id="identity-tab-employees" data-identity-tab="employees" role="tab" aria-selected="false" aria-controls="identity-panel-employees">"Employees"</button>
                        <button type="button" id="identity-tab-onboarding" data-identity-tab="onboarding" role="tab" aria-selected="false" aria-controls="identity-panel-onboarding">"Onboarding"</button>
                    </div>

                    <article id="identity-panel-auth" class="identity-panel active" data-identity-panel="auth" role="tabpanel" aria-labelledby="identity-tab-auth">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"ACCOUNT · PASSKEYS"</p>
                            <h4 id="auth-panel-title">"패스키 · MFA"</h4>
                            <p>"가능한 모든 기기에 패스키를 등록하면 더 빠르고 안전하게 로그인합니다."</p>
                        </div>
                        <div class="auth-grid">
                            <div class="auth-method-list" data-passkey-list="true">
                                <div class="auth-method"><span>"⌘"</span><strong>"MacBook Pro · Touch ID"</strong><small>"passkey · macOS · Apple · PRIMARY"</small><em>"방금 전"</em></div>
                                <div class="auth-method"><span>"◉"</span><strong>"iPhone 15 Pro · Face ID"</strong><small>"passkey · iOS · Apple"</small><em>"12 hours ago"</em></div>
                                <div class="auth-method"><span>"▣"</span><strong>"Authy · personal phone"</strong><small>"totp · added 2025-11-03"</small><em>"3 weeks ago"</em></div>
                                <div class="auth-method"><span>"⌁"</span><strong>"Recovery codes (10)"</strong><small>"recovery · printed · 10 unused"</small><em>"never"</em></div>
                            </div>
                            <aside class="security-score-card">
                                <p class="screen-anchor">"SECURITY SCORE"</p>
                                <strong data-security-score="true">"94/100"</strong>
                                // A-6: security score progressbar
                                <span class="score-bar" role="progressbar" aria-valuenow="94" aria-valuemin="0" aria-valuemax="100" aria-label="Security score: 94 out of 100" style="--bar: 94%"><em aria-hidden="true"></em></span>
                                <ol>
                                    <li>"✓ 패스키 2개 등록됨"</li>
                                    <li>"✓ TOTP 백업 활성화"</li>
                                    <li>"✓ 복구 코드 미사용"</li>
                                    <li>"○ Apple Watch 패스키 미등록"</li>
                                </ol>
                                <button type="button" data-identity-action="add-passkey">"+ 패스키 추가"</button>
                            </aside>
                        </div>
                        {identity_command_board()}
                    </article>

                    <article id="identity-panel-sessions" class="identity-panel" data-identity-panel="sessions" role="tabpanel" aria-labelledby="identity-tab-sessions">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"ACCOUNT · SESSIONS"</p>
                            <h4 id="sessions-panel-title">"세션 · 기기"</h4>
                            <p>"현재 로그인한 기기, 최근 활동, 의심 신호를 한 화면에서 확인합니다."</p>
                        </div>
                        <div class="identity-session-grid">
                            <article class="session-card primary">
                                <span class="device-glyph" aria-hidden="true">"⌘"</span>
                                <div><strong>"MacBook Pro"</strong><small>"Chrome · Seoul · current session"</small></div>
                                <em>"방금 전"</em>
                            </article>
                            <article class="session-card">
                                <span class="device-glyph" aria-hidden="true">"◉"</span>
                                <div><strong>"iPhone 15 Pro"</strong><small>"Safari · Seoul · passkey verified"</small></div>
                                <em>"12 hours ago"</em>
                            </article>
                            <article class="session-card">
                                <span class="device-glyph" aria-hidden="true">"▣"</span>
                                <div><strong>"Edge on Windows"</strong><small>"Finance office · remembered device"</small></div>
                                <em>"3 days ago"</em>
                            </article>
                        </div>
                        <ol class="identity-audit-log">
                            <li><time>"09:14"</time><strong>"New passkey challenge passed"</strong><span>"MFA guardrail OK"</span></li>
                            <li><time>"08:42"</time><strong>"Payroll role inspected"</strong><span>"No write mutation"</span></li>
                            <li><time>"Yesterday"</time><strong>"Recovery codes viewed"</strong><span>"Admin acknowledgement required"</span></li>
                        </ol>
                        {identity_sessions_anchor_board()}
                    </article>

                    <article id="identity-panel-roles" class="identity-panel" data-identity-panel="roles" role="tabpanel" aria-labelledby="identity-tab-roles">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"ACCOUNT · ACCESS"</p>
                            <h4 id="roles-panel-title">"역할 · 권한"</h4>
                            <p>"워크플로우, 급여, 감사, 직원 정보 접근이 어떤 근거로 허용되는지 표시합니다."</p>
                        </div>
                        <table class="role-matrix-table">
                            <thead><tr><th>"역할"</th><th>"범위"</th><th>"결정"</th><th>"근거"</th></tr></thead>
                            <tbody>
                                <tr><td><strong>"Tenant Admin"</strong><small>"owner"</small></td><td>"Workspace · billing · users"</td><td><span class="status-chip success">"Allow"</span></td><td>"법인 관리자"</td></tr>
                                <tr><td><strong>"Payroll Approver"</strong><small>"finance"</small></td><td>"Payroll close · filing"</td><td><span class="status-chip warning">"Review"</span></td><td>"2-person approval"</td></tr>
                                <tr><td><strong>"Workflow Builder"</strong><small>"studio"</small></td><td>"Draft · simulate"</td><td><span class="status-chip success">"Allow"</span></td><td>"No live execution"</td></tr>
                                <tr><td><strong>"External Vendor"</strong><small>"guest"</small></td><td>"Employee PII"</td><td><span class="status-chip danger">"Deny"</span></td><td>"PIPA boundary"</td></tr>
                            </tbody>
                        </table>
                        {identity_roles_anchor_board()}
                    </article>

                    <article id="identity-panel-org" class="identity-panel" data-identity-panel="org" role="tabpanel" aria-labelledby="identity-tab-org">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"SETTINGS · OVERVIEW"</p>
                            <h4 id="org-panel-title">"조직 프로필"</h4>
                            <p>"모든 핵심 설정을 한 곳에서 관리하고 변경사항은 감사 체인에 자동 기록됩니다."</p>
                        </div>
                        <div class="org-stat-grid">
                            <span><small>"임직원"</small><strong>"118명"</strong><em>"▲ +6 last quarter"</em></span>
                            <span><small>"월 인건비"</small><strong>"₩894,000,000"</strong><em>"▲ +4.2% MoM"</em></span>
                            <span><small>"활성 워크플로우"</small><strong>"42개"</strong><em>"▲ +3 since launch"</em></span>
                            <span><small>"미해결 행동항목"</small><strong>"7건"</strong><em>"▼ −2 this week"</em></span>
                        </div>
                        <div class="org-profile-grid">
                            <dl><dt>"법인명"</dt><dd>"오야티 주식회사"</dd><dt>"사업자등록번호"</dt><dd>"123-45-67890"</dd><dt>"대표자"</dt><dd>"이재현"</dd><dt>"본점"</dt><dd>"서울 강남구 테헤란로 521 12층"</dd></dl>
                            <dl><dt>"주거래"</dt><dd>"신한은행 · 주식회사 오야티"</dd><dt>"출금 항목"</dt><dd>"급여 · 원천세 · 4대보험"</dd><dt>"검증 상태"</dt><dd>"✓ 1원 검증 완료"</dd><dt>"결제 카드"</dt><dd>"신한카드 ****-4081"</dd></dl>
                            <dl><dt>"주기"</dt><dd>"월급"</dd><dt>"지급일"</dt><dd>"매월 25일"</dd><dt>"근태 마감"</dt><dd>"3일 전 (22일)"</dd><dt>"다음 마감"</dt><dd>"2026-05-22 (금)"</dd></dl>
                            <dl><dt>"국민연금"</dt><dd>"12345678901"</dd><dt>"건강보험"</dt><dd>"234567890"</dd><dt>"고용보험"</dt><dd>"EI-2024-0091"</dd><dt>"산재보험"</dt><dd>"WCI-2024-0091 · 0.65%"</dd></dl>
                        </div>
                        {identity_org_anchor_board()}
                    </article>

                    <article id="identity-panel-employees" class="identity-panel" data-identity-panel="employees" role="tabpanel" aria-labelledby="identity-tab-employees">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"SETTINGS · EMPLOYEES"</p>
                            <h4 id="employees-panel-title">"직원 디렉토리"</h4>
                            <p>
                                "FD-001 people, payroll, policy, Mail, and Community workloads stay product-first while "
                                "Oyatie Cloud proves identity data can be hosted as a governed tenant surface."
                            </p>
                        </div>
                        <div class="employee-directory-stats">
                            <span><small>"전체"</small><strong>"118명"</strong></span>
                            <span><small>"활성"</small><strong>"109명"</strong></span>
                            <span><small>"최근 30일 입사"</small><strong>"6명"</strong></span>
                            <span><small>"수습 종료 임박"</small><strong>"5명"</strong></span>
                        </div>
                        {workforce_anchor_board()}
                        <div class="employee-directory-tools">
                            <label><span aria-hidden="true">"⌕"</span><input data-employee-search="true" aria-label="Search employees" placeholder="이름, 직책, 팀, ID 검색..." /></label>
                            <div class="employee-filter-pills" aria-label="Employee filters">
                                <button type="button" class="active" data-employee-filter="all">"활성 109"</button>
                                <button type="button" data-employee-filter="infrastructure">"플랫폼팀"</button>
                                <button type="button" data-employee-filter="finance">"Finance"</button>
                            </div>
                            <button type="button" data-identity-action="add-employee">"+ 직원 추가"</button>
                        </div>
                        <table class="employee-directory-table">
                            <thead><tr><th>"이름"</th><th>"직책"</th><th>"부서 · 팀"</th><th>"매니저"</th><th>"입사일"</th><th>"상태"</th><th>"Action"</th></tr></thead>
                            <tbody>
                                <tr data-employee-row="true" data-employee-team="office"><td><strong>"이재현"</strong><small>"Jaehyun Lee · emp_0000"</small></td><td>"Chief Executive Officer"</td><td>"Office of CEO"</td><td>"—"</td><td>"2021-03-14"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="finance"><td><strong>"최유나"</strong><small>"Yuna Choi · emp_0011"</small></td><td>"Chief Financial Officer"</td><td>"Finance"</td><td>"이재현"</td><td>"2022-04-01"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>"박서준"</strong><small>"Seojun Park · emp_0001"</small></td><td>"VP of Engineering"</td><td>"플랫폼팀"</td><td>"이재현"</td><td>"2021-06-01"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>"김지영"</strong><small>"Jiyoung Kim · emp_0002"</small></td><td>"Engineering Manager"</td><td>"플랫폼팀"</td><td>"박서준"</td><td>"2022-09-12"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="infrastructure"><td><strong>"윤태민"</strong><small>"Taemin Yoon · emp_0003"</small></td><td>"Senior Software Engineer"</td><td>"플랫폼팀"</td><td>"김지영"</td><td>"2026-05-12"</td><td><span class="status-chip warning">"수습"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="product"><td><strong>"임도윤"</strong><small>"Doyun Lim · emp_0004"</small></td><td>"Engineering Manager · Product"</td><td>"프로덕트팀"</td><td>"박서준"</td><td>"2022-11-04"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="product"><td><strong>"강수아"</strong><small>"Sua Kang · emp_0005"</small></td><td>"Software Engineer"</td><td>"프로덕트팀"</td><td>"임도윤"</td><td>"2024-02-19"</td><td><span class="status-chip">"휴직"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                                <tr data-employee-row="true" data-employee-team="data"><td><strong>"정우진"</strong><small>"Woojin Jung · emp_0006"</small></td><td>"Software Engineer · Frontend"</td><td>"데이터팀"</td><td>"임도윤"</td><td>"2025-08-07"</td><td><span class="status-chip success">"활성"</span></td><td><button type="button" data-employee-action="inspect">"Inspect"</button></td></tr>
                            </tbody>
                        </table>
                    </article>

                    <article id="identity-panel-onboarding" class="identity-panel" data-identity-panel="onboarding" role="tabpanel" aria-labelledby="identity-tab-onboarding">
                        <div class="identity-panel-copy">
                            <p class="screen-anchor">"WORKSPACE SETUP"</p>
                            <h4 id="onboarding-panel-title">"워크스페이스 설정"</h4>
                            <p>
                                "Workspace setup is the tenant-admission path: FD-001 product workloads collect legal, payroll, "
                                "policy, schedule, and evidence facts before Oyatie Cloud hosts the tenant."
                            </p>
                        </div>
                        {onboarding_anchor_board()}
                        <div class="onboarding-flow">
                            <div class="onboarding-progress">
                                <strong data-onboarding-percent="true">"56%"</strong>
                                <span class="score-bar" role="progressbar" aria-valuenow="56" aria-valuemin="0" aria-valuemax="100" aria-label="Onboarding progress: 56%" style="--bar: 56%"><em aria-hidden="true"></em></span>
                                <button type="button" data-identity-action="advance-onboarding">"다음 단계 완료"</button>
                            </div>
                            <article class="setup-current-step">
                                <p class="screen-anchor">"CURRENT STEP"</p>
                                <h5>"급여 캘린더 확인"</h5>
                                <p>"지급일, 근태 마감, 원천세 신고 마감이 이번 달 워크플로우와 일치하는지 검토합니다."</p>
                                <ul>
                                    <li>"✓ 법인 정보 검증 완료"</li>
                                    <li>"✓ 은행 출금 계좌 검증 완료"</li>
                                    <li>"○ 캘린더 승인 대기"</li>
                                </ul>
                            </article>
                            <ol class="onboarding-steps">
                                <li class="done">"1 환영합니다"</li>
                                <li class="done">"2 법인 정보"</li>
                                <li class="done">"3 은행 · 결제"</li>
                                <li class="active">"4 급여 캘린더"</li>
                                <li>"5 4대보험 가입"</li>
                                <li>"6 관할"</li>
                                <li>"7 직원 가져오기"</li>
                                <li>"8 보존 · 정책"</li>
                                <li>"9 검토 · 가동"</li>
                            </ol>
                        </div>
                    </article>
                </div>

                <aside class="identity-context-rail" aria-label="Identity provenance">
                    <p class="screen-anchor">"PROVENANCE"</p>
                    <dl class="service-kv">
                        <div><dt>"Actor"</dt><dd>"Choi Yu-na · Admin"</dd></div>
                        <div><dt>"Scope"</dt><dd>"Auth · org · people"</dd></div>
                        <div><dt>"Receipt"</dt><dd>"REC-ID-2026-05"</dd></div>
                    </dl>
                    <p class="screen-anchor">"LOCAL ONLY"</p>
                    <ol class="notification-stack">
                        <li>"No real auth mutation"</li>
                        <li>"No external HR system write"</li>
                        <li>"Audit preview staged visually"</li>
                    </ol>
                </aside>
            </div>
        </section>
    }
}

# Workflow Studio — Arabic (Saudi Arabia) overlay per ADR-0206.
# RTL locale; rendered with dir="rtl" per docs/standards/rtl-rendering.md.
# Translation seed — to be reviewed by qualified Arabic localizer before promotion to production.

brand-workflow-studio = استوديو سير العمل
nav-canvas = اللوحة
nav-runs = التشغيل
nav-library = المكتبة
nav-settings = الإعدادات

workflow-studio-canvas-title = لوحة استوديو سير العمل
workflow-studio-canvas-empty-state = اسحب عقدة من المكتبة للبدء.
workflow-studio-canvas-node-count =
    { $count ->
        [0] لا توجد عقد بعد
        [one] عقدة واحدة
        [two] عقدتان
        [few] { $count } عقد
        [many] { $count } عقدة
       *[other] { $count } عقدة
    }
workflow-studio-canvas-zoom-percent = { $zoom }٪

collab-status-syncing = جاري المزامنة…
collab-status-synced = محدث
collab-status-offline = غير متصل

run-status-pending = قيد الانتظار
run-status-running = قيد التشغيل
run-status-succeeded = نجح
run-status-failed = فشل
run-status-cancelled = ألغي

a11y-skip-to-content = تخطي إلى المحتوى الرئيسي

error-canvas-load-failed = تعذر تحميل اللوحة. { $details }

dsar-request-export = تصدير بياناتي
dsar-request-delete = حذف بياناتي
dsar-request-rectify = تصحيح بياناتي

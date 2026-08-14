mod behavior_reports;
mod support;

use support::behavior::report_evaluation_time;
use support::md_report::write_reports_at;

#[test]
fn writes_behavior_markdown_reports() {
    let report_time = report_evaluation_time();
    let summary = write_reports_at(
        vec![
            behavior_reports::modifier::report(),
            behavior_reports::catalog_item::report(),
            behavior_reports::order_item::report(),
            behavior_reports::label::report(),
            behavior_reports::media::report(),
            behavior_reports::time::report(),
            behavior_reports::calendar::report(),
            behavior_reports::schedule::report(),
            behavior_reports::supply::report(),
        ],
        &report_time,
    )
    .unwrap();

    assert_eq!(
        summary.failed,
        0,
        "{} described test cases failed; see {}",
        summary.failed,
        summary.output_dir.join("index.md").display()
    );
    assert!(summary.total > 0);
}

use std::any::Any;
use std::fmt::Write as _;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;

use pos_core_kernel::prelude::EvaluationTime;

pub struct ReportCase {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(),
}

pub struct ModuleReport {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub cases: Vec<ReportCase>,
}

#[derive(Debug)]
pub struct ReportSummary {
    pub output_dir: PathBuf,
    pub total: usize,
    pub failed: usize,
}

struct ModuleOutcome {
    slug: &'static str,
    title: &'static str,
    description: &'static str,
    cases: Vec<CaseOutcome>,
}

struct CaseOutcome {
    name: &'static str,
    description: &'static str,
    status: CaseStatus,
    duration_ms: u128,
}

enum CaseStatus {
    Passed,
    Failed(String),
}

pub fn write_reports_at(
    modules: Vec<ModuleReport>,
    generated_at_time: &EvaluationTime,
) -> std::io::Result<ReportSummary> {
    let output_dir = report_output_dir();
    fs::create_dir_all(&output_dir)?;

    let outcomes: Vec<_> = modules.into_iter().map(run_module).collect();
    let generated_at =
        format_unix_timestamp(generated_at_time.utc_time().unix_millis().div_euclid(1000));

    for module in &outcomes {
        fs::write(
            output_dir.join(format!("{}.md", module.slug)),
            render_module(module, &generated_at),
        )?;
    }

    fs::write(
        output_dir.join("index.md"),
        render_index(&outcomes, &generated_at),
    )?;

    let total = outcomes.iter().map(|module| module.cases.len()).sum();
    let failed = outcomes
        .iter()
        .flat_map(|module| &module.cases)
        .filter(|case| matches!(case.status, CaseStatus::Failed(_)))
        .count();

    Ok(ReportSummary {
        output_dir,
        total,
        failed,
    })
}

fn report_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-coverage")
}

fn run_module(module: ModuleReport) -> ModuleOutcome {
    let cases = module.cases.into_iter().map(run_case).collect();

    ModuleOutcome {
        slug: module.slug,
        title: module.title,
        description: module.description,
        cases,
    }
}

fn run_case(case: ReportCase) -> CaseOutcome {
    let result = panic::catch_unwind(AssertUnwindSafe(|| (case.run)()));
    let status = match result {
        Ok(()) => CaseStatus::Passed,
        Err(error) => CaseStatus::Failed(panic_message(error)),
    };

    CaseOutcome {
        name: case.name,
        description: case.description,
        status,
        duration_ms: 0,
    }
}

fn render_index(modules: &[ModuleOutcome], generated_at: &str) -> String {
    let total: usize = modules.iter().map(|module| module.cases.len()).sum();
    let failed: usize = modules.iter().map(ModuleOutcome::failed).sum();
    let mut md = String::new();

    writeln!(&mut md, "# Test Report").unwrap();
    writeln!(&mut md).unwrap();
    writeln!(&mut md, "- Generated: {generated_at}").unwrap();
    writeln!(&mut md, "- Total cases: {total}").unwrap();
    writeln!(&mut md, "- Passed: {}", total - failed).unwrap();
    writeln!(&mut md, "- Failed: {failed}").unwrap();
    writeln!(&mut md).unwrap();
    writeln!(&mut md, "| Module | Description | Last run | Status |").unwrap();
    writeln!(&mut md, "| --- | --- | --- | --- |").unwrap();

    for module in modules {
        let status = if module.failed() == 0 {
            "Passed".to_owned()
        } else {
            format!("Failed ({})", module.failed())
        };

        writeln!(
            &mut md,
            "| [{}]({}.md) | {} | {} | {} |",
            escape_markdown(module.title),
            module.slug,
            escape_markdown(module.description),
            generated_at,
            status
        )
        .unwrap();
    }

    md
}

fn render_module(module: &ModuleOutcome, generated_at: &str) -> String {
    let total = module.cases.len();
    let failed = module.failed();
    let mut md = String::new();

    writeln!(&mut md, "# {}", module.title).unwrap();
    writeln!(&mut md).unwrap();
    writeln!(&mut md, "{}", module.description).unwrap();
    writeln!(&mut md).unwrap();
    writeln!(&mut md, "- Generated: {generated_at}").unwrap();
    writeln!(&mut md, "- Total cases: {total}").unwrap();
    writeln!(&mut md, "- Passed: {}", total - failed).unwrap();
    writeln!(&mut md, "- Failed: {failed}").unwrap();
    writeln!(&mut md).unwrap();
    writeln!(&mut md, "| Test | Description | Status | Time |").unwrap();
    writeln!(&mut md, "| --- | --- | --- | ---: |").unwrap();

    for case in &module.cases {
        let status = match &case.status {
            CaseStatus::Passed => "Passed".to_owned(),
            CaseStatus::Failed(message) => {
                format!("Failed: {}", escape_markdown(message))
            }
        };

        writeln!(
            &mut md,
            "| {} | {} | {} | {} ms |",
            escape_markdown(case.name),
            escape_markdown(case.description),
            status,
            case.duration_ms
        )
        .unwrap();
    }

    md
}

impl ModuleOutcome {
    fn failed(&self) -> usize {
        self.cases
            .iter()
            .filter(|case| matches!(case.status, CaseStatus::Failed(_)))
            .count()
    }
}

fn panic_message(error: Box<dyn Any + Send>) -> String {
    if let Some(message) = error.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = error.downcast_ref::<String>() {
        message.clone()
    } else {
        "test panicked without a string message".to_owned()
    }
}

fn format_unix_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = seconds_of_day % 3_600 / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let days = days_since_unix_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = month_index + if month_index < 10 { 3 } else { -9 };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

fn escape_markdown(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\r', " ")
        .replace('\n', "<br>")
}

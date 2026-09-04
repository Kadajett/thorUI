use crate::TaskResult;
use std::collections::BTreeSet;
use std::fs;
use thorui_lab::report::CapabilityReport;

pub fn validate(paths: &[String]) -> TaskResult {
    if paths.is_empty() {
        return Err("provide the matching main and companion report paths".into());
    }
    let reports = paths
        .iter()
        .map(|path| read_report(path))
        .collect::<TaskResult<Vec<_>>>()?;
    let findings = findings(&reports);
    if findings.is_empty() {
        println!("Thor capability report pair is complete");
        return Ok(());
    }
    for finding in &findings {
        eprintln!("- {finding}");
    }
    Err(format!("report pair has {} incomplete checks", findings.len()).into())
}

fn read_report(path: &str) -> TaskResult<CapabilityReport> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(Into::into)
}

fn findings(reports: &[CapabilityReport]) -> Vec<String> {
    let mut findings = Vec::new();
    validate_roles(reports, &mut findings);
    validate_surfaces(reports, &mut findings);
    require(
        reports
            .iter()
            .any(|report| has_frame_run(report, "main", 120)),
        "missing main-surface 120 Hz frame run",
        &mut findings,
    );
    require(
        reports
            .iter()
            .any(|report| has_frame_run(report, "companion", 60)),
        "missing companion-surface 60 Hz frame run",
        &mut findings,
    );
    require(
        reports.iter().any(has_controller_press),
        "no controller press was captured",
        &mut findings,
    );
    require(
        reports.iter().any(has_touch),
        "no touch pointer was captured",
        &mut findings,
    );
    require(
        reports
            .iter()
            .all(|report| report.peer_link.round_trip_ms.samples >= 16),
        "both surfaces need at least 16 peer round trips",
        &mut findings,
    );
    require(
        reports
            .iter()
            .map(|report| report.notes.len())
            .sum::<usize>()
            >= 4,
        "add notes for manual lifecycle and placement outcomes",
        &mut findings,
    );
    findings
}

fn validate_roles(reports: &[CapabilityReport], findings: &mut Vec<String>) {
    let roles = reports
        .iter()
        .map(|report| report.surface.role.as_str())
        .collect::<BTreeSet<_>>();
    require(
        roles.contains("main"),
        "missing main-surface report",
        findings,
    );
    require(
        roles.contains("companion"),
        "missing companion-surface report",
        findings,
    );
}

fn validate_surfaces(reports: &[CapabilityReport], findings: &mut Vec<String>) {
    for report in reports {
        let surface = &report.surface;
        if surface.viewport_width_css <= 0.0
            || surface.viewport_height_css <= 0.0
            || surface.device_pixel_ratio <= 0.0
        {
            findings.push(format!(
                "{} has incomplete surface dimensions",
                surface.role
            ));
        }
        if report.active_probes.service_worker_result != "registered" {
            findings.push(format!(
                "{} did not register its offline worker",
                surface.role
            ));
        }
    }
}

fn has_frame_run(report: &CapabilityReport, role: &str, expected_hz: u16) -> bool {
    report.frame_runs.iter().any(|run| {
        run.surface_role == role
            && run.expected_hz == expected_hz
            && run.distribution.samples >= 120
    })
}

fn has_controller_press(report: &CapabilityReport) -> bool {
    report
        .controllers
        .iter()
        .any(|controller| controller.buttons.iter().any(|button| button.pressed))
}

fn has_touch(report: &CapabilityReport) -> bool {
    report
        .pointer_samples
        .iter()
        .any(|sample| sample.pointer_type == "touch" && sample.event == "pointerdown")
}

fn require(condition: bool, finding: &str, findings: &mut Vec<String>) {
    if !condition {
        findings.push(finding.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::findings;
    use thorui_lab::report::CapabilityReport;

    #[test]
    fn empty_report_pair_lists_missing_evidence() {
        let findings = findings(&[CapabilityReport::default()]);
        assert!(
            findings
                .iter()
                .any(|finding| finding.contains("main-surface"))
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.contains("controller"))
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.contains("round trips"))
        );
    }
}
